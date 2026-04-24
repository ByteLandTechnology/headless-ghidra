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

fn artifact_dir(tmp: &TempDir, target: &str) -> std::path::PathBuf {
    tmp.path().join("artifacts").join(target)
}

fn add_minimal_p1_baseline(tmp: &TempDir, target: &str) {
    let workspace = tmp.path().to_str().unwrap();

    cli()
        .args(["--workspace", workspace, "--target", target])
        .args(["functions", "add"])
        .args(["--addr", "0x1000", "--name", "main"])
        .assert()
        .success();
    cli()
        .args(["--workspace", workspace, "--target", target])
        .args(["callgraph", "add-edge"])
        .args(["--from", "0x1000", "--to", "0x1000"])
        .assert()
        .success();
    cli()
        .args(["--workspace", workspace, "--target", target])
        .args(["types", "add"])
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
        .args(["--workspace", workspace, "--target", target])
        .args(["vtables", "add"])
        .args(["--class-name", "TestClass", "--addr", "0x2000"])
        .assert()
        .success();
    cli()
        .args(["--workspace", workspace, "--target", target])
        .args(["constants", "add"])
        .args(["--addr", "0x3000", "--name", "MAX"])
        .assert()
        .success();
    cli()
        .args(["--workspace", workspace, "--target", target])
        .args(["strings", "add"])
        .args(["--addr", "0x4000", "--content", "hello"])
        .assert()
        .success();
    cli()
        .args(["--workspace", workspace, "--target", target])
        .args(["imports", "add"])
        .args(["--library", "libc", "--symbol", "malloc"])
        .assert()
        .success();
}

fn add_runtime_record(tmp: &TempDir, target: &str, key: &str, value: &str) {
    cli()
        .args([
            "--workspace",
            tmp.path().to_str().unwrap(),
            "--target",
            target,
        ])
        .args(["runtime", "record"])
        .args(["--key", key, "--value", value])
        .assert()
        .success();
}

fn add_hotpath(tmp: &TempDir, target: &str, addr: &str) {
    cli()
        .args([
            "--workspace",
            tmp.path().to_str().unwrap(),
            "--target",
            target,
        ])
        .args(["hotpath", "add"])
        .args(["--addr", addr, "--reason", "runtime sample"])
        .assert()
        .success();
}

fn add_metadata(tmp: &TempDir, target: &str, addr: &str) {
    cli()
        .args([
            "--workspace",
            tmp.path().to_str().unwrap(),
            "--target",
            target,
        ])
        .args(["metadata", "enrich-function"])
        .args(["--addr", addr, "--name", "main", "--prototype", "int(void)"])
        .assert()
        .success();
}

fn add_substitution(tmp: &TempDir, target: &str, fn_id: &str, addr: &str) {
    cli()
        .args([
            "--workspace",
            tmp.path().to_str().unwrap(),
            "--target",
            target,
        ])
        .args(["substitute", "add"])
        .args([
            "--fn-id",
            fn_id,
            "--addr",
            addr,
            "--replacement",
            "return 0;",
        ])
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
program=""
addr=""
fn_id=""
min_entries=""
max_entries=""
scan_limit=""
segments=""
min_score=""
write_baseline=""
overwrite=""
report_path=""
extra_args=""
prev=""

for arg in "$@"; do
  if [ "$prev" = "-process" ]; then
    program="$arg"
    prev=""
    continue
  fi
  if [ "$prev" = "-postScript" ]; then
    script="$arg"
    script="${script##*/}"
    prev=""
    collect_args=1
    continue
  fi
  if [ "$arg" = "-process" ]; then
    prev="-process"
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
    elif { [ "$script" = "DecompileFunction.java" ] || [ "$logical_script" = "DecompileFunction.java" ]; } && [ -z "$addr" ]; then
      addr="$arg"
    elif { [ "$script" = "DecompileFunction.java" ] || [ "$logical_script" = "DecompileFunction.java" ]; } && [ -z "$fn_id" ]; then
      fn_id="$arg"
    elif { [ "$script" = "AnalyzeVtables.java" ] || [ "$logical_script" = "AnalyzeVtables.java" ]; } && [ -z "$min_entries" ]; then
      min_entries="$arg"
    elif { [ "$script" = "AnalyzeVtables.java" ] || [ "$logical_script" = "AnalyzeVtables.java" ]; } && [ -z "$max_entries" ]; then
      max_entries="$arg"
    elif { [ "$script" = "AnalyzeVtables.java" ] || [ "$logical_script" = "AnalyzeVtables.java" ]; } && [ -z "$scan_limit" ]; then
      scan_limit="$arg"
    elif { [ "$script" = "AnalyzeVtables.java" ] || [ "$logical_script" = "AnalyzeVtables.java" ]; } && [ -z "$segments" ]; then
      segments="$arg"
    elif { [ "$script" = "AnalyzeVtables.java" ] || [ "$logical_script" = "AnalyzeVtables.java" ]; } && [ -z "$min_score" ]; then
      min_score="$arg"
    elif { [ "$script" = "AnalyzeVtables.java" ] || [ "$logical_script" = "AnalyzeVtables.java" ]; } && [ -z "$write_baseline" ]; then
      write_baseline="$arg"
    elif { [ "$script" = "AnalyzeVtables.java" ] || [ "$logical_script" = "AnalyzeVtables.java" ]; } && [ -z "$overwrite" ]; then
      overwrite="$arg"
    elif { [ "$script" = "AnalyzeVtables.java" ] || [ "$logical_script" = "AnalyzeVtables.java" ]; } && [ -z "$report_path" ]; then
      report_path="$arg"
    else
      if [ -z "$extra_args" ]; then
        extra_args="$arg"
      else
        extra_args="$extra_args|$arg"
      fi
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

if [ "$script" = "AnalyzeVtables.java" ]; then
  if [ -z "$report_path" ]; then
    report_path="$workspace/artifacts/$target/baseline/vtable-analysis-report.yaml"
  fi
  mkdir -p "$(dirname "$report_path")"
  cat > "$report_path" <<EOF
target: $target
pointer_size: 8
scan_segments:
  - ".rodata"
candidates:
  - addr: "0x3000"
    status: accepted
    class: "MockClass"
    segment: ".rodata"
    score: 8
    confidence: "high"
    entry_count: 4
    entries:
      - "0x1000"
      - "0x1010"
      - "0x1020"
      - "0x1030"
    reasons:
      - "entry_count_in_expected_range=true"
      - "first_entry_destructor_like=true"
    associated_type: "MockClass"
    association_evidence:
      - "symbol:vtable for MockClass"
    signature_summary: "void(MockClass*)"
rejected:
  - addr: "0x4000"
    status: rejected
    class: "jump_table_0x4000"
    segment: ".rodata"
    score: 1
    confidence: "low"
    entry_count: 2
    entries:
      - "0x2000"
      - "0x2010"
    reasons:
      - "score_below_threshold=$min_score"
EOF

  if [ "$write_baseline" = "true" ]; then
    baseline_path="$workspace/artifacts/$target/baseline/vtables.yaml"
    if [ -f "$baseline_path" ] && [ "$overwrite" != "true" ]; then
      echo "Refusing to overwrite existing baseline vtables without overwrite=true: $baseline_path" >&2
      exit 1
    fi
    mkdir -p "$(dirname "$baseline_path")"
    cat > "$baseline_path" <<EOF
target: $target
vtables:
  - class: "MockClass"
    addr: "0x3000"
    entries:
      - "0x1000"
      - "0x1010"
      - "0x1020"
      - "0x1030"
    entry_count: 4
    confidence: "high"
    score: 8
    source: "ghidra_auto"
    segment: ".rodata"
    associated_type: "MockClass"
    association_evidence:
      - "symbol:vtable for MockClass"
    signature_summary: "void(MockClass*)"
EOF
  fi
fi

if [ "$script" = "ApplySignatures.java" ]; then
  out_dir="$workspace/artifacts/$target/runtime"
  mkdir -p "$out_dir"
  cat > "$out_dir/apply-signatures-record.yaml" <<EOF
script: "$script"
program: "${program:-}"
extra_args: "${extra_args:-}"
EOF
fi

if [ "$script" = "ImportTypesAndSignatures.java" ]; then
  out_dir="$workspace/artifacts/$target/runtime"
  mkdir -p "$out_dir"
  headers=""
  include_dirs=""
  signatures=""
  prev_arg=""
  IFS='|' read -r -a extra_parts <<< "${extra_args:-}"
  for extra in "${extra_parts[@]}"; do
    if [ "$prev_arg" = "--header" ]; then
      if [ -z "$headers" ]; then
        headers="$extra"
      else
        headers="$headers|$extra"
      fi
      prev_arg=""
      continue
    fi
    if [ "$prev_arg" = "--include-dir" ]; then
      if [ -z "$include_dirs" ]; then
        include_dirs="$extra"
      else
        include_dirs="$include_dirs|$extra"
      fi
      prev_arg=""
      continue
    fi
    if [ "$prev_arg" = "--signatures" ]; then
      signatures="$extra"
      prev_arg=""
      continue
    fi
    if [ "$extra" = "--header" ] || [ "$extra" = "--include-dir" ] || [ "$extra" = "--signatures" ]; then
      prev_arg="$extra"
    fi
  done
  cat > "$out_dir/import-types-and-signatures-record.yaml" <<EOF
script: "$script"
program: "${program:-}"
headers: "${headers:-}"
include_dirs: "${include_dirs:-}"
signatures: "${signatures:-}"
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

#[test]
fn p0_p4_artifact_groups_validate_and_gate() {
    let tmp = TempDir::new().unwrap();
    init_workspace(&tmp, "libtest");
    let workspace = tmp.path().to_str().unwrap();

    cli()
        .args(["--workspace", workspace, "--target", "libtest"])
        .args(["functions", "add"])
        .args(["--addr", "0x1000", "--name", "main"])
        .assert()
        .success();
    cli()
        .args(["--workspace", workspace, "--target", "libtest"])
        .args(["callgraph", "add-edge"])
        .args(["--from", "0x1000", "--to", "0x1000"])
        .assert()
        .success();
    cli()
        .args(["--workspace", workspace, "--target", "libtest"])
        .args(["types", "add"])
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
        .args(["--workspace", workspace, "--target", "libtest"])
        .args(["vtables", "add"])
        .args(["--class-name", "TestClass", "--addr", "0x2000"])
        .assert()
        .success();
    cli()
        .args(["--workspace", workspace, "--target", "libtest"])
        .args(["constants", "add"])
        .args(["--addr", "0x3000", "--name", "MAX"])
        .assert()
        .success();
    cli()
        .args(["--workspace", workspace, "--target", "libtest"])
        .args(["strings", "add"])
        .args(["--addr", "0x4000", "--content", "hello"])
        .assert()
        .success();
    cli()
        .args(["--workspace", workspace, "--target", "libtest"])
        .args(["imports", "add"])
        .args(["--library", "libc", "--symbol", "malloc"])
        .assert()
        .success();
    cli()
        .args(["--workspace", workspace, "--target", "libtest"])
        .args(["runtime", "record"])
        .args(["--key", "entrypoint", "--value", "0x1000"])
        .assert()
        .success();
    cli()
        .args(["--workspace", workspace, "--target", "libtest"])
        .args(["hotpath", "add"])
        .args(["--addr", "0x1000", "--reason", "runtime sample"])
        .assert()
        .success();
    cli()
        .args(["--workspace", workspace, "--target", "libtest"])
        .args(["runtime", "validate"])
        .assert()
        .success();
    cli()
        .args(["--workspace", workspace, "--target", "libtest"])
        .args(["gate", "check", "--phase", "P1"])
        .assert()
        .success();

    cli()
        .args(["--workspace", workspace, "--target", "libtest"])
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
    let source = tmp.path().join("zlib-src");
    std::fs::create_dir_all(&source).unwrap();
    std::fs::write(source.join("README"), "upstream").unwrap();
    cli()
        .args(["--workspace", workspace, "--target", "libtest"])
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
        .args(["--workspace", workspace, "--target", "libtest"])
        .args(["third-party", "classify-function"])
        .args(["--addr", "0x1000", "--classification", "library"])
        .assert()
        .success();
    cli()
        .args(["--workspace", workspace, "--target", "libtest"])
        .args(["gate", "check", "--phase", "P2"])
        .assert()
        .success();

    cli()
        .args(["--workspace", workspace, "--target", "libtest"])
        .args(["metadata", "enrich-function"])
        .args([
            "--addr",
            "0x1000",
            "--name",
            "main",
            "--prototype",
            "int(void)",
        ])
        .assert()
        .success();
    cli()
        .args(["--workspace", workspace, "--target", "libtest"])
        .args(["metadata", "validate"])
        .assert()
        .success();
    cli()
        .args(["--workspace", workspace, "--target", "libtest"])
        .args(["hotpath", "validate"])
        .assert()
        .success();
    cli()
        .args(["--workspace", workspace, "--target", "libtest"])
        .args(["gate", "check", "--phase", "P3"])
        .assert()
        .success();

    cli()
        .args(["--workspace", workspace, "--target", "libtest"])
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
    cli()
        .args(["--workspace", workspace, "--target", "libtest"])
        .args(["substitute", "validate"])
        .assert()
        .success();
    cli()
        .args(["--workspace", workspace, "--target", "libtest"])
        .args(["gate", "check", "--phase", "P4"])
        .assert()
        .success();
}

#[test]
fn p1_gate_rejects_missing_or_empty_runtime_artifacts() {
    let tmp = TempDir::new().unwrap();
    init_workspace(&tmp, "libtest");
    add_minimal_p1_baseline(&tmp, "libtest");
    add_runtime_record(&tmp, "libtest", "entrypoint", "0x1000");

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .args(["gate", "check", "--phase", "P1"])
        .assert()
        .failure()
        .stdout(
            predicate::str::contains("gate check failed")
                .and(predicate::str::contains("P1_hotpath_call_chain")),
        );

    let ad = artifact_dir(&tmp, "libtest");
    std::fs::remove_file(
        ad.join("runtime")
            .join("run-records")
            .join("entrypoint.yaml"),
    )
    .unwrap();
    std::fs::write(
        ad.join("runtime").join("hotpaths").join("call-chain.yaml"),
        "target: libtest\nfunctions: []\n",
    )
    .unwrap();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .args(["gate", "check", "--phase", "P1"])
        .assert()
        .failure()
        .stdout(
            predicate::str::contains("P1_run_records")
                .and(predicate::str::contains("missing run record"))
                .and(predicate::str::contains("P1_hotpath_call_chain")),
        );
}

#[test]
fn runtime_validate_rejects_stale_manifest_record_references() {
    let tmp = TempDir::new().unwrap();
    init_workspace(&tmp, "libtest");
    add_runtime_record(&tmp, "libtest", "entrypoint", "0x1000");
    let record = artifact_dir(&tmp, "libtest")
        .join("runtime")
        .join("run-records")
        .join("entrypoint.yaml");
    std::fs::remove_file(record).unwrap();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .args(["runtime", "validate"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("failed to read")
                .or(predicate::str::contains("No such file")
                    .or(predicate::str::contains("missing"))),
        );
}

#[test]
fn p3_gate_rejects_hotpath_without_matching_rename_or_signature() {
    let tmp = TempDir::new().unwrap();
    init_workspace(&tmp, "libtest");
    add_hotpath(&tmp, "libtest", "0x1000");
    add_metadata(&tmp, "libtest", "0x2000");

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .args(["gate", "check", "--phase", "P3"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("0x1000 missing rename"));

    let metadata_dir = artifact_dir(&tmp, "libtest").join("metadata");
    std::fs::write(
        metadata_dir.join("renames.yaml"),
        "target: libtest\nrenames:\n  - addr: \"0x1000\"\n    name: main\n",
    )
    .unwrap();
    std::fs::write(
        metadata_dir.join("signatures.yaml"),
        "target: libtest\nsignatures:\n  - addr: \"0x2000\"\n    prototype: int(void)\n",
    )
    .unwrap();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .args(["gate", "check", "--phase", "P3"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("0x1000 missing signature"));
}

#[test]
fn p4_gate_rejects_empty_fixtures_and_missing_p3_metadata() {
    let tmp = TempDir::new().unwrap();
    init_workspace(&tmp, "libtest");
    add_metadata(&tmp, "libtest", "0x1000");
    add_substitution(&tmp, "libtest", "fn_001", "0x1000");

    let functions_dir = artifact_dir(&tmp, "libtest")
        .join("substitution")
        .join("functions");
    std::fs::write(
        functions_dir.join("fn_001").join("substitution.yaml"),
        r#"target: libtest
fn_id: fn_001
addr: "0x1000"
replacement: return 0;
fixtures: []
status: recorded
"#,
    )
    .unwrap();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .args(["gate", "check", "--phase", "P4"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("has no fixtures"));

    add_substitution(&tmp, "libtest", "fn_002", "0x2000");

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .args(["gate", "check", "--phase", "P4"])
        .assert()
        .failure()
        .stdout(
            predicate::str::contains("0x2000 missing metadata rename").and(
                predicate::str::contains("0x2000 missing metadata signature"),
            ),
        );
}

#[test]
fn p2_gate_rejects_libraries_without_source_or_pristine_dirs() {
    let tmp = TempDir::new().unwrap();
    init_workspace(&tmp, "libtest");

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
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
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .args(["third-party", "classify-function"])
        .args(["--addr", "0x1000", "--classification", "library"])
        .assert()
        .success();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .args(["gate", "check", "--phase", "P2"])
        .assert()
        .failure()
        .stdout(
            predicate::str::contains("zlib missing source_path")
                .and(predicate::str::contains("zlib missing pristine_path")),
        );

    std::fs::write(
        artifact_dir(&tmp, "libtest")
            .join("third-party")
            .join("identified.yaml"),
        r#"target: libtest
libraries:
  - library: zlib
    version: 1.2.13
    confidence: high
    source_path: /tmp/zlib-src
    pristine_path: third-party/pristine/zlib@1.2.13
    function_classifications:
      - addr: "0x1000"
        classification: library
"#,
    )
    .unwrap();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .args(["gate", "check", "--phase", "P2"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("zlib pristine directory missing"));
}

#[test]
fn p2_gate_accepts_explicit_empty_third_party_review() {
    let tmp = TempDir::new().unwrap();
    init_workspace(&tmp, "libtest");

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .args(["third-party", "none"])
        .args(["--evidence", "review found no third-party libraries"])
        .assert()
        .success();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .args(["gate", "check", "--phase", "P2"])
        .assert()
        .success()
        .stdout(predicate::str::contains("gate check passed"));
}

#[test]
fn git_check_rejects_untracked_nested_p1_and_p4_yaml_until_added() {
    let tmp = TempDir::new().unwrap();
    init_workspace(&tmp, "libtest");
    ProcessCommand::new("git")
        .arg("init")
        .current_dir(tmp.path())
        .status()
        .unwrap();
    ProcessCommand::new("git")
        .args(["add", "artifacts"])
        .current_dir(tmp.path())
        .status()
        .unwrap();

    let ad = artifact_dir(&tmp, "libtest");
    let nested_record_dir = ad.join("runtime").join("run-records").join("manual");
    std::fs::create_dir_all(&nested_record_dir).unwrap();
    std::fs::write(
        nested_record_dir.join("nested.yaml"),
        "target: libtest\nrun_id: nested\nstatus: recorded\nobservations: []\n",
    )
    .unwrap();
    let nested_substitution_dir = ad.join("substitution").join("functions").join("fn_nested");
    std::fs::create_dir_all(&nested_substitution_dir).unwrap();
    std::fs::write(
        nested_substitution_dir.join("substitution.yaml"),
        r#"target: libtest
fn_id: fn_nested
addr: "0x1000"
replacement: return 0;
fixtures:
  - fixture_id: fixture_001
    source: test
status: recorded
"#,
    )
    .unwrap();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .args(["git-check", "validate"])
        .assert()
        .failure()
        .stdout(
            predicate::str::contains("git-check validation failed")
                .and(predicate::str::contains(
                    "runtime/run-records/manual/nested.yaml",
                ))
                .and(predicate::str::contains(
                    "substitution/functions/fn_nested/substitution.yaml",
                )),
        );

    ProcessCommand::new("git")
        .args(["add", "artifacts"])
        .current_dir(tmp.path())
        .status()
        .unwrap();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .args(["git-check", "validate"])
        .assert()
        .success()
        .stdout(predicate::str::contains("git-check validation passed"));

    assert!(
        !ad.join("gates").join("git-check.yaml").exists(),
        "git-check validate should not create an untracked self-report artifact"
    );

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .args(["git-check", "validate"])
        .assert()
        .success()
        .stdout(predicate::str::contains("git-check validation passed"));
}

#[test]
fn legacy_gate_phases_are_accepted_as_deprecated() {
    let tmp = TempDir::new().unwrap();
    init_workspace(&tmp, "libtest");

    for phase in ["P0.5", "P5", "P6"] {
        cli()
            .args(["--workspace", tmp.path().to_str().unwrap()])
            .args(["--target", "libtest"])
            .args(["gate", "check", "--phase", phase])
            .assert()
            .success()
            .stdout(predicate::str::contains("deprecated"));
    }
}

#[test]
fn vendor_pristine_records_pristine_path_without_git_commit() {
    let tmp = TempDir::new().unwrap();
    init_workspace(&tmp, "libtest");
    ProcessCommand::new("git")
        .arg("init")
        .current_dir(tmp.path())
        .status()
        .unwrap();
    let source = tmp.path().join("zlib-src");
    std::fs::create_dir_all(&source).unwrap();
    std::fs::write(source.join("README"), "upstream").unwrap();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .args(["third-party", "add"])
        .args(["--library", "zlib", "--version", "1.2.13"])
        .assert()
        .success();
    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .args(["third-party", "vendor-pristine"])
        .args([
            "--library",
            "zlib",
            "--source-path",
            source.to_str().unwrap(),
            "--commit",
        ])
        .assert()
        .success();

    let identified = std::fs::read_to_string(
        tmp.path()
            .join("artifacts")
            .join("libtest")
            .join("third-party")
            .join("identified.yaml"),
    )
    .unwrap();
    assert!(identified.contains("pristine_path: third-party/pristine/zlib@1.2.13"));
    assert!(identified.contains("source_path:"));
    assert!(!identified.contains("vendored_path:"));
    assert!(
        ProcessCommand::new("git")
            .args(["rev-parse", "--verify", "HEAD"])
            .current_dir(tmp.path())
            .status()
            .unwrap()
            .code()
            .unwrap_or(1)
            != 0
    );
}

#[test]
fn vendor_pristine_sanitizes_library_and_version_path_components() {
    let tmp = TempDir::new().unwrap();
    init_workspace(&tmp, "libtest");
    let source = tmp.path().join("pkg-src");
    std::fs::create_dir_all(&source).unwrap();
    std::fs::write(source.join("README"), "upstream").unwrap();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .args(["third-party", "add"])
        .args(["--library", "scope/pkg", "--version", "../1.2.3"])
        .assert()
        .success();
    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .args(["third-party", "vendor-pristine"])
        .args([
            "--library",
            "scope/pkg",
            "--source-path",
            source.to_str().unwrap(),
        ])
        .assert()
        .success();

    let ad = artifact_dir(&tmp, "libtest");
    assert!(ad.join("third-party/pristine/scope_pkg@.._1.2.3").is_dir());
    assert!(!ad.join("third-party/1.2.3").exists());
}

#[test]
fn substitute_add_rejects_unsafe_fn_id_path_components() {
    let tmp = TempDir::new().unwrap();
    init_workspace(&tmp, "libtest");

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .args(["substitute", "add"])
        .args([
            "--fn-id",
            "../escape",
            "--addr",
            "0x1000",
            "--replacement",
            "return 0;",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("fn-id must contain only ASCII"));

    assert!(
        !artifact_dir(&tmp, "libtest")
            .join("substitution")
            .join("escape")
            .exists()
    );
}

#[test]
fn git_check_and_gate_require_tracked_or_staged_artifacts_in_git_repo() {
    let tmp = TempDir::new().unwrap();
    init_workspace(&tmp, "libtest");
    ProcessCommand::new("git")
        .arg("init")
        .current_dir(tmp.path())
        .status()
        .unwrap();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .args(["gate", "check", "--phase", "P0"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("git_tracking"));

    ProcessCommand::new("git")
        .args(["add", "artifacts"])
        .current_dir(tmp.path())
        .status()
        .unwrap();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .args(["git-check", "validate"])
        .assert()
        .success();
    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .args(["gate", "check", "--phase", "P0"])
        .assert()
        .success();
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
fn ghidra_analyze_vtables_with_mock_ghidra_writes_report_and_baseline() {
    let tmp = TempDir::new().unwrap();
    init_workspace(&tmp, "libtest");
    let fake_ghidra = install_fake_ghidra(&tmp);

    cli()
        .env("GHIDRA_INSTALL_DIR", &fake_ghidra)
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .arg("ghidra")
        .arg("analyze-vtables")
        .arg("--write-baseline")
        .assert()
        .success()
        .stdout(predicate::str::contains("vtable analysis complete"))
        .stdout(predicate::str::contains("report_path"));

    let baseline_dir = tmp
        .path()
        .join("artifacts")
        .join("libtest")
        .join("baseline");
    let report = std::fs::read_to_string(baseline_dir.join("vtable-analysis-report.yaml")).unwrap();
    assert!(report.contains("candidates:"));
    assert!(report.contains("MockClass"));
    assert!(report.contains("score: 8"));
    assert!(report.contains("status: accepted"));

    let baseline = std::fs::read_to_string(baseline_dir.join("vtables.yaml")).unwrap();
    assert!(baseline.contains("class: \"MockClass\""));
    assert!(baseline.contains("\"0x1000\""));

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .arg("validate")
        .args(["--schema", "vtable-analysis"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("schema:vtable-analysis"));
}

#[cfg(unix)]
#[test]
fn ghidra_analyze_vtables_with_bundled_entry_keeps_cli_compatibility() {
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
        .arg("analyze-vtables")
        .arg("--write-baseline")
        .assert()
        .success()
        .stdout(predicate::str::contains("vtable analysis complete"));

    let baseline_dir = tmp
        .path()
        .join("artifacts")
        .join("libtest")
        .join("baseline");
    assert!(baseline_dir.join("vtable-analysis-report.yaml").exists());
    assert!(baseline_dir.join("vtables.yaml").exists());
}

#[cfg(unix)]
#[test]
fn ghidra_apply_signatures_preserves_names_unless_rename_flag_is_set() {
    let tmp = TempDir::new().unwrap();
    init_workspace(&tmp, "libtest");
    let fake_ghidra = install_fake_ghidra(&tmp);
    let fake_bundle = install_fake_bundled_entry(&tmp);
    let metadata_dir = tmp
        .path()
        .join("artifacts")
        .join("libtest")
        .join("metadata");
    std::fs::create_dir_all(&metadata_dir).unwrap();
    std::fs::write(
        metadata_dir.join("signatures.yaml"),
        "target: libtest\nsignatures:\n  - addr: 0x1000\n    prototype: int decode(void)\n",
    )
    .unwrap();

    cli()
        .env("GHIDRA_INSTALL_DIR", &fake_ghidra)
        .env("GHIDRA_SCRIPTS_DIR", &fake_bundle)
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .args(["ghidra", "apply-signatures"])
        .assert()
        .success()
        .stdout(predicate::str::contains("signatures applied"));

    let record_path = tmp
        .path()
        .join("artifacts")
        .join("libtest")
        .join("runtime")
        .join("apply-signatures-record.yaml");
    let record = std::fs::read_to_string(&record_path).unwrap();
    assert!(record.contains("script: \"ApplySignatures.java\""));
    assert!(record.contains("program: \"dummy.bin\""));
    assert!(record.contains("extra_args: \"\""));

    cli()
        .env("GHIDRA_INSTALL_DIR", &fake_ghidra)
        .env("GHIDRA_SCRIPTS_DIR", &fake_bundle)
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .args(["ghidra", "apply-signatures", "--rename-from-signature"])
        .assert()
        .success();

    let record = std::fs::read_to_string(record_path).unwrap();
    assert!(record.contains("--rename-from-signature"));
}

#[cfg(unix)]
#[test]
fn ghidra_import_types_and_signatures_uses_pipeline_binary_name_and_default_signatures() {
    let tmp = TempDir::new().unwrap();
    init_workspace(&tmp, "libtest");
    let fake_ghidra = install_fake_ghidra(&tmp);
    let headers_dir = tmp.path().join("include");
    let fake_bundle = install_fake_bundled_entry(&tmp);
    std::fs::create_dir_all(&headers_dir).unwrap();
    let metadata_dir = tmp
        .path()
        .join("artifacts")
        .join("libtest")
        .join("metadata");
    std::fs::create_dir_all(&metadata_dir).unwrap();

    let context_header = headers_dir.join("custom_types.h");
    let api_header = headers_dir.join("custom_api.h");
    let default_signatures = metadata_dir.join("signatures.yaml");
    std::fs::write(
        &context_header,
        "typedef struct CustomContext { int x; } CustomContext;",
    )
    .unwrap();
    std::fs::write(
        &api_header,
        "typedef struct CustomRecord { int y; } CustomRecord;",
    )
    .unwrap();
    std::fs::write(
        &default_signatures,
        "target: libtest\nsignatures:\n  - addr: 0x1000\n    prototype: int decode(void)\n",
    )
    .unwrap();

    cli()
        .env("GHIDRA_INSTALL_DIR", &fake_ghidra)
        .env("GHIDRA_SCRIPTS_DIR", &fake_bundle)
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .arg("ghidra")
        .arg("import-types-and-signatures")
        .args(["--header", context_header.to_str().unwrap()])
        .args(["--header", api_header.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("types and signatures imported"))
        .stdout(predicate::str::contains("headers"))
        .stdout(predicate::str::contains("dummy.bin"))
        .stdout(predicate::str::contains(context_header.to_str().unwrap()))
        .stdout(predicate::str::contains(api_header.to_str().unwrap()));
    // Default signatures path should be reported when metadata/signatures.yaml exists.

    let record = std::fs::read_to_string(
        tmp.path()
            .join("artifacts")
            .join("libtest")
            .join("runtime")
            .join("import-types-and-signatures-record.yaml"),
    )
    .unwrap();
    assert!(record.contains("script: \"ImportTypesAndSignatures.java\""));
    assert!(record.contains("program: \"dummy.bin\""));
    assert!(record.contains(context_header.to_str().unwrap()));
    assert!(record.contains(api_header.to_str().unwrap()));
    assert!(record.contains(default_signatures.to_str().unwrap()));
}

#[cfg(unix)]
#[test]
fn ghidra_import_types_and_signatures_passes_explicit_signatures_path() {
    let tmp = TempDir::new().unwrap();
    init_workspace(&tmp, "libtest");
    let fake_ghidra = install_fake_ghidra(&tmp);
    let fake_bundle = install_fake_bundled_entry(&tmp);
    let headers_dir = tmp.path().join("include");
    let input_dir = tmp.path().join("input");
    std::fs::create_dir_all(&headers_dir).unwrap();
    std::fs::create_dir_all(&input_dir).unwrap();

    let context_header = headers_dir.join("custom_types.h");
    let explicit_signatures = input_dir.join("custom-signatures.yaml");
    std::fs::write(
        &context_header,
        "typedef struct CustomContext { int x; } CustomContext;",
    )
    .unwrap();
    std::fs::write(
        &explicit_signatures,
        "target: libtest\nsignatures:\n  - addr: 0x1000\n    signature: int decode(void)\n",
    )
    .unwrap();

    cli()
        .env("GHIDRA_INSTALL_DIR", &fake_ghidra)
        .env("GHIDRA_SCRIPTS_DIR", &fake_bundle)
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "libtest"])
        .arg("ghidra")
        .arg("import-types-and-signatures")
        .args(["--header", context_header.to_str().unwrap()])
        .args(["--include-dir", headers_dir.to_str().unwrap()])
        .args(["--signatures", explicit_signatures.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("ImportTypesAndSignatures.java"))
        .stdout(predicate::str::contains(headers_dir.to_str().unwrap()))
        .stdout(predicate::str::contains(
            explicit_signatures.to_str().unwrap(),
        ));

    let record = std::fs::read_to_string(
        tmp.path()
            .join("artifacts")
            .join("libtest")
            .join("runtime")
            .join("import-types-and-signatures-record.yaml"),
    )
    .unwrap();
    assert!(record.contains("program: \"dummy.bin\""));
    assert!(record.contains(context_header.to_str().unwrap()));
    assert!(record.contains(headers_dir.to_str().unwrap()));
    assert!(record.contains(explicit_signatures.to_str().unwrap()));
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
