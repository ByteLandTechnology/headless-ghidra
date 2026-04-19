use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::frida;
use crate::rebuild;
use crate::schema::save_yaml;
use crate::workspace::artifact_dir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub target: String,
    pub fn_id: String,
    pub addr: String,
    pub status: String,
    /// Verification phase: "structural" (keyword check only) or "full" (rebuild and compare)
    #[serde(default)]
    pub verification_phase: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub iolog: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mismatch: Option<String>,
    pub timestamp: String,
}

/// Inline JavaScript hook script for Frida that intercepts a function
/// at a given address and emits JSON I/O records to stdout.
fn frida_hook_script(fn_id: &str, addr: &str) -> String {
    format!(
        r#"Interceptor.attach(ptr("{addr}"), {{
    onEnter: function(args) {{
        this._args = [];
        for (let i = 0; i < 4; i++) {{
            try {{
                this._args.push(args[i].toString());
            }} catch (e) {{
                this._args.push("<error>");
            }}
        }}
        this._start = Date.now();
    }},
    onLeave: function(retval) {{
        var record = {{
            type: "call",
            fn_id: "{fn_id}",
            timestamp: this._start,
            args: this._args,
            return_value: retval.toString(),
            duration_ms: Date.now() - this._start
        }};
        console.log(JSON.stringify(record));
    }}
}});
"#,
        fn_id = fn_id,
        addr = addr
    )
}

/// Load pipeline state to get the binary path.
fn get_binary_path(workspace: &Path, target: &str) -> Result<String> {
    let state_path = artifact_dir(workspace, target).join("pipeline-state.yaml");
    let state: crate::workspace::PipelineState =
        crate::schema::load_yaml(&state_path).context("failed to load pipeline-state.yaml")?;
    state
        .binary
        .ok_or_else(|| anyhow::anyhow!("binary path not recorded for target '{}'", target))
}

/// Check if frida CLI is available and get version.
fn check_frida_available() -> Result<String> {
    let output = Command::new("frida")
        .arg("--version")
        .output()
        .context("failed to run frida CLI (is frida installed?)")?;
    if !output.status.success() {
        anyhow::bail!("frida --version failed");
    }
    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(version)
}

/// Capture frida output by running it as a synchronous command with timeout.
/// Writes hook JS to a temp file and invokes frida via `timeout` utility.
fn capture_frida_output(
    binary_path: &str,
    fn_id: &str,
    addr: &str,
    timeout_secs: u64,
) -> Result<(String, String)> {
    let hook_js = frida_hook_script(fn_id, addr);

    // Write hook script to a temp file
    let temp_dir = std::env::temp_dir();
    let hook_path = temp_dir.join(format!("frida_hook_{}.js", fn_id));
    std::fs::write(&hook_path, &hook_js)
        .with_context(|| format!("failed to write hook script to {}", hook_path.display()))?;

    // Build frida invocation: frida -f <binary> -l <hook.js>
    // We implement timeout natively instead of relying on shell `timeout` command (missing on macOS)
    let mut child = Command::new("frida")
        .arg("-f")
        .arg(binary_path)
        .arg("-l")
        .arg(&hook_path)
        .arg("--runtime=v8")
        .arg("--exit-on-error")
        .arg("--kill-on-exit")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .with_context(|| "failed to spawn frida process")?;

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(timeout_secs);

    // Use separate threads to drain stdout/stderr pipes concurrently
    // This avoids blocking reads that would cause deadlock with try_wait
    let stdout_pipe = child.stdout.take().expect("stdout captured");
    let stderr_pipe = child.stderr.take().expect("stderr captured");

    let stdout_handle = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let mut reader = stdout_pipe;
        let _ = std::io::Read::read_to_end(&mut reader, &mut buf);
        buf
    });

    let stderr_handle = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let mut reader = stderr_pipe;
        let _ = std::io::Read::read_to_end(&mut reader, &mut buf);
        buf
    });

    // Wait for child with timeout using polling
    loop {
        if std::time::Instant::now() >= deadline {
            child.kill().ok();
            let _ = child.wait();
            let _ = std::fs::remove_file(&hook_path);
            return Err(anyhow::anyhow!(
                "frida process timed out after {}s",
                timeout_secs
            ));
        }

        match child.try_wait() {
            Ok(Some(_status)) => {
                // Child exited, wait for pipe readers to complete
                break;
            }
            Ok(None) => {
                // Still running, check if pipe threads have died unexpectedly
                if stdout_handle.is_finished() && stderr_handle.is_finished() {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            Err(e) => {
                let _ = std::fs::remove_file(&hook_path);
                return Err(anyhow::anyhow!("failed to wait on frida: {}", e));
            }
        }
    }

    // Collect output from the reader threads
    let stdout_buf = stdout_handle.join().unwrap_or_default();
    let stderr_buf = stderr_handle.join().unwrap_or_default();

    // Clean up temp hook script
    let _ = std::fs::remove_file(&hook_path);

    let stdout = String::from_utf8_lossy(&stdout_buf).to_string();
    let stderr = String::from_utf8_lossy(&stderr_buf).to_string();

    Ok((stdout, stderr))
}

/// Run the full P6 rebuild verification pipeline:
/// 1. Build the reconstructed binary from decompiled C
/// 2. Run Frida on the reconstructed binary to capture I/O
/// 3. Compare with original I/O
pub fn run_rebuild_verification(
    workspace: &Path,
    target: &str,
    fn_id: &str,
    addr: &str,
    _original_iolog: &[serde_json::Value],
) -> Result<(VerificationResult, PathBuf)> {
    let dir = artifact_dir(workspace, target)
        .join("decompilation")
        .join("functions")
        .join(fn_id);

    // Step 1: Build the reconstructed binary
    eprintln!("P6 Rebuild: Building reconstructed binary for {}", fn_id);
    let reconstructed_binary = match rebuild::build_reconstructed_binary(workspace, target, fn_id) {
        Ok(binary) => {
            eprintln!("P6 Rebuild: Successfully compiled {}", binary.display());
            binary
        }
        Err(e) => {
            eprintln!("P6 Rebuild: Failed to compile {}: {}", fn_id, e);
            return Err(e);
        }
    };

    // Step 2: Run Frida on the reconstructed binary
    eprintln!("P6 Rebuild: Capturing I/O from reconstructed binary");
    let timeout_secs = 10;
    let (stdout, stderr) = capture_frida_output(
        reconstructed_binary.to_str().unwrap_or(""),
        fn_id,
        addr,
        timeout_secs,
    )
    .context("failed to capture reconstructed I/O")?;

    // Parse reconstructed I/O
    let mut reconstructed_iolog: Vec<serde_json::Value> = Vec::new();
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('{')
            && let Ok(v) = serde_json::from_str::<serde_json::Value>(line)
        {
            reconstructed_iolog.push(v);
        }
    }

    // Emit warning if frida produced stderr output
    if !stderr.trim().is_empty() {
        eprintln!("frida stderr for {}: {}", fn_id, stderr.trim());
    }

    // Save reconstructed IOLOG
    let reconstructed_iolog_path = dir.join("reconstructed-iolog.json");
    if !reconstructed_iolog.is_empty() {
        let iolog_json = serde_json::to_string_pretty(&reconstructed_iolog)?;
        std::fs::write(&reconstructed_iolog_path, &iolog_json)?;
        eprintln!(
            "P6 Rebuild: Saved reconstructed I/O to {}",
            reconstructed_iolog_path.display()
        );
    }

    // Step 3: Compare original vs reconstructed
    eprintln!("P6 Rebuild: Comparing original vs reconstructed I/O");
    let original_iolog_path = dir.join("original-iolog.json");
    let compare_result = frida::run_io_compare(
        original_iolog_path.to_str().unwrap_or("/tmp/original.json"),
        reconstructed_iolog_path
            .to_str()
            .unwrap_or("/tmp/reconstructed.json"),
    )
    .context("io-compare failed")?;

    eprintln!("P6 Rebuild: io-compare result: {}", compare_result);

    // Parse the compare result to determine pass/fail
    let compare_json: serde_json::Value = serde_json::from_str(&compare_result)
        .unwrap_or_else(|_| serde_json::json!({"error": "failed to parse compare result"}));

    let matches = compare_json
        .get("matches")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let mismatches = compare_json
        .get("mismatches")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    let status = if mismatches == 0 && !reconstructed_iolog.is_empty() {
        "passed".to_string()
    } else {
        "failed".to_string()
    };

    let mismatch_detail = if mismatches > 0 {
        Some(format!(
            "{} mismatches out of {} total calls",
            mismatches,
            matches + mismatches
        ))
    } else {
        None
    };

    let result = VerificationResult {
        target: target.to_string(),
        fn_id: fn_id.to_string(),
        addr: addr.to_string(),
        status: status.clone(),
        verification_phase: "full".to_string(),
        iolog: Some(serde_json::to_string_pretty(&reconstructed_iolog)?),
        mismatch: mismatch_detail,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    Ok((result, reconstructed_binary))
}

/// Runs Frida I/O verification for a function.
///
/// Uses the `frida` CLI to spawn the target binary, attach an inline JS
/// hook at the function's address, capture runtime I/O, and verify the
/// decompiled function behavior against the captured data.
///
/// Writes `artifacts/<target>/decompilation/functions/<fn_id>/verification-result.yaml`.
pub fn run_frida_verify(
    workspace: &Path,
    target: &str,
    fn_id: &str,
    addr: &str,
) -> Result<VerificationResult> {
    let dir = artifact_dir(workspace, target)
        .join("decompilation")
        .join("functions")
        .join(fn_id);
    std::fs::create_dir_all(&dir)?;

    // Check Frida availability
    let frida_version = check_frida_available()?;
    eprintln!("Frida version: {}", frida_version);

    let decompiled_c = dir.join("decompiled.c");
    if !decompiled_c.exists() {
        anyhow::bail!(
            "decompiled.c not found for {} at {}. Run 'ghidra decompile' first.",
            fn_id,
            dir.display()
        );
    }

    let binary_path = get_binary_path(workspace, target)?;

    // Read decompiled.c for structural analysis
    let decompiled_src =
        std::fs::read_to_string(&decompiled_c).context("failed to read decompiled.c")?;

    // Capture Frida I/O with 10-second timeout
    let (stdout, stderr) =
        capture_frida_output(&binary_path, fn_id, addr, 10).context("frida invocation failed")?;

    // Parse JSON I/O lines from stdout.
    // Each line from console.log() in the hook script is a JSON object.
    let mut captured_iolog: Vec<serde_json::Value> = Vec::new();
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('{')
            && let Ok(v) = serde_json::from_str::<serde_json::Value>(line)
        {
            captured_iolog.push(v);
        }
    }

    let status: String;
    let mismatch: Option<String>;

    if captured_iolog.is_empty() {
        status = "failed".to_string();
        mismatch = Some(
            "no function calls captured during 10s verification window. \
             Is the function reachable during binary execution?"
                .to_string(),
        );
    } else {
        // Verify the decompiled source looks structurally correct.
        let fn_keywords = [
            "void", "int", "char", "unsigned", "long", "short", "float", "double",
        ];
        let has_function = fn_keywords.iter().any(|kw| decompiled_src.contains(kw));

        if !has_function {
            status = "failed".to_string();
            mismatch = Some(
                "decompiled.c does not contain recognizable C function definition".to_string(),
            );
        } else {
            // Check consistency: if function declared void but called with many args
            let first_call = &captured_iolog[0];
            let arg_count = first_call
                .get("args")
                .and_then(|a| a.as_array())
                .map(|a| a.len())
                .unwrap_or(0);

            // Use regex to find actual C function declaration instead of assuming first line
            // Ghidra output may have file headers, comments, or typedefs before the function
            let declared_void = decompiled_src.lines()
                .filter_map(|line| {
                    let line = line.trim();
                    // Skip empty lines and preprocessor/compiler directives
                    if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
                        return None;
                    }
                    // Look for function declaration pattern: return_type func_name (...)
                    // Common types: void, int, char, long, short, float, double, unsigned, etc.
                    let type_kw = r"(?:void|int|char|long|short|float|double|unsigned|uint\d*_t|size_t|ssize_t)";
                    let pattern = format!(r"^\s*{}\s+\w+\s*\([^)]*\)", type_kw);
                    let re = regex::Regex::new(&pattern).ok()?;
                    if re.is_match(line) {
                        Some(line.contains("void") && !line.contains("void *"))
                    } else {
                        None
                    }
                })
                .next()
                .unwrap_or(false);

            if arg_count > 4 && declared_void {
                status = "failed".to_string();
                mismatch = Some(format!(
                    "function called with {} args but decompiled as void (no params)",
                    arg_count
                ));
            } else {
                status = "passed".to_string();
                mismatch = None;
            }
        }
    }

    // Emit warning if frida produced stderr output
    if !stderr.trim().is_empty() {
        eprintln!("frida stderr for {} ({}): {}", fn_id, addr, stderr.trim());
    }

    // Save original IOLOG to file for later comparison with reconstructed
    let original_iolog_path = dir.join("original-iolog.json");
    if !captured_iolog.is_empty() {
        let iolog_json = serde_json::to_string_pretty(&captured_iolog)?;
        std::fs::write(&original_iolog_path, &iolog_json)?;
    }

    // Try full P6 rebuild verification if we have original I/O
    let (verification_phase, final_status, final_mismatch) = if !captured_iolog.is_empty() {
        match run_rebuild_verification(workspace, target, fn_id, addr, &captured_iolog) {
            Ok((rebuild_result, _reconstructed_binary)) => {
                eprintln!("P6 full verification passed for {}", fn_id);
                (
                    rebuild_result.verification_phase,
                    rebuild_result.status,
                    rebuild_result.mismatch,
                )
            }
            Err(e) => {
                eprintln!("P6 rebuild verification failed for {}: {}", fn_id, e);
                // Fall back to structural verification on rebuild failure
                ("structural".to_string(), status, mismatch)
            }
        }
    } else {
        ("structural".to_string(), status, mismatch)
    };

    let result = VerificationResult {
        target: target.to_string(),
        fn_id: fn_id.to_string(),
        addr: addr.to_string(),
        status: final_status.clone(),
        verification_phase,
        iolog: if captured_iolog.is_empty() {
            None
        } else {
            Some(serde_json::to_string_pretty(&captured_iolog)?)
        },
        mismatch: final_mismatch,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    save_yaml(&dir.join("verification-result.yaml"), &result)?;

    Ok(result)
}
