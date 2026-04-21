use crate::frida::device::{DeviceSelector, run_frida_with_device};
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::io::Write;

pub const IO_CAPTURE_JS: &str = include_str!("../../frida-scripts/io-capture.js");
pub const SIGNATURE_ANALYSIS_JS: &str = include_str!("../../frida-scripts/signature-analysis.js");
pub const CALL_TREE_TRACE_JS: &str = include_str!("../../frida-scripts/call-tree-trace.js");
pub const DISPATCH_VTABLE_TRACE_JS: &str =
    include_str!("../../frida-scripts/dispatch-vtable-trace.js");
pub const HOTPATH_COVERAGE_JS: &str = include_str!("../../frida-scripts/hotpath-coverage.js");
pub const IO_COMPARE_JS: &str = include_str!("../../frida-scripts/io-compare.js");
pub const DECOMP_COMPARE_JS: &str = include_str!("../../frida-scripts/decomp-compare.js");
pub const FUZZ_INPUT_GEN_JS: &str = include_str!("../../frida-scripts/fuzz-input-gen.js");
pub const FRIDA_RUN_JS: &str = include_str!("../../frida-scripts/frida-run.js");
pub const FRIDA_TRACE_JS: &str = include_str!("../../frida-scripts/frida-trace.js");
pub const FRIDA_INVOKE_JS: &str = include_str!("../../frida-scripts/frida-invoke.js");

pub struct ScriptRunner {
    pub device: DeviceSelector,
    pub spawn_target: Option<String>,
    pub env_vars: HashMap<String, String>,
    pub proc_args: Vec<String>,
    pub timeout_secs: u64,
}

impl Default for ScriptRunner {
    fn default() -> Self {
        Self {
            device: DeviceSelector::Local,
            spawn_target: None,
            env_vars: HashMap::new(),
            proc_args: Vec::new(),
            timeout_secs: 60,
        }
    }
}

impl ScriptRunner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn device(mut self, device: DeviceSelector) -> Self {
        self.device = device;
        self
    }

    pub fn spawn_target(mut self, target: &str) -> Self {
        self.spawn_target = Some(target.to_string());
        self
    }

    pub fn env(mut self, key: &str, value: &str) -> Self {
        self.env_vars.insert(key.to_string(), value.to_string());
        self
    }

    pub fn proc_args(mut self, args: &[&str]) -> Self {
        self.proc_args = args.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    pub fn run(&self, script: &str) -> Result<String> {
        let temp_dir = std::env::temp_dir();
        let script_path = temp_dir.join(format!("frida_script_{}.js", std::process::id()));

        // Build a JSON env block and prepend it; also replace legacy %%KEY%% placeholders
        let env_json = serde_json::to_string(&self.env_vars)
            .unwrap_or_else(|_| "{}".to_string());
        let env_block = format!("const __ENV_PARAMS__ = {env_json};\n");

        let mut modified_script = script.to_string();
        // Replace legacy %%KEY%% placeholders with JSON-safe lookups
        for key in self.env_vars.keys() {
            let placeholder = format!("%%{}%%", key);
            let replacement = format!("__ENV_PARAMS__[\"{}\"]", key);
            modified_script = modified_script.replace(&placeholder, &replacement);
        }

        let full_script = format!("{env_block}{modified_script}");

        {
            let mut file = fs::File::create(&script_path)?;
            file.write_all(full_script.as_bytes())?;
        }

        let target = self.spawn_target.as_deref();
        let proc_refs: Vec<&str> = self.proc_args.iter().map(|s| s.as_str()).collect();
        let (stdout, stderr) = run_frida_with_device(
            &self.device,
            script_path.to_str().unwrap(),
            target,
            &proc_refs,
            self.timeout_secs,
        )?;

        let _ = fs::remove_file(&script_path);

        if !stderr.is_empty() && stderr.contains("error") {
            eprintln!("Frida warning: {}", stderr);
        }

        Ok(stdout)
    }
}

pub fn run_io_capture(
    target: &str,
    proc_args: Option<&str>,
    device: &DeviceSelector,
    timeout: u64,
) -> Result<String> {
    let args_vec: Vec<&str> = proc_args
        .map(|a| a.split_whitespace().collect())
        .unwrap_or_default();

    ScriptRunner::new()
        .device(device.clone())
        .timeout(timeout)
        .spawn_target(target)
        .proc_args(&args_vec)
        .run(IO_CAPTURE_JS)
}

pub fn run_signature_analysis(
    spawn_target: Option<&str>,
    funcs: &str,
    proc_args: Option<&str>,
    device: &DeviceSelector,
) -> Result<String> {
    let args_vec: Vec<&str> = proc_args
        .map(|a| a.split_whitespace().collect())
        .unwrap_or_default();

    let mut runner = ScriptRunner::new()
        .device(device.clone())
        .proc_args(&args_vec);
    if let Some(target) = spawn_target {
        runner = runner.spawn_target(target);
    }
    runner = runner.env("FUNCS", funcs);
    runner.run(SIGNATURE_ANALYSIS_JS)
}

pub fn run_call_tree_trace(
    spawn_target: Option<&str>,
    max_depth: Option<u32>,
    libs: Option<&str>,
    proc_args: Option<&str>,
    device: &DeviceSelector,
) -> Result<String> {
    let args_vec: Vec<&str> = proc_args
        .map(|a| a.split_whitespace().collect())
        .unwrap_or_default();

    let mut runner = ScriptRunner::new()
        .device(device.clone())
        .proc_args(&args_vec);
    if let Some(target) = spawn_target {
        runner = runner.spawn_target(target);
    }
    if let Some(depth) = max_depth {
        runner = runner.env("MAX_DEPTH", &depth.to_string());
    }
    if let Some(l) = libs {
        runner = runner.env("LIBS", l);
    }
    runner.run(CALL_TREE_TRACE_JS)
}

pub fn run_dispatch_vtable_trace(ranges: &str, device: &DeviceSelector) -> Result<String> {
    ScriptRunner::new()
        .device(device.clone())
        .env("VTABLE_RANGES", ranges)
        .run(DISPATCH_VTABLE_TRACE_JS)
}

pub fn run_hotpath_coverage(
    spawn_target: Option<&str>,
    threshold: Option<u32>,
    interval: Option<u32>,
    proc_args: Option<&str>,
    device: &DeviceSelector,
) -> Result<String> {
    let args_vec: Vec<&str> = proc_args
        .map(|a| a.split_whitespace().collect())
        .unwrap_or_default();

    let mut runner = ScriptRunner::new()
        .device(device.clone())
        .proc_args(&args_vec);
    if let Some(target) = spawn_target {
        runner = runner.spawn_target(target);
    }
    if let Some(t) = threshold {
        runner = runner.env("HOT_THRESHOLD", &t.to_string());
    }
    if let Some(i) = interval {
        runner = runner.env("REPORT_INTERVAL_MS", &i.to_string());
    }
    runner.run(HOTPATH_COVERAGE_JS)
}

/// Compare two values with ASLR-aware pointer handling:
/// - NULL (0x0) vs non-NULL is a mismatch
/// - Both NULL (0x0) is a match
/// - Both non-NULL pointers with 0x prefix: skip comparison (ASLR)
/// - Otherwise: direct string comparison
fn compare_values_with_aslr(orig: &str, recon: &str) -> bool {
    let orig_is_null =
        orig == "0x0" || orig == "0x00" || orig == "0x00000000" || orig == "0x0000000000000000";
    let recon_is_null =
        recon == "0x0" || recon == "0x00" || recon == "0x00000000" || recon == "0x0000000000000000";

    if orig_is_null && recon_is_null {
        true
    } else if orig_is_null || recon_is_null {
        false
    } else if orig.starts_with("0x") && recon.starts_with("0x") {
        // Both non-NULL pointers - skip comparison due to ASLR
        true
    } else {
        orig == recon
    }
}

pub fn run_io_compare(original: &str, reconstructed: &str) -> Result<String> {
    // Pure Rust implementation - no Frida needed
    let original_content = std::fs::read_to_string(original)?;
    let reconstructed_content = std::fs::read_to_string(reconstructed)?;

    let original_log: serde_json::Value = serde_json::from_str(&original_content)
        .map_err(|e| anyhow::anyhow!("Failed to parse original log: {}", e))?;
    let reconstructed_log: serde_json::Value = serde_json::from_str(&reconstructed_content)
        .map_err(|e| anyhow::anyhow!("Failed to parse reconstructed log: {}", e))?;

    let orig_array = original_log
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("Original log must be an array"))?;
    let recon_array = reconstructed_log
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("Reconstructed log must be an array"))?;

    let mut matches = 0;
    let mut mismatches = 0;
    let mut results: Vec<serde_json::Value> = Vec::new();

    for i in 0..std::cmp::min(orig_array.len(), recon_array.len()) {
        let orig = &orig_array[i];
        let recon = &recon_array[i];

        let orig_type = orig.get("type").and_then(|v| v.as_str()).unwrap_or("");
        let orig_name = orig
            .get("name")
            .or_else(|| orig.get("function"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let recon_type = recon.get("type").and_then(|v| v.as_str()).unwrap_or("");
        let recon_name = recon
            .get("name")
            .or_else(|| recon.get("function"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if orig_type != recon_type || orig_name != recon_name {
            results.push(serde_json::json!({
                "match": false,
                "index": i,
                "reason": "type/name mismatch",
                "orig": orig,
                "recon": recon
            }));
            mismatches += 1;
            continue;
        }

        // Compare args element-wise with ASLR-aware pointer comparison
        let orig_args_arr = orig.get("args").and_then(|v| v.as_array());
        let recon_args_arr = recon.get("args").and_then(|v| v.as_array());

        let args_match = match (orig_args_arr, recon_args_arr) {
            (Some(oa), Some(ra)) => {
                if oa.len() != ra.len() {
                    false
                } else {
                    oa.iter().zip(ra.iter()).all(|(o, r)| {
                        let o_str = o.as_str().unwrap_or("");
                        let r_str = r.as_str().unwrap_or("");
                        compare_values_with_aslr(o_str, r_str)
                    })
                }
            }
            (None, None) => true,
            _ => false,
        };

        let orig_ret = orig
            .get("return_value")
            .or_else(|| orig.get("return_value"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let recon_ret = recon
            .get("return_value")
            .or_else(|| recon.get("return_value"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let ret_match = compare_values_with_aslr(orig_ret, recon_ret);

        if args_match && ret_match {
            matches += 1;
        } else {
            results.push(serde_json::json!({
                "match": false,
                "index": i,
                "args_match": args_match,
                "ret_match": ret_match,
                "orig_args": orig.get("args"),
                "recon_args": recon.get("args"),
                "orig_ret": orig_ret,
                "recon_ret": recon_ret
            }));
            mismatches += 1;
        }
    }

    let summary = serde_json::json!({
        "summary": true,
        "matches": matches,
        "mismatches": mismatches,
        "total": orig_array.len()
    });

    let mut output = Vec::new();
    for r in results {
        output.push(r.to_string());
    }
    output.push(summary.to_string());

    Ok(output.join("\n"))
}

pub fn run_decomp_compare(
    spawn_target: Option<&str>,
    func: &str,
    proc_args: Option<&str>,
    device: &DeviceSelector,
) -> Result<String> {
    let args_vec: Vec<&str> = proc_args
        .map(|a| a.split_whitespace().collect())
        .unwrap_or_default();

    let mut runner = ScriptRunner::new()
        .device(device.clone())
        .proc_args(&args_vec);
    if let Some(target) = spawn_target {
        runner = runner.spawn_target(target);
    }
    runner = runner.env("FUNC", func);
    runner.run(DECOMP_COMPARE_JS)
}

pub fn run_fuzz_input_gen(types_yaml: &str, output: Option<&str>) -> Result<String> {
    // Read types YAML and generate fuzz inputs
    let yaml_content = std::fs::read_to_string(types_yaml)?;
    let output_path = output.unwrap_or("/tmp/fuzz_inputs.json");

    // Parse YAML to extract type definitions
    let mut fuzz_inputs: Vec<serde_json::Value> = Vec::new();

    // Simple YAML parser for function signatures
    // Expected format:
    // functions:
    //   - name: add
    //     signature: int(int, int)
    //   - name: multiply
    //     signature: int(int, int)
    //
    // types:
    //   - name: int
    //     values: [0, 1, -1]
    //   - name: string
    //     values: ["", "A", "AAAA..."]

    let lines: Vec<&str> = yaml_content.lines().collect();
    let mut in_functions = false;
    let mut in_types = false;
    let mut current_name = String::new();

    for line in &lines {
        let trimmed = line.trim();

        if trimmed.starts_with("functions:") {
            in_functions = true;
            in_types = false;
            continue;
        } else if trimmed.starts_with("types:") {
            in_functions = false;
            in_types = true;
            continue;
        }

        if trimmed == "-" || trimmed.is_empty() {
            continue;
        }

        // Parse name: value pairs
        if let Some(name_pos) = trimmed.find("name:") {
            let name_part = trimmed[name_pos..].trim_start_matches("name:");
            current_name = name_part
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();
        }

        // Parse prototype: for functions (correct YAML key per schema)
        if in_functions
            && !current_name.is_empty()
            && let Some(sig_pos) = trimmed.find("prototype:")
        {
            let sig = trimmed[sig_pos..]
                .trim_start_matches("prototype:")
                .trim()
                .trim_matches('"')
                .trim_matches('\'');

            // Generate fuzz inputs from signature
            for input in generate_inputs_for_signature(sig) {
                fuzz_inputs.push(serde_json::json!({
                    "function": current_name,
                    "type": "call",
                    "input": input
                }));
            }
            current_name.clear();
        }

        // Parse values: for types
        if in_types
            && !current_name.is_empty()
            && let Some(values_pos) = trimmed.find("values:")
        {
            let values_str = trimmed[values_pos..].trim_start_matches("values:");
            // Try to parse as array
            if values_str.contains('[') {
                let array_str = values_str.trim_start_matches("values:").trim();
                if let Ok(values) = serde_json::from_str::<Vec<serde_json::Value>>(array_str) {
                    for val in values {
                        fuzz_inputs.push(serde_json::json!({
                            "type": current_name,
                            "value": val
                        }));
                    }
                }
            }
            current_name.clear();
        }
    }

    // If no inputs generated, use defaults
    if fuzz_inputs.is_empty() {
        fuzz_inputs = get_default_fuzz_inputs();
    }

    let output_content = serde_json::to_string_pretty(&fuzz_inputs)
        .map_err(|e| anyhow::anyhow!("Failed to serialize fuzz inputs: {}", e))?;
    std::fs::write(output_path, &output_content)?;

    Ok(format!(
        "# Generated {} fuzz inputs to: {}\n{}",
        fuzz_inputs.len(),
        output_path,
        output_content
    ))
}

fn generate_inputs_for_signature(signature: &str) -> Vec<String> {
    let mut inputs = Vec::new();

    // Extract parameter types from signature like "int(int, int)"
    if let Some(params_start) = signature.find('(') {
        let params_str = &signature[params_start + 1..signature.len() - 1];
        let param_types: Vec<&str> = params_str.split(',').map(|s| s.trim()).collect();

        for param_type in param_types {
            match param_type {
                "int" | "long" | "short" | "char" => {
                    inputs.push("0".to_string());
                    inputs.push("1".to_string());
                    inputs.push("-1".to_string());
                }
                "unsigned" | "uint" | "size_t" => {
                    inputs.push("0".to_string());
                    inputs.push("1".to_string());
                    inputs.push("0xffffffff".to_string());
                }
                "float" | "double" => {
                    inputs.push("0.0".to_string());
                    inputs.push("1.0".to_string());
                    inputs.push("-1.0".to_string());
                }
                "void*" | "char*" | "ptr" => {
                    inputs.push("NULL".to_string());
                    inputs.push("0x1".to_string());
                }
                "string" | "char[]" => {
                    inputs.push("\"\"".to_string());
                    inputs.push("\"A\"".to_string());
                    inputs.push("\"AAAA...\"".to_string());
                }
                _ => {
                    inputs.push("0".to_string());
                    inputs.push("NULL".to_string());
                }
            }
        }
    }

    if inputs.is_empty() {
        inputs.push("0".to_string());
        inputs.push("NULL".to_string());
    }

    inputs
}

fn get_default_fuzz_inputs() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({"type": "int", "value": 0}),
        serde_json::json!({"type": "int", "value": 1}),
        serde_json::json!({"type": "int", "value": -1}),
        serde_json::json!({"type": "uint", "value": "0xffffffff"}),
        serde_json::json!({"type": "pointer", "value": "NULL"}),
        serde_json::json!({"type": "string", "value": ""}),
        serde_json::json!({"type": "string", "value": "A"}),
        serde_json::json!({"type": "string", "value": "AAAAAAA"}),
    ]
}

pub fn run_script(
    script: &str,
    target: &str,
    args: Option<&str>,
    _stdin: Option<&str>,
    timeout: u64,
    device: &DeviceSelector,
) -> Result<String> {
    let proc_args: Vec<&str> = args
        .map(|a| a.split_whitespace().collect())
        .unwrap_or_default();

    ScriptRunner::new()
        .device(device.clone())
        .timeout(timeout)
        .spawn_target(target)
        .proc_args(&proc_args)
        .run(script)
}

pub fn run_trace_script(
    script: &str,
    target: &str,
    functions: Option<&str>,
    proc_args: Option<&str>,
    timeout: u64,
    device: &DeviceSelector,
) -> Result<String> {
    let args_vec: Vec<&str> = proc_args
        .map(|a| a.split_whitespace().collect())
        .unwrap_or_default();

    let mut runner = ScriptRunner::new()
        .device(device.clone())
        .timeout(timeout)
        .spawn_target(target)
        .proc_args(&args_vec);

    if let Some(f) = functions {
        runner = runner.env("TRACE_FUNCTIONS", f);
    }

    runner.run(script)
}

pub fn run_invoke_script(
    script: &str,
    target: &str,
    function: &str,
    signature: Option<&str>,
    args: Option<&str>,
    device: &DeviceSelector,
) -> Result<String> {
    let mut runner = ScriptRunner::new()
        .device(device.clone())
        .timeout(60)
        .spawn_target(target)
        .env("INVOKE_FUNCTION", function);

    if let Some(s) = signature {
        runner = runner.env("INVOKE_SIGNATURE", s);
    }
    if let Some(a) = args {
        runner = runner.env("INVOKE_ARGS", a);
    }

    runner.run(script)
}
