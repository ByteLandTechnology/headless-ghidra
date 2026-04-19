use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn cli() -> Command {
    Command::cargo_bin("ghidra-agent-cli").unwrap()
}

// ---------------------------------------------------------------------------
// Top-level: no args renders help
// ---------------------------------------------------------------------------
#[test]
fn no_args_shows_help() {
    cli()
        .assert()
        .success()
        .stdout(predicate::str::contains("GHIDRA-AGENT-CLI(1)"));
}

#[test]
fn help_flag_shows_help() {
    cli()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("GHIDRA-AGENT-CLI(1)"));
}

#[test]
fn help_subcommand_shows_help() {
    cli().arg("help").assert().success();
}

// ---------------------------------------------------------------------------
// Output format flags
// ---------------------------------------------------------------------------
#[test]
fn json_format_output() {
    let tmp = TempDir::new().unwrap();
    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "test-target"])
        .arg("--format")
        .arg("json")
        .arg("scope")
        .arg("set")
        .args(["--mode", "full", "--entries", "0x1000"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"status\": \"ok\""));
}

#[test]
fn toml_format_output() {
    let tmp = TempDir::new().unwrap();
    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "test-target"])
        .arg("--format")
        .arg("toml")
        .arg("scope")
        .arg("set")
        .args(["--mode", "full", "--entries", "0x1000"])
        .assert()
        .success()
        .stdout(predicate::str::contains("status = \"ok\""));
}

// ---------------------------------------------------------------------------
// Workspace init
// ---------------------------------------------------------------------------
#[test]
fn workspace_init_creates_structure() {
    let tmp = TempDir::new().unwrap();
    let binary = tmp.path().join("dummy.bin");
    std::fs::write(&binary, b"\x7fELF").unwrap();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .arg("workspace")
        .arg("init")
        .args(["--target", "libtest"])
        .args(["--binary", binary.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("target 'libtest' initialized"));

    // Verify directory structure was created
    let td = tmp.path().join("targets").join("libtest");
    let ad = tmp.path().join("artifacts").join("libtest");
    // targets/ holds ghidra-projects/ (gitignored)
    assert!(td.join("ghidra-projects").exists());
    // artifacts/ holds all analysis output
    assert!(ad.join("intake").exists());
    assert!(ad.join("baseline").exists());
    assert!(ad.join("scope.yaml").exists());
    assert!(ad.join("pipeline-state.yaml").exists());
}

// ---------------------------------------------------------------------------
// Scope management
// ---------------------------------------------------------------------------
#[test]
fn scope_set_and_show() {
    let tmp = TempDir::new().unwrap();
    let binary = tmp.path().join("dummy.bin");
    std::fs::write(&binary, b"\x7fELF").unwrap();

    // Init
    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .arg("workspace")
        .arg("init")
        .args(["--target", "libtest"])
        .args(["--binary", binary.to_str().unwrap()])
        .assert()
        .success();

    // Set scope
    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .arg("scope")
        .arg("set")
        .args(["--mode", "full", "--entries", "0x1000,0x2000"])
        .assert()
        .success();

    // Show scope
    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .arg("scope")
        .arg("show")
        .assert()
        .success()
        .stdout(predicate::str::contains("0x1000"));
}

#[test]
fn scope_add_and_remove_entry() {
    let tmp = TempDir::new().unwrap();
    let binary = tmp.path().join("dummy.bin");
    std::fs::write(&binary, b"\x7fELF").unwrap();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .arg("workspace")
        .arg("init")
        .args(["--target", "libtest"])
        .args(["--binary", binary.to_str().unwrap()])
        .assert()
        .success();

    // Add entry
    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .arg("scope")
        .arg("add-entry")
        .args(["--entry", "0x3000"])
        .assert()
        .success();

    // Remove entry
    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .arg("scope")
        .arg("remove-entry")
        .args(["--entry", "0x3000"])
        .assert()
        .success();
}

// ---------------------------------------------------------------------------
// Workspace state
// ---------------------------------------------------------------------------
#[test]
fn workspace_state_show() {
    let tmp = TempDir::new().unwrap();
    let binary = tmp.path().join("dummy.bin");
    std::fs::write(&binary, b"\x7fELF").unwrap();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .arg("workspace")
        .arg("init")
        .args(["--target", "libtest"])
        .args(["--binary", binary.to_str().unwrap()])
        .assert()
        .success();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .arg("workspace")
        .arg("state")
        .arg("show")
        .assert()
        .success()
        .stdout(predicate::str::contains("pipeline state"));
}

#[test]
fn workspace_state_set_phase() {
    let tmp = TempDir::new().unwrap();
    let binary = tmp.path().join("dummy.bin");
    std::fs::write(&binary, b"\x7fELF").unwrap();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .arg("workspace")
        .arg("init")
        .args(["--target", "libtest"])
        .args(["--binary", binary.to_str().unwrap()])
        .assert()
        .success();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .arg("workspace")
        .arg("state")
        .arg("set-phase")
        .args(["--phase", "P1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("phase set to P1"));
}

// ---------------------------------------------------------------------------
// Functions baseline
// ---------------------------------------------------------------------------
#[test]
fn functions_add_and_list() {
    let tmp = TempDir::new().unwrap();
    let binary = tmp.path().join("dummy.bin");
    std::fs::write(&binary, b"\x7fELF").unwrap();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .arg("workspace")
        .arg("init")
        .args(["--target", "libtest"])
        .args(["--binary", binary.to_str().unwrap()])
        .assert()
        .success();

    // Add function
    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .arg("functions")
        .arg("add")
        .args(["--addr", "0x1000"])
        .args(["--name", "main"])
        .args(["--size", "256"])
        .assert()
        .success()
        .stdout(predicate::str::contains("function added at 0x1000"));

    // List functions
    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .arg("functions")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("main"));
}

#[test]
fn functions_rename() {
    let tmp = TempDir::new().unwrap();
    let binary = tmp.path().join("dummy.bin");
    std::fs::write(&binary, b"\x7fELF").unwrap();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .arg("workspace")
        .arg("init")
        .args(["--target", "libtest"])
        .args(["--binary", binary.to_str().unwrap()])
        .assert()
        .success();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .arg("functions")
        .arg("add")
        .args(["--addr", "0x1000", "--name", "old_name"])
        .assert()
        .success();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .arg("functions")
        .arg("rename")
        .args(["--addr", "0x1000", "--new-name", "new_name"])
        .assert()
        .success()
        .stdout(predicate::str::contains("renamed to new_name"));
}

#[test]
fn functions_show() {
    let tmp = TempDir::new().unwrap();
    let binary = tmp.path().join("dummy.bin");
    std::fs::write(&binary, b"\x7fELF").unwrap();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .arg("workspace")
        .arg("init")
        .args(["--target", "libtest"])
        .args(["--binary", binary.to_str().unwrap()])
        .assert()
        .success();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .arg("functions")
        .arg("add")
        .args(["--addr", "0x1000", "--name", "test_func"])
        .assert()
        .success();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .arg("functions")
        .arg("show")
        .args(["--addr", "0x1000"])
        .assert()
        .success()
        .stdout(predicate::str::contains("test_func"));
}

#[test]
fn functions_remove() {
    let tmp = TempDir::new().unwrap();
    let binary = tmp.path().join("dummy.bin");
    std::fs::write(&binary, b"\x7fELF").unwrap();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .arg("workspace")
        .arg("init")
        .args(["--target", "libtest"])
        .args(["--binary", binary.to_str().unwrap()])
        .assert()
        .success();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .arg("functions")
        .arg("add")
        .args(["--addr", "0x1000"])
        .assert()
        .success();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .arg("functions")
        .arg("remove")
        .args(["--addr", "0x1000"])
        .assert()
        .success()
        .stdout(predicate::str::contains("removed"));
}

// ---------------------------------------------------------------------------
// Callgraph
// ---------------------------------------------------------------------------
#[test]
fn callgraph_add_remove_list() {
    let tmp = TempDir::new().unwrap();
    let binary = tmp.path().join("dummy.bin");
    std::fs::write(&binary, b"\x7fELF").unwrap();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .arg("workspace")
        .arg("init")
        .args(["--target", "libtest"])
        .args(["--binary", binary.to_str().unwrap()])
        .assert()
        .success();

    // Add edge
    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .arg("callgraph")
        .arg("add-edge")
        .args(["--from", "0x1000", "--to", "0x2000"])
        .assert()
        .success();

    // List
    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .arg("callgraph")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("0x1000"));

    // Remove edge
    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .arg("callgraph")
        .arg("remove-edge")
        .args(["--from", "0x1000", "--to", "0x2000"])
        .assert()
        .success();
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------
#[test]
fn types_add_remove_list() {
    let tmp = TempDir::new().unwrap();
    let binary = tmp.path().join("dummy.bin");
    std::fs::write(&binary, b"\x7fELF").unwrap();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .arg("workspace")
        .arg("init")
        .args(["--target", "libtest"])
        .args(["--binary", binary.to_str().unwrap()])
        .assert()
        .success();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .arg("types")
        .arg("add")
        .args([
            "--name",
            "MyStruct",
            "--kind",
            "struct",
            "--definition",
            "{ int x; }",
        ])
        .assert()
        .success();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .arg("types")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("MyStruct"));

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .arg("types")
        .arg("remove")
        .args(["--name", "MyStruct"])
        .assert()
        .success();
}

// ---------------------------------------------------------------------------
// Strings
// ---------------------------------------------------------------------------
#[test]
fn strings_add_and_list() {
    let tmp = TempDir::new().unwrap();
    let binary = tmp.path().join("dummy.bin");
    std::fs::write(&binary, b"\x7fELF").unwrap();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .arg("workspace")
        .arg("init")
        .args(["--target", "libtest"])
        .args(["--binary", binary.to_str().unwrap()])
        .assert()
        .success();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .arg("strings")
        .arg("add")
        .args(["--addr", "0x5000", "--content", "hello world"])
        .assert()
        .success();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .arg("strings")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("hello world"));
}

// ---------------------------------------------------------------------------
// Gate checks
// ---------------------------------------------------------------------------
#[test]
fn gate_check_p0_passes_after_init() {
    let tmp = TempDir::new().unwrap();
    let binary = tmp.path().join("dummy.bin");
    std::fs::write(&binary, b"\x7fELF").unwrap();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .arg("workspace")
        .arg("init")
        .args(["--target", "libtest"])
        .args(["--binary", binary.to_str().unwrap()])
        .assert()
        .success();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .arg("gate")
        .arg("check")
        .args(["--phase", "P0"])
        .assert()
        .success()
        .stdout(predicate::str::contains("gate check passed"));
}

// ---------------------------------------------------------------------------
// Execution log
// ---------------------------------------------------------------------------
#[test]
fn execution_log_append_and_list() {
    let tmp = TempDir::new().unwrap();
    let binary = tmp.path().join("dummy.bin");
    std::fs::write(&binary, b"\x7fELF").unwrap();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .arg("workspace")
        .arg("init")
        .args(["--target", "libtest"])
        .args(["--binary", binary.to_str().unwrap()])
        .assert()
        .success();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .arg("execution-log")
        .arg("append")
        .args(["--script", "test.java", "--status", "ok"])
        .assert()
        .success();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .arg("execution-log")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("test.java"));
}

// ---------------------------------------------------------------------------
// Progress
// ---------------------------------------------------------------------------
#[test]
fn progress_mark_and_list() {
    let tmp = TempDir::new().unwrap();
    let binary = tmp.path().join("dummy.bin");
    std::fs::write(&binary, b"\x7fELF").unwrap();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .arg("workspace")
        .arg("init")
        .args(["--target", "libtest"])
        .args(["--binary", binary.to_str().unwrap()])
        .assert()
        .success();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .arg("progress")
        .arg("mark-decompiled")
        .args(["--fn-id", "fn_001", "--addr", "0x1000"])
        .assert()
        .success();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .arg("progress")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("fn_001"));
}

// ---------------------------------------------------------------------------
// Context
// ---------------------------------------------------------------------------
#[test]
fn context_show() {
    cli()
        .arg("context")
        .arg("show")
        .assert()
        .success()
        .stdout(predicate::str::contains("context"));
}

#[test]
fn context_clear() {
    cli()
        .arg("context")
        .arg("clear")
        .assert()
        .success()
        .stdout(predicate::str::contains("cleared"));
}

// ---------------------------------------------------------------------------
// Paths
// ---------------------------------------------------------------------------
#[test]
fn paths_command() {
    cli()
        .arg("paths")
        .assert()
        .success()
        .stdout(predicate::str::contains("runtime paths"));
}

// ---------------------------------------------------------------------------
// Error: missing target
// ---------------------------------------------------------------------------
#[test]
fn missing_target_error() {
    let tmp = TempDir::new().unwrap();
    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .arg("scope")
        .arg("show")
        .assert()
        .failure();
}
