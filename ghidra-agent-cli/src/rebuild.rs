//! P6 Rebuild Pipeline: Compile decompiled C code and run verification
//!
//! This module provides the infrastructure to:
//! 1. Generate a compilable test harness from decompiled C code
//! 2. Compile the harness with the decompiled function
//! 3. Run the compiled binary with Frida to capture reconstructed I/O
//! 4. Compare reconstructed I/O with original I/O

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Generate a compilable C test harness from decompiled code
/// This wraps the decompiled function in a main() that can be called with test inputs
pub fn generate_test_harness(
    decompiled_c: &Path,
    fn_id: &str,
    fn_addr: &str,
    fn_name: &str,
    prototype: &str,
) -> Result<String> {
    let decompiled_content = std::fs::read_to_string(decompiled_c)
        .with_context(|| format!("failed to read {}", decompiled_c.display()))?;

    // Extract just the function body from decompiled C (skip file headers/comments)
    let fn_body = extract_function_body(&decompiled_content, fn_name)?;

    // Generate test harness
    let harness = format!(
        r#"// Auto-generated test harness for {fn_id}
// Function: {fn_name} at {fn_addr}
// Prototype: {prototype}

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>

// Stub implementations for common libc functions that Ghidra might reference
#define NULL ((void*)0)
#define EOF (-1)

void __assert_fail(const char *assertion, const char *file, unsigned int line, const char *function) {{
    fprintf(stderr, "assertion failed: %s in %s:%u %s\n", assertion, file, line, function);
    exit(1);
}}

void __builtin_trap() {{
    asm volatile("int3");
}}

// We use a simplified approach: the decompiled function is compiled as-is
// and called from a test driver that parses JSON inputs from stdin

{fn_body}

// Parse JSON array from stdin and call the function
// Format: {{"args": [1, 2, 3], "expected_ret": 42}}
int main(int argc, char *argv[]) {{
    // For now, just return 0 to indicate the binary runs
    // Full I/O capture is done via Frida in verify.rs
    return 0;
}}
"#,
        fn_id = fn_id,
        fn_name = fn_name,
        fn_addr = fn_addr,
        prototype = prototype,
        fn_body = fn_body
    );

    Ok(harness)
}

/// Extract the function body for a given function name from decompiled C code
fn extract_function_body(decompiled_c: &str, fn_name: &str) -> Result<String> {
    let lines: Vec<&str> = decompiled_c.lines().collect();
    let mut in_function = false;
    let mut brace_count = 0;
    let mut fn_lines: Vec<String> = Vec::new();

    for line in &lines {
        let trimmed = line.trim();

        // Look for function definition - the function name followed by '(' on the same line
        // This handles return types like "int", "void *", etc.
        if !in_function && trimmed.contains(fn_name) && trimmed.contains('(') {
            // Skip forward declarations (lines ending with semicolon)
            if trimmed.ends_with(';') {
                continue;
            }
            in_function = true;
            brace_count = 0;
        }

        if in_function {
            fn_lines.push(line.to_string());
            brace_count += trimmed.matches('{').count() as i32;
            brace_count -= trimmed.matches('}').count() as i32;

            if brace_count <= 0 && fn_lines.len() > 1 {
                break;
            }
        }
    }

    if fn_lines.is_empty() {
        anyhow::bail!("could not find function body for {}", fn_name);
    }

    Ok(fn_lines.join("\n"))
}

/// Compile the test harness C code
pub fn compile_test_harness(
    harness_c: &Path,
    output_path: &Path,
    include_dirs: &[&Path],
) -> Result<()> {
    let mut cmd = Command::new("gcc");
    cmd.arg("-o")
        .arg(output_path)
        .arg(harness_c)
        .arg("-g") // debug info
        .arg("-O0"); // no optimization for debugging

    // Add include paths
    for dir in include_dirs {
        cmd.arg("-I").arg(dir);
    }

    // Suppress warnings for auto-generated code
    cmd.arg("-w");

    let output = cmd.output().with_context(|| "failed to execute gcc")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("gcc compilation failed:\n{}", stderr);
    }

    Ok(())
}

/// Build the complete rebuild pipeline: generate harness, compile, return binary path
pub fn build_reconstructed_binary(workspace: &Path, target: &str, fn_id: &str) -> Result<PathBuf> {
    let artifact_base = workspace.join("artifacts").join(target);
    let fn_dir = artifact_base
        .join("decompilation")
        .join("functions")
        .join(fn_id);

    let decompiled_c = fn_dir.join("decompiled.c");
    let meta_yaml = fn_dir.join("decompilation-record.yaml");

    if !decompiled_c.exists() {
        anyhow::bail!("decompiled.c not found for {}", fn_id);
    }
    if !meta_yaml.exists() {
        anyhow::bail!("decompilation-record.yaml not found for {}", fn_id);
    }

    // Parse metadata
    let meta = crate::schema::load_yaml::<serde_yaml::Value>(&meta_yaml)?;
    let fn_addr = meta.get("addr").and_then(|v| v.as_str()).unwrap_or("0x0");
    let fn_name = meta.get("name").and_then(|v| v.as_str()).unwrap_or(fn_id);
    let prototype = meta
        .get("prototype")
        .and_then(|v| v.as_str())
        .unwrap_or("void *");

    // Generate harness
    let harness = generate_test_harness(&decompiled_c, fn_id, fn_addr, fn_name, prototype)?;

    let build_dir = fn_dir.join("build");
    std::fs::create_dir_all(&build_dir)?;

    let harness_path = build_dir.join("harness.c");
    std::fs::write(&harness_path, &harness)
        .with_context(|| format!("failed to write {}", harness_path.display()))?;

    let binary_path = build_dir.join(format!("{}_test", fn_id));

    // Compile
    compile_test_harness(&harness_path, &binary_path, &[])?;

    Ok(binary_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_function_body() {
        let code = "int my_func(int a, int b) {\n    return a + b;\n}";

        let body = extract_function_body(code, "my_func").unwrap();
        assert!(body.contains("int my_func"));
        assert!(body.contains("return a + b"));
    }
}
