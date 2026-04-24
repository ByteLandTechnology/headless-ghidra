use anyhow::{Result, anyhow};
use std::io::Write;
use std::path::{Path, PathBuf};

const BUNDLED_ENTRY_SCRIPT_NAME: &str = "GhidraAgentCliEntry.java";
const BUNDLED_SCRIPT_NAMES: &[&str] = &[
    "ApplyRenames.java",
    "AnalyzeVtables.java",
    "ApplySignatures.java",
    "AutoAnalyze.java",
    "AnalyzeVtables.java",
    "DecompileFunction.java",
    "ExportBaseline.java",
    "ExportCallGraph.java",
    "IdentifyLibraries.java",
    "ImportTypesAndSignatures.java",
    "ImportBinary.java",
    "LintReviewArtifacts.java",
    "RebuildProject.java",
    "ReviewEvidenceCandidates.java",
    "VerifyFunctionSignatures.java",
    "VerifyRenames.java",
];

/// Search common Ghidra installation locations
fn search_ghidra_locations() -> Option<PathBuf> {
    let search_paths = [
        // macOS Applications
        PathBuf::from("/Applications"),
        // Home directory
        PathBuf::from(std::env::var("HOME").ok()?).join("ghidra"),
        // Homebrew on Apple Silicon (Cellar structure)
        PathBuf::from("/opt/homebrew/Cellar/ghidra"),
        // Homebrew prefix (opt symlink)
        PathBuf::from("/opt/homebrew/opt/ghidra"),
        // Homebrew on Intel
        PathBuf::from("/usr/local/Cellar/ghidra"),
        // /opt
        PathBuf::from("/opt/ghidra"),
        // /usr/local
        PathBuf::from("/usr/local/ghidra"),
        // Current directory (for dev)
        std::env::current_dir().ok()?,
    ];

    for base in &search_paths {
        // Check if this is a Ghidra root directly
        if base.join("support").join("analyzeHeadless").exists() {
            return Some(base.clone());
        }
        // Check if this is a versioned directory (e.g., /opt/homebrew/Cellar/ghidra/12.0.4)
        if let Ok(entries) = std::fs::read_dir(base) {
            for entry in entries.flatten() {
                let path = entry.path();
                // Check libexec (Homebrew Ghidra structure)
                // Entry is /opt/homebrew/Cellar/ghidra/12.0.4/libexec, so check path/support/analyzeHeadless directly
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                if name == "libexec" && path.join("support").join("analyzeHeadless").exists() {
                    return Some(path);
                }
                // Check for ghidra-*/support structure
                if path.is_dir() {
                    let name = path.file_name().unwrap_or_default().to_string_lossy();
                    let name_lower = name.to_lowercase();
                    if (name_lower.starts_with("ghidra")
                        || name
                            .chars()
                            .next()
                            .map(|c| c.is_ascii_digit())
                            .unwrap_or(false))
                        && path.join("support").join("analyzeHeadless").exists()
                    {
                        return Some(path);
                    }
                }
            }
        }
    }
    None
}

pub fn discover_ghidra(install_dir: Option<&Path>) -> Result<PathBuf> {
    // 1. Explicit path
    if let Some(dir) = install_dir {
        let headless = dir.join("support").join("analyzeHeadless");
        if headless.exists() {
            return Ok(dir.to_path_buf());
        }
        return Err(anyhow!("Ghidra not found at {}", dir.display()));
    }

    // 2. Environment variable
    if let Ok(dir) = std::env::var("GHIDRA_INSTALL_DIR") {
        let p = PathBuf::from(&dir);
        if p.join("support").join("analyzeHeadless").exists() {
            return Ok(p);
        }
    }

    // 3. Auto-search common locations
    if let Some(path) = search_ghidra_locations() {
        return Ok(path);
    }

    Err(anyhow!(
        "Ghidra not found. Set GHIDRA_INSTALL_DIR, use --install-dir, or install Ghidra in /Applications, ~/ghidra, or Homebrew locations"
    ))
}

pub fn ghidra_projects_dir(workspace: &Path, target: &str) -> PathBuf {
    workspace
        .join("targets")
        .join(target)
        .join("ghidra-projects")
}

/// Resolves the Ghidra scripts directory.
/// Priority:
/// 1. GHIDRA_SCRIPTS_DIR env var
/// 2. workspace/ghidra-script-bundle/
/// 3. workspace/ghidra-scripts/
/// 4. CLI binary's own directory (and its parent), preferring ghidra-script-bundle/
/// 5. CWD-based lookup, preferring ghidra-script-bundle/
/// 6. ghidra_dir's built-in scripts
pub fn resolve_scripts_dir(workspace: &Path, ghidra_dir: &Path) -> PathBuf {
    fn first_existing(candidates: &[PathBuf]) -> Option<PathBuf> {
        candidates.iter().find(|path| path.exists()).cloned()
    }

    // 1. Env var override
    if let Ok(dir) = std::env::var("GHIDRA_SCRIPTS_DIR") {
        let p = PathBuf::from(&dir);
        if p.exists() {
            return p;
        }
    }
    // 2. workspace/ghidra-script-bundle/
    let ws_bundle = workspace.join("ghidra-script-bundle");
    if ws_bundle.exists() {
        return ws_bundle;
    }
    // 3. workspace/ghidra-scripts/
    let ws_scripts = workspace.join("ghidra-scripts");
    if ws_scripts.exists() {
        return ws_scripts;
    }
    // 4. Walk up from CLI binary's directory to find ghidra-script-bundle/ or ghidra-scripts/
    //    Handles layouts like: <install>/target/release/ghidra-agent-cli
    //    with scripts at:      <install>/ghidra-script-bundle/ or <install>/ghidra-scripts/
    if let Ok(exe) = std::env::current_exe()
        && let Some(mut dir) = exe.parent()
    {
        loop {
            if let Some(candidate) =
                first_existing(&[dir.join("ghidra-script-bundle"), dir.join("ghidra-scripts")])
            {
                return candidate;
            }
            dir = match dir.parent() {
                Some(p) => p,
                None => break,
            };
        }
    }
    // 5. CWD-based lookup (for running from repo root)
    if let Ok(cwd) = std::env::current_dir() {
        if let Some(candidate) = first_existing(&[
            cwd.join("ghidra-agent-cli").join("ghidra-script-bundle"),
            cwd.join("ghidra-agent-cli").join("ghidra-scripts"),
            cwd.join("ghidra-script-bundle"),
            cwd.join("ghidra-scripts"),
        ]) {
            return candidate;
        }
        if let Some(parent) = cwd.parent()
            && let Some(candidate) = first_existing(&[
                parent.join("ghidra-script-bundle"),
                parent.join("ghidra-scripts"),
            ])
        {
            return candidate;
        }
    }
    // 6. Fall back to ghidra's built-in Base scripts (most useful)
    ghidra_dir
        .join("Ghidra")
        .join("Features")
        .join("Base")
        .join("ghidra_scripts")
}

fn bundled_entry_script_exists(scripts_dir: &Path) -> bool {
    scripts_dir.join(BUNDLED_ENTRY_SCRIPT_NAME).exists()
}

fn logical_script_name(script: &str) -> &str {
    Path::new(script)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(script)
}

fn should_use_bundled_entry(script: &str, scripts_dir: &Path) -> bool {
    bundled_entry_script_exists(scripts_dir)
        && BUNDLED_SCRIPT_NAMES.contains(&logical_script_name(script))
}

fn should_disable_auto_analysis(script: &str) -> bool {
    matches!(
        logical_script_name(script),
        "AnalyzeVtables.java"
            | "ApplyRenames.java"
            | "ApplySignatures.java"
            | "DecompileFunction.java"
            | "ExportBaseline.java"
            | "ImportTypesAndSignatures.java"
            | "VerifyFunctionSignatures.java"
            | "VerifyRenames.java"
    )
}

/// Run Ghidra analyzeHeadless with -import flag for binary import
/// Uses native -import which establishes program context for subsequent scripts
pub fn run_headless_import(
    _workspace: &Path,
    ghidra_dir: &Path,
    project_dir: &Path,
    target: &str,
    binary_path: &Path,
) -> Result<()> {
    let headless = ghidra_dir.join("support").join("analyzeHeadless");
    let mut cmd = std::process::Command::new(&headless);
    cmd.arg(project_dir)
        .arg(target)
        .arg("-import")
        .arg(binary_path);

    let status = cmd.status()?;
    if !status.success() {
        return Err(anyhow!(
            "Ghidra headless import failed with status {}",
            status
        ));
    }
    Ok(())
}

pub fn run_headless(
    workspace: &Path,
    ghidra_dir: &Path,
    project_dir: &Path,
    target: &str,
    script: &str,
    extra_args: &[&str],
) -> Result<()> {
    // Scripts handle their own program context (import/rebuild) - no -process needed
    run_headless_impl(
        workspace,
        ghidra_dir,
        project_dir,
        target,
        script,
        extra_args,
        None,
    )
}

/// Run Ghidra analyzeHeadless with an optional program to open via -process
pub fn run_headless_with_program(
    workspace: &Path,
    ghidra_dir: &Path,
    project_dir: &Path,
    target: &str,
    script: &str,
    extra_args: &[&str],
    program_name: Option<&str>,
) -> Result<()> {
    // When a program name is provided, use -process to open it first
    run_headless_impl(
        workspace,
        ghidra_dir,
        project_dir,
        target,
        script,
        extra_args,
        program_name,
    )
}

fn run_headless_impl(
    workspace: &Path,
    ghidra_dir: &Path,
    project_dir: &Path,
    target: &str,
    script: &str,
    extra_args: &[&str],
    program_name: Option<&str>,
) -> Result<()> {
    let headless = ghidra_dir.join("support").join("analyzeHeadless");
    let scripts_dir = resolve_scripts_dir(workspace, ghidra_dir);
    let use_bundled_entry = should_use_bundled_entry(script, &scripts_dir);
    let post_script_name = if use_bundled_entry {
        BUNDLED_ENTRY_SCRIPT_NAME
    } else {
        script
    };
    let post_script_path = scripts_dir.join(post_script_name);
    let post_script = if post_script_path.is_file() {
        post_script_path.to_string_lossy().into_owned()
    } else {
        post_script_name.to_string()
    };
    let mut cmd = std::process::Command::new(&headless);
    cmd.arg(project_dir).arg(target);

    // Only use -process if a program name is explicitly provided
    if let Some(name) = program_name {
        cmd.arg("-process").arg(name);
    }
    if should_disable_auto_analysis(script) {
        cmd.arg("-noanalysis");
    }

    cmd.arg("-postScript").arg(&post_script);
    if use_bundled_entry {
        cmd.arg(logical_script_name(script));
    }

    // Script args come before -scriptPath: workspace, target, then script-specific extras
    cmd.arg(workspace.to_string_lossy().as_ref());
    cmd.arg(target);
    for arg in extra_args {
        cmd.arg(arg);
    }

    cmd.arg("-scriptPath");
    cmd.arg(&scripts_dir);

    let output = cmd.output()?;
    std::io::stdout().write_all(&output.stdout)?;
    std::io::stderr().write_all(&output.stderr)?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let ghidra_reported_script_error =
        stdout.contains("SCRIPT ERROR") || stderr.contains("SCRIPT ERROR");

    if !output.status.success() || ghidra_reported_script_error {
        return Err(anyhow!(
            "Ghidra headless script {} failed with status {}",
            post_script,
            output.status
        ));
    }
    Ok(())
}
