use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
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
        "clang++ failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );

    output
}

#[cfg(unix)]
#[test]
#[ignore = "requires real Ghidra installation and writable host config"]
fn ghidra_analyze_vtables_real_binary_detects_candidates() {
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
