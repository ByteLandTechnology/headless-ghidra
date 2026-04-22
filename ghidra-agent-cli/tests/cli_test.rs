use assert_cmd::Command;
use predicates::prelude::*;
use std::process::Command as ProcessCommand;
use tempfile::TempDir;

fn cli() -> Command {
    Command::cargo_bin("ghidra-agent-cli").unwrap()
}

fn init_workspace(tmp: &TempDir, target: &str) {
    let binary = tmp.path().join("dummy.bin");
    std::fs::write(&binary, b"\x7fELF").unwrap();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .arg("workspace")
        .arg("init")
        .args(["--target", target])
        .args(["--binary", binary.to_str().unwrap()])
        .assert()
        .success();
}

#[cfg(unix)]
fn install_fake_ghidra(tmp: &TempDir) -> std::path::PathBuf {
    use std::os::unix::fs::PermissionsExt;

    let ghidra_dir = tmp.path().join("fake-ghidra");
    let support_dir = ghidra_dir.join("support");
    std::fs::create_dir_all(&support_dir).unwrap();

    let script_path = support_dir.join("analyzeHeadless");
    std::fs::write(
        &script_path,
        r#"#!/bin/sh
set -eu

script=""
logical_script=""
collect_args=0
workspace=""
target=""
addr=""
fn_id=""
prev=""

for arg in "$@"; do
  if [ "$prev" = "-postScript" ]; then
    script="$arg"
    prev=""
    collect_args=1
    continue
  fi
  if [ "$arg" = "-postScript" ]; then
    prev="-postScript"
    continue
  fi
  if [ "$arg" = "-scriptPath" ]; then
    collect_args=0
    break
  fi
  if [ "$collect_args" = "1" ]; then
    if [ "$script" = "GhidraAgentCliEntry.java" ] && [ -z "$logical_script" ]; then
      logical_script="$arg"
    elif [ -z "$workspace" ]; then
      workspace="$arg"
    elif [ -z "$target" ]; then
      target="$arg"
    elif [ -z "$addr" ]; then
      addr="$arg"
    elif [ -z "$fn_id" ]; then
      fn_id="$arg"
    fi
  fi
done

if [ -n "$logical_script" ]; then
  script="$logical_script"
fi

if [ "$script" = "DecompileFunction.java" ]; then
  if [ "$addr" = "0x2000" ]; then
    echo "simulated decompile failure for $fn_id" >&2
    exit 1
  fi

  out_dir="$workspace/artifacts/$target/decompilation/functions/$fn_id"
  mkdir -p "$out_dir"
  printf '/* decompiled %s at %s */\n' "$fn_id" "$addr" > "$out_dir/$fn_id.c"
  cat > "$out_dir/decompilation-record.yaml" <<EOF
fn_id: $fn_id
addr: "$addr"
name: test_$fn_id
prototype: int(void)
timestamp: 2026-04-21T00:00:00Z
EOF
fi
"#,
    )
    .unwrap();

    let mut perms = std::fs::metadata(&script_path).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&script_path, perms).unwrap();

    ghidra_dir
}

fn install_fake_bundled_entry(tmp: &TempDir) -> std::path::PathBuf {
    let bundle_dir = tmp.path().join("ghidra-script-bundle");
    std::fs::create_dir_all(&bundle_dir).unwrap();
    std::fs::write(bundle_dir.join("GhidraAgentCliEntry.java"), b"// fake").unwrap();
    std::fs::write(
        bundle_dir.join("ghidra-agent-cli-ghidra-scripts.jar"),
        b"fake-jar",
    )
    .unwrap();
    std::fs::write(bundle_dir.join("snakeyaml-2.6.jar"), b"fake-dependency").unwrap();
    bundle_dir
}

fn java_available(cmd: &str) -> bool {
    ProcessCommand::new(cmd)
        .arg("-version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[test]
fn yaml_parser_preserves_numeric_hex_and_normalizes_back_to_hex() {
    if !java_available("javac") || !java_available("java") {
        eprintln!("skipping java smoke test because javac/java is unavailable");
        return;
    }

    let tmp = TempDir::new().unwrap();
    let yaml_path = tmp.path().join("functions.yaml");
    std::fs::write(
        &yaml_path,
        r#"target: smoke
functions:
  - addr: 0x2000
    name: target_func
"#,
    )
    .unwrap();

    let smoke_path = tmp.path().join("YamlAddressSmoke.java");
    std::fs::write(
        &smoke_path,
        r#"import java.nio.file.Path;
import java.util.List;

public final class YamlAddressSmoke {
    public static void main(String[] args) throws Exception {
        List<YamlParsers.FunctionEntry> functions = YamlParsers.loadFunctions(Path.of(args[0]));
        YamlParsers.FunctionEntry entry = functions.get(0);
        Object raw = entry.getAddrValue();
        System.out.println("isNumber=" + (raw instanceof Number));
        System.out.println("raw=" + raw);
        System.out.println("normalized=" + AddressStrings.normalize(raw));
    }
}
"#,
    )
    .unwrap();

    let scripts_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ghidra-scripts");
    let snakeyaml_jar = scripts_dir.join("lib").join("snakeyaml-2.6.jar");
    let classes_dir = tmp.path().join("classes");
    std::fs::create_dir_all(&classes_dir).unwrap();

    let javac_output = ProcessCommand::new("javac")
        .arg("-cp")
        .arg(snakeyaml_jar.to_str().unwrap())
        .arg("-d")
        .arg(classes_dir.to_str().unwrap())
        .arg(scripts_dir.join("YamlParsers.java").to_str().unwrap())
        .arg(scripts_dir.join("AddressStrings.java").to_str().unwrap())
        .arg(smoke_path.to_str().unwrap())
        .output()
        .unwrap();
    assert!(
        javac_output.status.success(),
        "javac failed: {}",
        String::from_utf8_lossy(&javac_output.stderr)
    );

    let classpath = format!(
        "{}:{}",
        classes_dir.to_str().unwrap(),
        snakeyaml_jar.to_str().unwrap()
    );
    let java_output = ProcessCommand::new("java")
        .arg("-cp")
        .arg(&classpath)
        .arg("YamlAddressSmoke")
        .arg(yaml_path.to_str().unwrap())
        .output()
        .unwrap();
    assert!(
        java_output.status.success(),
        "java failed: {}",
        String::from_utf8_lossy(&java_output.stderr)
    );

    let stdout = String::from_utf8_lossy(&java_output.stdout);
    assert!(stdout.contains("isNumber=true"), "stdout was: {stdout}");
    assert!(stdout.contains("raw=8192"), "stdout was: {stdout}");
    assert!(stdout.contains("normalized=2000"), "stdout was: {stdout}");
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
    init_workspace(&tmp, "libtest");

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

#[test]
fn ghidra_decompile_batch_requires_next_batch_file_before_ghidra_lookup() {
    let tmp = TempDir::new().unwrap();
    init_workspace(&tmp, "libtest");

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .arg("ghidra")
        .arg("decompile")
        .arg("--batch")
        .assert()
        .failure()
        .stderr(predicate::str::contains("missing next batch file"));
}

#[test]
fn ghidra_decompile_batch_rejects_single_function_flags() {
    let tmp = TempDir::new().unwrap();
    init_workspace(&tmp, "libtest");

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .arg("ghidra")
        .arg("decompile")
        .args(["--batch", "--fn-id", "fn_001", "--addr", "0x1000"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--batch cannot be combined with --fn-id or --addr",
        ));
}

#[test]
fn ghidra_decompile_batch_rejects_empty_next_batch() {
    let tmp = TempDir::new().unwrap();
    init_workspace(&tmp, "libtest");

    let next_batch = tmp
        .path()
        .join("artifacts")
        .join("libtest")
        .join("decompilation")
        .join("next-batch.yaml");
    std::fs::write(
        next_batch,
        "target: libtest\nstrategy: breadth-first\nbatch: []\n",
    )
    .unwrap();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .arg("ghidra")
        .arg("decompile")
        .arg("--batch")
        .assert()
        .failure()
        .stderr(predicate::str::contains("contains no batch entries"));
}

#[test]
fn ghidra_decompile_batch_rejects_malformed_next_batch() {
    let tmp = TempDir::new().unwrap();
    init_workspace(&tmp, "libtest");

    let next_batch = tmp
        .path()
        .join("artifacts")
        .join("libtest")
        .join("decompilation")
        .join("next-batch.yaml");
    std::fs::write(
        next_batch,
        "target: libtest\nstrategy: breadth-first\nbatch: [\n",
    )
    .unwrap();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .arg("ghidra")
        .arg("decompile")
        .arg("--batch")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid next batch file"));
}

#[cfg(unix)]
#[test]
fn ghidra_decompile_single_with_mock_ghidra_creates_artifacts() {
    let tmp = TempDir::new().unwrap();
    init_workspace(&tmp, "libtest");
    let fake_ghidra = install_fake_ghidra(&tmp);

    cli()
        .env("GHIDRA_INSTALL_DIR", &fake_ghidra)
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .arg("ghidra")
        .arg("decompile")
        .args(["--fn-id", "fn_001", "--addr", "0x1000"])
        .assert()
        .success()
        .stdout(predicate::str::contains("decompilation complete"));

    let fn_dir = tmp
        .path()
        .join("artifacts")
        .join("libtest")
        .join("decompilation")
        .join("functions")
        .join("fn_001");
    assert!(fn_dir.join("fn_001.c").exists());
    assert!(fn_dir.join("decompilation-record.yaml").exists());

    let progress_path = tmp
        .path()
        .join("artifacts")
        .join("libtest")
        .join("decompilation")
        .join("progress.yaml");
    assert!(!progress_path.exists());
}

#[cfg(unix)]
#[test]
fn ghidra_decompile_single_with_bundled_entry_keeps_cli_compatibility() {
    let tmp = TempDir::new().unwrap();
    init_workspace(&tmp, "libtest");
    let fake_ghidra = install_fake_ghidra(&tmp);
    let fake_bundle = install_fake_bundled_entry(&tmp);

    cli()
        .env("GHIDRA_INSTALL_DIR", &fake_ghidra)
        .env("GHIDRA_SCRIPTS_DIR", &fake_bundle)
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .arg("ghidra")
        .arg("decompile")
        .args(["--fn-id", "fn_001", "--addr", "0x1000"])
        .assert()
        .success()
        .stdout(predicate::str::contains("decompilation complete"));

    let fn_dir = tmp
        .path()
        .join("artifacts")
        .join("libtest")
        .join("decompilation")
        .join("functions")
        .join("fn_001");
    assert!(fn_dir.join("fn_001.c").exists());
    assert!(fn_dir.join("decompilation-record.yaml").exists());
}

#[cfg(unix)]
#[test]
fn ghidra_decompile_batch_partial_success_updates_progress() {
    let tmp = TempDir::new().unwrap();
    init_workspace(&tmp, "libtest");
    let fake_ghidra = install_fake_ghidra(&tmp);

    let next_batch = tmp
        .path()
        .join("artifacts")
        .join("libtest")
        .join("decompilation")
        .join("next-batch.yaml");
    std::fs::write(
        next_batch,
        r#"target: libtest
strategy: breadth-first
batch:
  - fn_id: fn_001
    addr: "0x1000"
    reason: selected
  - fn_id: fn_002
    addr: "0x2000"
    reason: selected
  - fn_id: fn_003
    addr: "0x3000"
    reason: selected
"#,
    )
    .unwrap();

    cli()
        .env("GHIDRA_INSTALL_DIR", &fake_ghidra)
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .arg("ghidra")
        .arg("decompile")
        .arg("--batch")
        .assert()
        .failure()
        .stdout(predicate::str::contains("requested: 3"))
        .stdout(predicate::str::contains("succeeded: 2"))
        .stdout(predicate::str::contains("failed: 1"))
        .stdout(predicate::str::contains("fn_id: fn_001"))
        .stdout(predicate::str::contains("fn_id: fn_002"))
        .stdout(predicate::str::contains("fn_id: fn_003"));

    let decompilation_dir = tmp
        .path()
        .join("artifacts")
        .join("libtest")
        .join("decompilation");
    assert!(
        decompilation_dir
            .join("functions")
            .join("fn_001")
            .join("fn_001.c")
            .exists()
    );
    assert!(!decompilation_dir.join("functions").join("fn_002").exists());
    assert!(
        decompilation_dir
            .join("functions")
            .join("fn_003")
            .join("fn_003.c")
            .exists()
    );

    let progress = std::fs::read_to_string(decompilation_dir.join("progress.yaml")).unwrap();
    assert!(progress.contains("fn_001"));
    assert!(progress.contains("0x1000"));
    assert!(!progress.contains("fn_002"));
    assert!(!progress.contains("0x2000"));
    assert!(progress.contains("fn_003"));
    assert!(progress.contains("0x3000"));
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
