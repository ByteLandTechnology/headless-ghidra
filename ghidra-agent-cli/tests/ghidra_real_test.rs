use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;
use tempfile::TempDir;

fn cli() -> Command {
    Command::cargo_bin("ghidra-agent-cli").unwrap()
}

fn compiler_path() -> Option<&'static str> {
    if ProcessCommand::new("/usr/bin/clang++")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        Some("/usr/bin/clang++")
    } else {
        None
    }
}

fn c_compiler_path() -> Option<&'static str> {
    if ProcessCommand::new("/usr/bin/clang")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        Some("/usr/bin/clang")
    } else {
        None
    }
}

fn command_available(command: &str, args: &[&str]) -> bool {
    ProcessCommand::new(command)
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

fn create_real_vtable_binary(tmp: &TempDir) -> PathBuf {
    let src = tmp.path().join("real_vtable_sample.cpp");
    fs::write(
        &src,
        r#"
struct Base {
    virtual ~Base() {}
    virtual int f(int x) { return x + 1; }
    virtual void g() {}
};

struct Derived : Base {
    ~Derived() override {}
    int f(int x) override { return x + 2; }
    void g() override {}
};

int use(Base* b) {
    b->g();
    return b->f(40);
}

int main() {
    Derived d;
    return use(&d);
}
"#,
    )
    .unwrap();

    let output = tmp.path().join("real_vtable_sample");
    let compiler = compiler_path().expect("clang++ must be available for real ghidra tests");
    let result = ProcessCommand::new(compiler)
        .args(["-std=c++17", "-O0", "-g", "-fno-inline"])
        .arg("-o")
        .arg(&output)
        .arg(&src)
        .output()
        .expect("failed to invoke clang++");

    assert!(
        result.status.success(),
        "clang failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );

    output
}

fn create_real_pipeline_binary(tmp: &TempDir) -> PathBuf {
    let src = tmp.path().join("real_pipeline_sample.c");
    fs::write(
        &src,
        r#"
#include <stdio.h>
#include <stdlib.h>

__attribute__((visibility("default"))) __attribute__((noinline))
int hg_hot_function(int x) {
    printf("hg_hot_function(%d)\n", x);
    return x * 7 + 3;
}

__attribute__((visibility("default"))) __attribute__((noinline))
int hg_bridge_function(int x) {
    int result = hg_hot_function(x + 2);
    return result;
}

int main(int argc, char **argv) {
    int value = argc > 1 ? atoi(argv[1]) : 5;
    int result = hg_bridge_function(value);
    printf("result=%d\n", result);
    return result == 52 ? 0 : 1;
}
"#,
    )
    .unwrap();

    let output = tmp.path().join("real_pipeline_sample");
    let compiler = c_compiler_path().expect("clang must be available for real integration tests");
    let mut command = ProcessCommand::new(compiler);
    command.args(["-O0", "-g", "-fno-inline"]);
    if cfg!(target_os = "macos") {
        command.arg("-Wl,-export_dynamic");
    } else {
        command.arg("-rdynamic");
    }
    let result = command.arg("-o").arg(&output).arg(&src).output().unwrap();

    assert!(
        result.status.success(),
        "clang++ failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );

    output
}

fn cli_output(args: &[&str]) -> String {
    let scripts_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("ghidra-scripts");
    let assert = cli()
        .env("GHIDRA_SCRIPTS_DIR", scripts_dir)
        .args(args)
        .assert()
        .success();
    String::from_utf8_lossy(&assert.get_output().stdout).into_owned()
}

fn workspace_args<'a>(workspace: &'a Path, target: &'a str) -> Vec<&'a str> {
    vec![
        "--workspace",
        workspace.to_str().unwrap(),
        "--target",
        target,
    ]
}

fn run_cli(workspace: &Path, target: &str, args: &[&str]) -> String {
    let mut all = workspace_args(workspace, target);
    all.extend_from_slice(args);
    cli_output(&all)
}

fn find_function_addr(workspace: &Path, target: &str, name_fragment: &str) -> (String, String) {
    let functions_path = workspace
        .join("artifacts")
        .join(target)
        .join("baseline")
        .join("functions.yaml");
    let content = fs::read_to_string(&functions_path).unwrap();
    let doc: serde_yaml::Value = serde_yaml::from_str(&content).unwrap();
    let functions = doc
        .get("functions")
        .and_then(|v| v.as_sequence())
        .expect("baseline/functions.yaml must contain functions");

    for (idx, function) in functions.iter().enumerate() {
        let name = function
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        if name.contains(name_fragment) {
            let addr = function
                .get("addr")
                .and_then(|v| v.as_str())
                .expect("function must have addr")
                .to_string();
            return (format!("fn_{:03}", idx + 1), addr);
        }
    }

    panic!("function containing '{name_fragment}' not found in:\n{content}");
}

fn init_git_repo(path: &Path) {
    let output = ProcessCommand::new("git")
        .arg("init")
        .current_dir(path)
        .output()
        .expect("failed to run git init");
    assert!(
        output.status.success(),
        "git init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn git_add_artifacts(path: &Path) {
    let output = ProcessCommand::new("git")
        .args(["add", "artifacts"])
        .current_dir(path)
        .output()
        .expect("failed to run git add artifacts");
    assert!(
        output.status.success(),
        "git add artifacts failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn write_real_pristine_source(tmp: &TempDir) -> PathBuf {
    let pristine = tmp.path().join("pristine-real-lib");
    fs::create_dir_all(&pristine).unwrap();
    fs::write(
        pristine.join("real_lib.cpp"),
        r#"
extern "C" int hg_hot_function(int x);

int call_hot_from_pristine_source(int x) {
    return hg_hot_function(x);
}
"#,
    )
    .unwrap();
    pristine
}

#[cfg(unix)]
#[test]
fn ghidra_analyze_vtables_real_binary_detects_candidates() {
    if compiler_path().is_none() || !ghidra_available() {
        eprintln!("skipping real Ghidra vtable test; clang++ or Ghidra is unavailable");
        return;
    }

    let tmp = TempDir::new().unwrap();
    let binary = create_real_vtable_binary(&tmp);

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .arg("workspace")
        .arg("init")
        .args(["--target", "real_vtable_sample"])
        .args(["--binary", binary.to_str().unwrap()])
        .assert()
        .success();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "real_vtable_sample"])
        .arg("ghidra")
        .arg("import")
        .assert()
        .success()
        .stdout(predicate::str::contains("imported"));

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "real_vtable_sample"])
        .arg("ghidra")
        .arg("auto-analyze")
        .assert()
        .success()
        .stdout(predicate::str::contains("auto-analysis complete"));

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .args(["--target", "real_vtable_sample"])
        .arg("ghidra")
        .arg("analyze-vtables")
        .args(["--write-baseline", "--overwrite"])
        .assert()
        .success()
        .stdout(predicate::str::contains("vtable analysis complete"));

    let baseline_dir = tmp
        .path()
        .join("artifacts")
        .join("real_vtable_sample")
        .join("baseline");
    let report = fs::read_to_string(baseline_dir.join("vtable-analysis-report.yaml")).unwrap();
    let baseline = fs::read_to_string(baseline_dir.join("vtables.yaml")).unwrap();

    assert!(report.contains("candidates:"), "report was:\n{report}");
    assert!(
        report.contains("status: accepted"),
        "expected an accepted candidate, report was:\n{report}"
    );
    assert!(baseline.contains("vtables:"), "baseline was:\n{baseline}");
    assert!(
        baseline.contains("associated_type:") || baseline.contains("class:"),
        "expected class/type information, baseline was:\n{baseline}"
    );
}

#[cfg(unix)]
#[test]
fn frida_real_run_and_trace_capture_exported_function() {
    if c_compiler_path().is_none() || !frida_available() {
        eprintln!("skipping real Frida trace test; clang or Frida is unavailable");
        return;
    }

    let tmp = TempDir::new().unwrap();
    let binary = create_real_pipeline_binary(&tmp);
    let binary = binary.to_str().unwrap();

    let run_output = cli_output(&[
        "frida",
        "run",
        "--target",
        binary,
        "--args",
        "5",
        "--timeout",
        "10",
    ]);
    assert!(
        run_output.contains("hg_hot_function"),
        "run output was:\n{run_output}"
    );
    assert!(
        run_output.contains(r#""type":"call""#),
        "run output was:\n{run_output}"
    );
    assert!(
        run_output.contains(r#""type":"return""#),
        "run output was:\n{run_output}"
    );

    let trace_output = cli_output(&[
        "frida",
        "trace",
        "--target",
        binary,
        "--functions",
        "hg_hot_function",
        "--timeout",
        "10",
        "--",
        "5",
    ]);
    assert!(
        trace_output.contains("hg_hot_function"),
        "trace output was:\n{trace_output}"
    );
    assert!(
        trace_output.contains(r#""type":"return""#),
        "trace output was:\n{trace_output}"
    );
}

#[cfg(unix)]
#[test]
fn real_p0_p4_pipeline_end_to_end_no_mocks() {
    if c_compiler_path().is_none() || !ghidra_available() || !frida_available() {
        eprintln!("skipping real P0-P4 pipeline test; clang, Ghidra, or Frida is unavailable");
        return;
    }

    let tmp = TempDir::new().unwrap();
    init_git_repo(tmp.path());

    let target = "real_pipeline_sample";
    let binary = create_real_pipeline_binary(&tmp);
    let binary_str = binary.to_str().unwrap();

    cli()
        .args(["--workspace", tmp.path().to_str().unwrap()])
        .arg("workspace")
        .arg("init")
        .args(["--target", target])
        .args(["--binary", binary_str])
        .assert()
        .success();

    run_cli(tmp.path(), target, &["ghidra", "import"]);
    run_cli(tmp.path(), target, &["ghidra", "auto-analyze"]);
    run_cli(tmp.path(), target, &["ghidra", "export-baseline"]);

    let (fn_id, hot_addr) = find_function_addr(tmp.path(), target, "hg_hot_function");

    let native_run = ProcessCommand::new(&binary)
        .arg("5")
        .output()
        .expect("failed to run real sample binary");
    assert!(
        native_run.status.success(),
        "sample binary failed: stdout=\n{}\nstderr=\n{}",
        String::from_utf8_lossy(&native_run.stdout),
        String::from_utf8_lossy(&native_run.stderr)
    );
    let fixture_dir = tmp
        .path()
        .join("artifacts")
        .join(target)
        .join("runtime")
        .join("fixtures")
        .join("native-run");
    fs::create_dir_all(&fixture_dir).unwrap();
    fs::write(fixture_dir.join("stdout.txt"), &native_run.stdout).unwrap();
    fs::write(fixture_dir.join("stderr.txt"), &native_run.stderr).unwrap();

    run_cli(
        tmp.path(),
        target,
        &[
            "runtime",
            "record",
            "--key",
            "native-run",
            "--value",
            "runtime/fixtures/native-run/stdout.txt",
            "--note",
            "real sample binary executed once and stdout/stderr fixtures were persisted",
        ],
    );

    let frida_run = cli_output(&[
        "frida",
        "run",
        "--target",
        binary_str,
        "--args",
        "5",
        "--timeout",
        "10",
    ]);
    assert!(
        frida_run.contains("hg_hot_function"),
        "frida run was:\n{frida_run}"
    );
    assert!(
        frida_run.contains(r#""type":"return""#),
        "frida run was:\n{frida_run}"
    );

    let frida_trace = cli_output(&[
        "frida",
        "trace",
        "--target",
        binary_str,
        "--functions",
        "hg_hot_function",
        "--timeout",
        "10",
        "--",
        "5",
    ]);
    assert!(
        frida_trace.contains("hg_hot_function"),
        "frida trace was:\n{frida_trace}"
    );
    assert!(
        frida_trace.contains(r#""type":"return""#),
        "frida trace was:\n{frida_trace}"
    );

    run_cli(
        tmp.path(),
        target,
        &[
            "hotpath",
            "add",
            "--addr",
            &hot_addr,
            "--reason",
            "real frida trace captured hg_hot_function",
            "--weight",
            "10",
        ],
    );

    let pristine = write_real_pristine_source(&tmp);
    run_cli(
        tmp.path(),
        target,
        &[
            "third-party",
            "add",
            "--library",
            "real-pristine-lib",
            "--version",
            "1.0.0",
            "--confidence",
            "medium",
            "--evidence",
            "real source tree copied by vendor-pristine",
        ],
    );
    run_cli(
        tmp.path(),
        target,
        &[
            "third-party",
            "vendor-pristine",
            "--library",
            "real-pristine-lib",
            "--source-path",
            pristine.to_str().unwrap(),
        ],
    );
    run_cli(
        tmp.path(),
        target,
        &[
            "third-party",
            "classify-function",
            "--addr",
            &hot_addr,
            "--classification",
            "library-adjacent-hotpath",
            "--evidence",
            "real baseline address selected from Ghidra export",
        ],
    );

    run_cli(
        tmp.path(),
        target,
        &[
            "metadata",
            "enrich-function",
            "--addr",
            &hot_addr,
            "--name",
            "hg_hot_function",
            "--prototype",
            "int hg_hot_function(int)",
            "--note",
            "real Ghidra baseline address",
        ],
    );

    run_cli(
        tmp.path(),
        target,
        &[
            "ghidra",
            "decompile",
            "--fn-id",
            &fn_id,
            "--addr",
            &hot_addr,
        ],
    );
    let decomp_dir = tmp
        .path()
        .join("artifacts")
        .join(target)
        .join("decompilation")
        .join("functions")
        .join(&fn_id);
    assert!(decomp_dir.join(format!("{fn_id}.c")).exists());
    assert!(decomp_dir.join("decompilation-record.yaml").exists());

    run_cli(
        tmp.path(),
        target,
        &[
            "substitute",
            "add",
            "--fn-id",
            &fn_id,
            "--addr",
            &hot_addr,
            "--replacement",
            "return x * 7 + 3;",
            "--note",
            "substitution recorded after real Ghidra decompilation",
        ],
    );

    git_add_artifacts(tmp.path());

    run_cli(tmp.path(), target, &["runtime", "validate"]);
    run_cli(tmp.path(), target, &["hotpath", "validate"]);
    run_cli(tmp.path(), target, &["metadata", "validate"]);
    run_cli(tmp.path(), target, &["substitute", "validate"]);
    run_cli(tmp.path(), target, &["git-check", "validate"]);

    let gate_output = run_cli(tmp.path(), target, &["gate", "check", "--phase", "all"]);
    assert!(
        gate_output.contains("gate check passed"),
        "gate output was:\n{gate_output}"
    );
}
