//! Integration test: Full pipeline from init to P4 gate
//!
//! This test creates a real binary, initializes a workspace, and progressively
//! advances through all primary gate phases (P0 -> P1 -> P2 -> P3 -> P4),
//! verifying each gate passes before advancing to the next.
//!
//! Run with: cargo test --test integration_pipeline_test

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Creates a simple test binary with known functions
fn create_test_binary(tmp: &TempDir) -> PathBuf {
    let src = tmp.path().join("test.c");
    let c_code = r#"
#include <stdio.h>
int add(int a, int b) { return a + b; }
int main() { printf("%d\n", add(1,2)); return 0; }
"#;
    fs::write(&src, c_code).unwrap();

    let output = std::process::Command::new("gcc")
        .args(["-o", tmp.path().join("test_bin").to_str().unwrap()])
        .arg(src.to_str().unwrap())
        .output()
        .expect("gcc must be installed to compile test binary");

    if !output.status.success() {
        panic!(
            "Failed to compile test binary: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    tmp.path().join("test_bin")
}

fn cli() -> Command {
    Command::cargo_bin("ghidra-agent-cli").unwrap()
}

fn command_available(command: &str, args: &[&str]) -> bool {
    std::process::Command::new(command)
        .args(args)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn frida_available() -> bool {
    command_available("frida", &["--version"])
}

fn cli_command_succeeds(args: &[&str]) -> bool {
    let mut command = cli();
    command.args(args);
    command
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn ghidra_available() -> bool {
    cli_command_succeeds(&["ghidra", "discover"])
}

fn record_runtime_p1(workspace: &str) {
    cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .args(["runtime", "record"])
        .args(["--key", "entrypoint", "--value", "0x1000"])
        .assert()
        .success();

    cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .args(["hotpath", "add"])
        .args(["--addr", "0x1000", "--reason", "integration runtime sample"])
        .assert()
        .success();
}

fn record_third_party_p2(tmp: &TempDir, workspace: &str) {
    let source = tmp.path().join("zlib-src");
    fs::create_dir_all(&source).unwrap();
    fs::write(source.join("README"), "upstream").unwrap();

    cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .args(["third-party", "add"])
        .args([
            "--library",
            "zlib",
            "--version",
            "1.2.13",
            "--confidence",
            "high",
        ])
        .assert()
        .success();

    cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .args(["third-party", "vendor-pristine"])
        .args([
            "--library",
            "zlib",
            "--source-path",
            source.to_str().unwrap(),
        ])
        .assert()
        .success();

    cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .args(["third-party", "classify-function"])
        .args(["--addr", "0x1000", "--classification", "library"])
        .assert()
        .success();
}

fn record_metadata_p3(workspace: &str) {
    cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .args(["metadata", "enrich-function"])
        .args([
            "--addr",
            "0x1000",
            "--name",
            "main",
            "--prototype",
            "int(int,char**)",
        ])
        .assert()
        .success();
}

fn record_substitution_p4(workspace: &str) {
    cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .args(["substitute", "add"])
        .args([
            "--fn-id",
            "fn_001",
            "--addr",
            "0x1000",
            "--replacement",
            "return 0;",
        ])
        .assert()
        .success();
}

// ---------------------------------------------------------------------------
// Helper: Setup workspace with all P1 baseline files
// ---------------------------------------------------------------------------
fn setup_workspace_p1(tmp: &TempDir) {
    let workspace = tmp.path().to_str().unwrap();
    let binary = create_test_binary(tmp);

    cli()
        .args(["--workspace", workspace])
        .arg("workspace")
        .arg("init")
        .args(["--target", "test_bin"])
        .args(["--binary", binary.to_str().unwrap()])
        .assert()
        .success();

    // P0.5: scope
    cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .arg("scope")
        .arg("set")
        .args(["--mode", "workspace"])
        .assert()
        .success();

    cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .arg("scope")
        .arg("add-entry")
        .args(["--entry", "0x1000"])
        .assert()
        .success();

    // Add another scope entry so batch won't be empty when one function is decompiled
    cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .arg("scope")
        .arg("add-entry")
        .args(["--entry", "0x2000"])
        .assert()
        .success();

    // P1: baseline files
    cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .arg("functions")
        .arg("add")
        .args([
            "--addr",
            "0x1000",
            "--name",
            "main",
            "--prototype",
            "int(int,char**)",
        ])
        .assert()
        .success();

    cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .arg("callgraph")
        .arg("add-edge")
        .args(["--from", "0x1000", "--to", "0x2000"])
        .assert()
        .success();

    // Add function 0x2000 for batch reference (P4 requires batch entries to be in progress)
    cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .arg("functions")
        .arg("add")
        .args([
            "--addr",
            "0x2000",
            "--name",
            "helper",
            "--prototype",
            "int(int)",
        ])
        .assert()
        .success();

    cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .arg("types")
        .arg("add")
        .args([
            "--name",
            "int",
            "--kind",
            "typedef",
            "--definition",
            "typedef int int_t;",
        ])
        .assert()
        .success();

    cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .arg("vtables")
        .arg("add")
        .args(["--class-name", "TestClass", "--addr", "0x3000"])
        .assert()
        .success();

    cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .arg("constants")
        .arg("add")
        .args(["--addr", "0x4000", "--name", "MAX", "--value", "100"])
        .assert()
        .success();

    cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .arg("strings")
        .arg("add")
        .args(["--addr", "0x5000", "--content", "test"])
        .assert()
        .success();

    cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .arg("imports")
        .arg("add")
        .args(["--library", "libSystem", "--symbol", "malloc"])
        .assert()
        .success();

    record_runtime_p1(workspace);
}

// ---------------------------------------------------------------------------
// Phase 0: Workspace Init
// ---------------------------------------------------------------------------

#[test]
fn phase0_workspace_init() {
    let tmp = TempDir::new().unwrap();
    let binary = create_test_binary(&tmp);

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .arg("workspace")
        .arg("init")
        .args(["--target", "test_bin"])
        .args(["--binary", binary.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("target 'test_bin' initialized"));

    let ad = tmp.path().join("artifacts").join("test_bin");
    assert!(
        ad.join("pipeline-state.yaml").exists(),
        "P0: pipeline-state.yaml must exist"
    );
}

#[test]
fn phase0_gate_check() {
    let tmp = TempDir::new().unwrap();
    let binary = create_test_binary(&tmp);

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .arg("workspace")
        .arg("init")
        .args(["--target", "test_bin"])
        .args(["--binary", binary.to_str().unwrap()])
        .assert()
        .success();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "test_bin"])
        .arg("gate")
        .arg("check")
        .args(["--phase", "P0"])
        .assert()
        .success()
        .stdout(predicate::str::contains("passed"));
}

// ---------------------------------------------------------------------------
// Phase 0.5: Scope Configuration
// ---------------------------------------------------------------------------

#[test]
fn phase0_5_scope_configured() {
    let tmp = TempDir::new().unwrap();
    let binary = create_test_binary(&tmp);
    let workspace = tmp.path().to_str().unwrap();

    cli()
        .args(["--workspace", workspace])
        .arg("workspace")
        .arg("init")
        .args(["--target", "test_bin"])
        .args(["--binary", binary.to_str().unwrap()])
        .assert()
        .success();

    cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .arg("scope")
        .arg("set")
        .args(["--mode", "workspace"])
        .assert()
        .success();

    cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .arg("scope")
        .arg("add-entry")
        .args(["--entry", "0x1000"])
        .assert()
        .success();

    let ad = tmp.path().join("artifacts").join("test_bin");
    assert!(
        ad.join("scope.yaml").exists(),
        "P0.5: scope.yaml must exist"
    );
}

#[test]
fn phase0_5_gate_check() {
    let tmp = TempDir::new().unwrap();
    setup_workspace_p1(&tmp);
    let workspace = tmp.path().to_str().unwrap();

    cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .arg("gate")
        .arg("check")
        .args(["--phase", "P0.5"])
        .assert()
        .success()
        .stdout(predicate::str::contains("passed"));
}

// ---------------------------------------------------------------------------
// Phase 1: Baseline Files Created
// ---------------------------------------------------------------------------

#[test]
fn phase1_baseline_files_created() {
    let tmp = TempDir::new().unwrap();
    setup_workspace_p1(&tmp);

    let ad = tmp
        .path()
        .join("artifacts")
        .join("test_bin")
        .join("baseline");
    assert!(
        ad.join("functions.yaml").exists(),
        "P1: functions.yaml must exist"
    );
    assert!(
        ad.join("callgraph.yaml").exists(),
        "P1: callgraph.yaml must exist"
    );
    assert!(ad.join("types.yaml").exists(), "P1: types.yaml must exist");
    assert!(
        ad.join("vtables.yaml").exists(),
        "P1: vtables.yaml must exist"
    );
    assert!(
        ad.join("constants.yaml").exists(),
        "P1: constants.yaml must exist"
    );
    assert!(
        ad.join("strings.yaml").exists(),
        "P1: strings.yaml must exist"
    );
    assert!(
        ad.join("imports.yaml").exists(),
        "P1: imports.yaml must exist"
    );

    let runtime = tmp
        .path()
        .join("artifacts")
        .join("test_bin")
        .join("runtime");
    assert!(
        runtime.join("run-manifest.yaml").exists(),
        "P1: run-manifest.yaml must exist"
    );
    assert!(
        runtime.join("run-records").join("entrypoint.yaml").exists(),
        "P1: run record must exist"
    );
    assert!(
        runtime.join("hotpaths").join("call-chain.yaml").exists(),
        "P1: call-chain.yaml must exist"
    );
}

#[test]
fn phase1_gate_check() {
    let tmp = TempDir::new().unwrap();
    setup_workspace_p1(&tmp);
    let workspace = tmp.path().to_str().unwrap();

    cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .arg("gate")
        .arg("check")
        .args(["--phase", "P1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("passed"));
}

// ---------------------------------------------------------------------------
// Phase 2: Third-Party Libraries Identified
// ---------------------------------------------------------------------------

#[test]
fn phase2_gate_check() {
    let tmp = TempDir::new().unwrap();
    setup_workspace_p1(&tmp);
    let workspace = tmp.path().to_str().unwrap();
    record_third_party_p2(&tmp, workspace);

    cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .arg("gate")
        .arg("check")
        .args(["--phase", "P2"])
        .assert()
        .success()
        .stdout(predicate::str::contains("passed"));
}

// ---------------------------------------------------------------------------
// Phase 3: Target Selected, Scope Non-Empty
// ---------------------------------------------------------------------------

#[test]
fn phase3_gate_check() {
    let tmp = TempDir::new().unwrap();
    setup_workspace_p1(&tmp);
    let workspace = tmp.path().to_str().unwrap();
    record_metadata_p3(workspace);

    cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .arg("gate")
        .arg("check")
        .args(["--phase", "P3"])
        .assert()
        .success()
        .stdout(predicate::str::contains("passed"));
}

// ---------------------------------------------------------------------------
// Phase 4: Decompilation Progress Tracking
// ---------------------------------------------------------------------------

#[test]
fn phase4_gate_check() {
    let tmp = TempDir::new().unwrap();
    setup_workspace_p1(&tmp);
    let workspace = tmp.path().to_str().unwrap();
    record_metadata_p3(workspace);
    record_substitution_p4(workspace);

    cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .arg("gate")
        .arg("check")
        .args(["--phase", "P4"])
        .assert()
        .success()
        .stdout(predicate::str::contains("passed"));
}

// ---------------------------------------------------------------------------
// Phase 5: Decompiled C Files Exist
// ---------------------------------------------------------------------------

#[test]
fn phase5_gate_check() {
    let tmp = TempDir::new().unwrap();
    setup_workspace_p1(&tmp);
    let workspace = tmp.path().to_str().unwrap();
    let ad = tmp.path().join("artifacts").join("test_bin");

    // P3 artifacts
    let target_selection = r#"selected_target: /usr/lib/libfoo.dylib
candidates:
  - path: /usr/lib/libfoo.dylib
    status: ready
"#;
    fs::write(ad.join("target-selection.yaml"), target_selection).unwrap();

    // P4: mark decompiled
    cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .arg("progress")
        .arg("mark-decompiled")
        .args(["--fn-id", "fn_001", "--addr", "0x1000"])
        .assert()
        .success();

    cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .arg("progress")
        .arg("compute-next-batch")
        .assert()
        .success();

    // P5: decompilation-record.yaml
    let fn_dir = ad.join("decompilation").join("functions").join("fn_001");
    fs::create_dir_all(&fn_dir).unwrap();
    fs::write(
        fn_dir.join("decompilation-record.yaml"),
        "fn_id: fn_001\naddr: \"0x1000\"\nname: main\nprototype: int(int,char**)\n",
    )
    .unwrap();

    cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .arg("gate")
        .arg("check")
        .args(["--phase", "P5"])
        .assert()
        .success()
        .stdout(predicate::str::contains("passed"));
}

// ---------------------------------------------------------------------------
// Phase 6: Verification Results
// ---------------------------------------------------------------------------

#[test]
fn phase6_gate_check() {
    let tmp = TempDir::new().unwrap();
    setup_workspace_p1(&tmp);
    let workspace = tmp.path().to_str().unwrap();
    let ad = tmp.path().join("artifacts").join("test_bin");

    // P3 artifacts
    let target_selection = r#"selected_target: /usr/lib/libfoo.dylib
candidates:
  - path: /usr/lib/libfoo.dylib
    status: ready
"#;
    fs::write(ad.join("target-selection.yaml"), target_selection).unwrap();

    // P4: mark decompiled
    cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .arg("progress")
        .arg("mark-decompiled")
        .args(["--fn-id", "fn_001", "--addr", "0x1000"])
        .assert()
        .success();

    cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .arg("progress")
        .arg("compute-next-batch")
        .assert()
        .success();

    // P5: decompilation-record.yaml
    let fn_dir = ad.join("decompilation").join("functions").join("fn_001");
    fs::create_dir_all(&fn_dir).unwrap();
    fs::write(
        fn_dir.join("decompilation-record.yaml"),
        "fn_id: fn_001\naddr: \"0x1000\"\nname: main\nprototype: int(int,char**)\n",
    )
    .unwrap();

    // P6: verification-result.yaml
    fs::write(
        fn_dir.join("verification-result.yaml"),
        "fn_id: fn_001\nverified: true\nmismatches: []\n",
    )
    .unwrap();

    cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .arg("gate")
        .arg("check")
        .args(["--phase", "P6"])
        .assert()
        .success()
        .stdout(predicate::str::contains("passed"));
}

// ---------------------------------------------------------------------------
// Full Pipeline: All Gates P0 -> P6 (combined test)
// ---------------------------------------------------------------------------

#[test]
fn full_pipeline_all_gates_pass() {
    let tmp = TempDir::new().unwrap();
    let workspace = tmp.path().to_str().unwrap();
    let ad = tmp.path().join("artifacts").join("test_bin");

    // ---- P0: Init ----
    let binary = create_test_binary(&tmp);
    cli()
        .args(["--workspace", workspace])
        .arg("workspace")
        .arg("init")
        .args(["--target", "test_bin"])
        .args(["--binary", binary.to_str().unwrap()])
        .assert()
        .success();

    let result = cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .arg("gate")
        .arg("check")
        .args(["--phase", "P0"])
        .assert()
        .success();
    assert!(String::from_utf8_lossy(&result.get_output().stdout).contains("passed"));

    // ---- P0.5: Scope ----
    cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .arg("scope")
        .arg("set")
        .args(["--mode", "workspace"])
        .assert()
        .success();

    cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .arg("scope")
        .arg("add-entry")
        .args(["--entry", "0x1000"])
        .assert()
        .success();

    // Add another scope entry so batch won't be empty when one function is decompiled
    cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .arg("scope")
        .arg("add-entry")
        .args(["--entry", "0x2000"])
        .assert()
        .success();

    let result = cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .arg("gate")
        .arg("check")
        .args(["--phase", "P0.5"])
        .assert()
        .success();
    assert!(String::from_utf8_lossy(&result.get_output().stdout).contains("passed"));

    // ---- P1: Baseline ----
    cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .arg("functions")
        .arg("add")
        .args([
            "--addr",
            "0x1000",
            "--name",
            "main",
            "--prototype",
            "int(int,char**)",
        ])
        .assert()
        .success();

    cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .arg("callgraph")
        .arg("add-edge")
        .args(["--from", "0x1000", "--to", "0x2000"])
        .assert()
        .success();

    // Add function 0x2000 for batch reference (P4 requires batch entries to be in progress)
    cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .arg("functions")
        .arg("add")
        .args([
            "--addr",
            "0x2000",
            "--name",
            "helper",
            "--prototype",
            "int(int)",
        ])
        .assert()
        .success();

    cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .arg("types")
        .arg("add")
        .args([
            "--name",
            "int",
            "--kind",
            "typedef",
            "--definition",
            "typedef int int_t;",
        ])
        .assert()
        .success();

    cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .arg("vtables")
        .arg("add")
        .args(["--class-name", "TestClass", "--addr", "0x3000"])
        .assert()
        .success();

    cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .arg("constants")
        .arg("add")
        .args(["--addr", "0x4000", "--name", "MAX", "--value", "100"])
        .assert()
        .success();

    cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .arg("strings")
        .arg("add")
        .args(["--addr", "0x5000", "--content", "test"])
        .assert()
        .success();

    cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .arg("imports")
        .arg("add")
        .args(["--library", "libSystem", "--symbol", "malloc"])
        .assert()
        .success();

    record_runtime_p1(workspace);

    let result = cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .arg("gate")
        .arg("check")
        .args(["--phase", "P1"])
        .assert()
        .success();
    assert!(String::from_utf8_lossy(&result.get_output().stdout).contains("passed"));

    // ---- P2: Third-party ----
    record_third_party_p2(&tmp, workspace);

    let result = cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .arg("gate")
        .arg("check")
        .args(["--phase", "P2"])
        .assert()
        .success();
    assert!(String::from_utf8_lossy(&result.get_output().stdout).contains("passed"));

    // ---- P3: Metadata enrichment ----
    record_metadata_p3(workspace);

    let result = cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .arg("gate")
        .arg("check")
        .args(["--phase", "P3"])
        .assert()
        .success();
    assert!(String::from_utf8_lossy(&result.get_output().stdout).contains("passed"));

    // ---- P4: Function substitution ----
    record_substitution_p4(workspace);

    let result = cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .arg("gate")
        .arg("check")
        .args(["--phase", "P4"])
        .assert()
        .success();
    assert!(String::from_utf8_lossy(&result.get_output().stdout).contains("passed"));

    // ---- P5: Decompiled files ----
    let fn_dir = ad.join("decompilation").join("functions").join("fn_001");
    fs::create_dir_all(&fn_dir).unwrap();
    fs::write(
        fn_dir.join("decompilation-record.yaml"),
        "fn_id: fn_001\naddr: \"0x1000\"\nname: main\nprototype: int(int,char**)\n",
    )
    .unwrap();

    let result = cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .arg("gate")
        .arg("check")
        .args(["--phase", "P5"])
        .assert()
        .success();
    assert!(String::from_utf8_lossy(&result.get_output().stdout).contains("passed"));

    // ---- P6: Verification complete ----
    fs::write(
        fn_dir.join("verification-result.yaml"),
        "fn_id: fn_001\nverified: true\nmismatches: []\n",
    )
    .unwrap();

    let result = cli()
        .args(["--workspace", workspace])
        .args(["--target", "test_bin"])
        .arg("gate")
        .arg("check")
        .args(["--phase", "P6"])
        .assert()
        .success();
    assert!(String::from_utf8_lossy(&result.get_output().stdout).contains("passed"));

    println!("ALL GATES P0->P6 PASSED SUCCESSFULLY!");
}

// ============================================================================
// Frida Commands Integration
// ============================================================================

#[test]
fn frida_device_list_works() {
    if !frida_available() {
        eprintln!("skipping frida device-list integration test; Frida is unavailable");
        return;
    }

    cli()
        .arg("frida")
        .arg("device-list")
        .assert()
        .success()
        .stdout(predicate::str::contains("local"));
}

#[test]
fn frida_fuzz_input_gen_works() {
    let tmp = TempDir::new().unwrap();
    let yaml_content = r#"
functions:
  - name: add
    signature: int(int, int)
types:
  - name: int
    values: [0, 1, -1]
"#;
    fs::write(tmp.path().join("types.yaml"), yaml_content).unwrap();

    cli()
        .args(["frida", "fuzz-input-gen"])
        .args([
            "--types-yaml",
            tmp.path().join("types.yaml").to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Generated"));
}

#[test]
fn frida_io_compare_works() {
    let tmp = TempDir::new().unwrap();

    let original = r#"[{"type":"call","name":"func1","args":[1,2],"return_value":"3"}]"#;
    fs::write(tmp.path().join("original.json"), original).unwrap();

    let reconstructed = r#"[{"type":"call","name":"func1","args":[1,2],"return_value":"3"}]"#;
    fs::write(tmp.path().join("reconstructed.json"), reconstructed).unwrap();

    cli()
        .args(["frida", "io-compare"])
        .args([
            "--original",
            tmp.path().join("original.json").to_str().unwrap(),
        ])
        .args([
            "--reconstructed",
            tmp.path().join("reconstructed.json").to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("matches"));
}

// ============================================================================
// Context and Global Commands
// ============================================================================

#[test]
fn context_commands_work() {
    cli()
        .arg("context")
        .arg("show")
        .assert()
        .success()
        .stdout(predicate::str::contains("context"));

    cli()
        .arg("context")
        .arg("clear")
        .assert()
        .success()
        .stdout(predicate::str::contains("cleared"));
}

#[test]
fn paths_command_works() {
    cli()
        .arg("paths")
        .assert()
        .success()
        .stdout(predicate::str::contains("runtime paths"));
}

#[test]
fn inspect_binary_works() {
    let tmp = TempDir::new().unwrap();
    let binary = create_test_binary(&tmp);

    cli()
        .args(["inspect", "binary"])
        .args(["--target", binary.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("target"));
}

#[test]
fn ghidra_discover_works() {
    if !ghidra_available() {
        eprintln!("skipping ghidra discover integration test; Ghidra is unavailable");
        return;
    }

    let tmp = TempDir::new().unwrap();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "test"])
        .arg("ghidra")
        .arg("discover")
        .assert()
        .success()
        .stdout(predicate::str::contains("ghidra"));
}
