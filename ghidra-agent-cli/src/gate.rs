#![allow(clippy::collapsible_if)]

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::git_status;
use crate::schema::save_yaml;
use crate::workspace::artifact_dir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateReport {
    pub target: String,
    pub phase: String,
    pub passed: bool,
    pub checks: Vec<GateCheck>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateCheck {
    pub id: String,
    pub description: String,
    pub passed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

pub fn save_gate_report(workspace: &Path, target: &str, report: &GateReport) -> Result<()> {
    save_yaml(
        &artifact_dir(workspace, target)
            .join("gates")
            .join(format!("{}-report.yaml", report.phase.to_lowercase())),
        report,
    )
}

pub const ALL_PHASES: &[&str] = &["P0", "P1", "P2", "P3", "P4"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseInfo {
    pub phase: String,
    pub name: String,
    pub checks: Vec<(String, String)>,
}

pub fn phase_descriptions() -> Vec<PhaseInfo> {
    vec![
        PhaseInfo {
            phase: "P0".into(),
            name: "Intake".into(),
            checks: vec![(
                "P0_01".into(),
                "pipeline-state.yaml exists and is valid YAML with target field".into(),
            )],
        },
        PhaseInfo {
            phase: "P1".into(),
            name: "Baseline+Runtime".into(),
            checks: vec![
                (
                    "P1_baseline".into(),
                    "baseline YAML files exist and are parseable".into(),
                ),
                (
                    "P1_runtime".into(),
                    "runtime run manifest, run records, and hotpath call-chain exist".into(),
                ),
            ],
        },
        PhaseInfo {
            phase: "P2".into(),
            name: "Third-Party".into(),
            checks: vec![
                (
                    "P2_01".into(),
                    "third-party/identified.yaml exists and is parseable".into(),
                ),
                (
                    "P2_02".into(),
                    "third-party: >= 1 library with confidence >= medium".into(),
                ),
                (
                    "P2_03".into(),
                    "third-party: function_classification coverage > 0%".into(),
                ),
            ],
        },
        PhaseInfo {
            phase: "P3".into(),
            name: "Metadata Enrichment".into(),
            checks: vec![
                (
                    "P3_01".into(),
                    "metadata renames and signatures exist with entries".into(),
                ),
                (
                    "P3_02".into(),
                    "metadata covers all P1 hotpath functions".into(),
                ),
            ],
        },
        PhaseInfo {
            phase: "P4".into(),
            name: "Function Substitution".into(),
            checks: vec![
                (
                    "P4_01".into(),
                    "substitution function records exist with fixtures".into(),
                ),
                (
                    "P4_02".into(),
                    "substitutions reference P3 named and typed functions".into(),
                ),
            ],
        },
    ]
}

fn validate_yaml_has_field(path: &Path, field: &str) -> Result<bool> {
    let content = fs::read_to_string(path)?;
    let doc: serde_yaml::Value = serde_yaml::from_str(&content)?;
    Ok(doc.get(field).is_some())
}

fn validate_yaml_parseable(path: &Path) -> bool {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return false,
    };
    serde_yaml::from_str::<serde_yaml::Value>(&content).is_ok()
}

fn validate_yaml_sequence_nonempty(path: &Path, field: &str) -> bool {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return false,
    };
    let doc = match serde_yaml::from_str::<serde_yaml::Value>(&content) {
        Ok(d) => d,
        Err(_) => return false,
    };
    doc.get(field)
        .and_then(|v| v.as_sequence())
        .map(|seq| !seq.is_empty())
        .unwrap_or(false)
}

pub fn check_phase(workspace: &Path, target: &str, phase: &str) -> Result<GateReport> {
    let ad = artifact_dir(workspace, target);

    let mut checks = match phase {
        "P0" => p0_checks(workspace, target, &ad),
        "P0.5" => legacy_phase_checks("P0.5", "P0.5 is deprecated; use P1 Baseline+Runtime"),
        "P1" => p1_checks(workspace, target, &ad),
        "P2" => p2_checks(workspace, target, &ad),
        "P3" => p3_checks(workspace, target, &ad),
        "P4" => p4_checks(workspace, target, &ad),
        "P5" => legacy_phase_checks("P5", "P5 is deprecated; use P4 Function Substitution"),
        "P6" => legacy_phase_checks("P6", "P6 is deprecated; use P4 Function Substitution"),
        _ => vec![GateCheck {
            id: format!("{}_01", phase.replace('.', "_")),
            description: format!("{} gate placeholder", phase),
            passed: true,
            detail: None,
        }],
    };
    if matches!(phase, "P0" | "P1" | "P2" | "P3" | "P4") {
        checks.push(git_tracking_check(
            workspace,
            target,
            phase,
            &phase_artifacts(phase),
        ));
    }

    let passed = checks.iter().all(|c| c.passed);
    Ok(GateReport {
        target: target.to_string(),
        phase: phase.to_string(),
        passed,
        checks,
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

fn legacy_phase_checks(phase: &str, detail: &str) -> Vec<GateCheck> {
    vec![GateCheck {
        id: format!("{}_deprecated", phase.replace('.', "_")),
        description: detail.into(),
        passed: true,
        detail: Some("legacy phase accepted for compatibility only".into()),
    }]
}

fn phase_artifacts(phase: &str) -> Vec<&'static str> {
    match phase {
        "P0" => vec!["pipeline-state.yaml", "scope.yaml"],
        "P1" => vec![
            "baseline/functions.yaml",
            "baseline/callgraph.yaml",
            "baseline/types.yaml",
            "baseline/vtables.yaml",
            "baseline/constants.yaml",
            "baseline/strings.yaml",
            "baseline/imports.yaml",
            "runtime/run-manifest.yaml",
            "runtime/run-records",
            "runtime/hotpaths/call-chain.yaml",
        ],
        "P2" => vec!["third-party/identified.yaml"],
        "P3" => vec![
            "metadata/renames.yaml",
            "metadata/signatures.yaml",
            "runtime/hotpaths/call-chain.yaml",
        ],
        "P4" => vec!["substitution/next-batch.yaml", "substitution/functions"],
        _ => vec![],
    }
}

fn git_tracking_check(
    workspace: &Path,
    target: &str,
    phase: &str,
    artifact_rel_paths: &[&str],
) -> GateCheck {
    let worktree = match git_status::discover_worktree(workspace) {
        Ok(Some(worktree)) => worktree,
        Ok(None) => {
            return GateCheck {
                id: format!("{}_git_tracking", phase),
                description: "phase artifacts are tracked or staged in git".into(),
                passed: true,
                detail: Some(
                    "workspace is not in a git repository; git tracking check skipped".into(),
                ),
            };
        }
        Err(err) => {
            return GateCheck {
                id: format!("{}_git_tracking", phase),
                description: "phase artifacts are tracked or staged in git".into(),
                passed: false,
                detail: Some(err.to_string()),
            };
        }
    };

    let mut failures = Vec::new();
    for rel in artifact_rel_paths {
        let path = artifact_dir(workspace, target).join(rel);
        if !path.exists() {
            failures.push(format!("{} missing", path.display()));
            continue;
        }
        if path.is_dir() {
            match directory_yaml_tracking_failures(&worktree, &path) {
                Ok(mut inner) => failures.append(&mut inner),
                Err(err) => failures.push(format!("{}: {err}", path.display())),
            }
        } else if let Some(detail) = git_path_tracking_failure(&worktree, &path) {
            failures.push(detail);
        }
    }

    GateCheck {
        id: format!("{}_git_tracking", phase),
        description: "phase artifacts are tracked or staged in git".into(),
        passed: failures.is_empty(),
        detail: if failures.is_empty() {
            None
        } else {
            Some(failures.join("; "))
        },
    }
}

fn git_path_tracking_failure(worktree: &git_status::GitWorktree, path: &Path) -> Option<String> {
    let repo_rel = git_status::repo_relative_path(worktree, path);
    let status = git_status::status_file(worktree, &repo_rel);
    if status.tracked_or_staged {
        None
    } else {
        Some(format!("{} status {}", repo_rel.display(), status.display))
    }
}

fn directory_yaml_tracking_failures(
    worktree: &git_status::GitWorktree,
    dir: &Path,
) -> Result<Vec<String>> {
    let mut failures = Vec::new();
    let mut yaml_count = 0usize;
    for entry in walkdir::WalkDir::new(dir) {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                failures.push(err.to_string());
                continue;
            }
        };
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("yaml") {
            yaml_count += 1;
            if let Some(detail) = git_path_tracking_failure(worktree, path) {
                failures.push(detail);
            }
        }
    }
    if yaml_count == 0 {
        failures.push(format!("{} has no YAML files", dir.display()));
    }
    Ok(failures)
}

fn directory_tracking_failures(
    worktree: &git_status::GitWorktree,
    dir: &Path,
) -> Result<Vec<String>> {
    let mut failures = Vec::new();
    let mut file_count = 0usize;
    for entry in walkdir::WalkDir::new(dir) {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                failures.push(err.to_string());
                continue;
            }
        };
        let path = entry.path();
        if path.is_file() {
            file_count += 1;
            if let Some(detail) = git_path_tracking_failure(worktree, path) {
                failures.push(detail);
            }
        }
    }
    if file_count == 0 {
        failures.push(format!("{} has no files", dir.display()));
    }
    Ok(failures)
}

fn p0_checks(_workspace: &Path, _target: &str, ad: &Path) -> Vec<GateCheck> {
    let path = ad.join("pipeline-state.yaml");
    let exists = path.exists();
    let mut checks = vec![GateCheck {
        id: "P0_01".into(),
        description: "pipeline-state.yaml exists and is valid YAML with target field".into(),
        passed: exists
            && validate_yaml_parseable(&path)
            && validate_yaml_has_field(&path, "target").unwrap_or(false),
        detail: if !exists {
            Some("pipeline-state.yaml not found".into())
        } else if !validate_yaml_parseable(&path) {
            Some("pipeline-state.yaml is not valid YAML".into())
        } else if validate_yaml_has_field(&path, "target").unwrap_or(false) {
            None
        } else {
            Some("pipeline-state.yaml missing required 'target' field".into())
        },
    }];

    // P0_02: pipeline-state.yaml has phase field
    let has_phase = exists && validate_yaml_has_field(&path, "phase").unwrap_or(false);
    checks.push(GateCheck {
        id: "P0_02".into(),
        description: "pipeline-state.yaml has phase field".into(),
        passed: has_phase,
        detail: if has_phase {
            None
        } else {
            Some("pipeline-state.yaml missing required 'phase' field".into())
        },
    });

    checks
}

fn p1_checks(workspace: &Path, target: &str, ad: &Path) -> Vec<GateCheck> {
    let names = [
        "functions.yaml",
        "callgraph.yaml",
        "types.yaml",
        "vtables.yaml",
        "constants.yaml",
        "strings.yaml",
        "imports.yaml",
    ];
    let mut checks: Vec<GateCheck> = names
        .iter()
        .map(|name| {
            let path = ad.join("baseline").join(name);
            let exists = path.exists();
            let parseable = exists && validate_yaml_parseable(&path);
            GateCheck {
                id: format!("P1_{}", name.replace('.', "_")),
                description: format!("baseline/{name} exists and is parseable"),
                passed: parseable,
                detail: if !exists {
                    Some(format!("baseline/{name} not found"))
                } else if !parseable {
                    Some(format!("baseline/{name} is not valid YAML"))
                } else {
                    None
                },
            }
        })
        .collect();

    let runtime_path = ad.join("runtime").join("run-manifest.yaml");
    checks.push(GateCheck {
        id: "P1_run_manifest_yaml".into(),
        description: "runtime/run-manifest.yaml exists with non-empty observations".into(),
        passed: validate_yaml_sequence_nonempty(&runtime_path, "observations"),
        detail: if runtime_path.exists() {
            None
        } else {
            Some("runtime/run-manifest.yaml not found".into())
        },
    });

    let (run_record_count, run_record_failures) = runtime_run_record_failures(ad);
    checks.push(GateCheck {
        id: "P1_run_records".into(),
        description: "runtime/run-manifest.yaml references parseable run record YAML".into(),
        passed: run_record_count > 0 && run_record_failures.is_empty(),
        detail: if run_record_failures.is_empty() {
            Some(format!(
                "{} manifest run records verified",
                run_record_count
            ))
        } else {
            Some(run_record_failures.join("; "))
        },
    });

    let hotpath_path = ad.join("runtime").join("hotpaths").join("call-chain.yaml");
    checks.push(GateCheck {
        id: "P1_hotpath_call_chain".into(),
        description: "runtime/hotpaths/call-chain.yaml exists with non-empty functions".into(),
        passed: validate_yaml_sequence_nonempty(&hotpath_path, "functions"),
        detail: if hotpath_path.exists() {
            None
        } else {
            Some("runtime/hotpaths/call-chain.yaml not found".into())
        },
    });

    let _ = (workspace, target);
    checks
}

fn runtime_run_record_failures(ad: &Path) -> (usize, Vec<String>) {
    let runtime_dir = ad.join("runtime");
    let manifest_path = runtime_dir.join("run-manifest.yaml");
    let content = match fs::read_to_string(&manifest_path) {
        Ok(content) => content,
        Err(_) => return (0, vec!["runtime/run-manifest.yaml not readable".into()]),
    };
    let doc = match serde_yaml::from_str::<serde_yaml::Value>(&content) {
        Ok(doc) => doc,
        Err(err) => {
            return (
                0,
                vec![format!("runtime/run-manifest.yaml invalid YAML: {err}")],
            );
        }
    };
    let Some(records) = doc.get("run_records").and_then(|v| v.as_sequence()) else {
        return (
            0,
            vec!["runtime/run-manifest.yaml has missing or non-sequence run_records".into()],
        );
    };
    if records.is_empty() {
        return (
            0,
            vec!["runtime/run-manifest.yaml has empty run_records".into()],
        );
    }

    let mut failures = Vec::new();
    let mut count = 0usize;
    for entry in records {
        let Some(rel) = entry.as_str() else {
            failures.push("runtime run record path is not a string".into());
            continue;
        };
        if !is_safe_relative_yaml_path(rel) {
            failures.push(format!("{rel} is not a safe relative YAML path"));
            continue;
        }
        count += 1;
        let record_path = runtime_dir.join(rel);
        let record_content = match fs::read_to_string(&record_path) {
            Ok(content) => content,
            Err(_) => {
                failures.push(format!("missing run record {}", record_path.display()));
                continue;
            }
        };
        let record_doc = match serde_yaml::from_str::<serde_yaml::Value>(&record_content) {
            Ok(doc) => doc,
            Err(err) => {
                failures.push(format!("{} invalid YAML: {err}", record_path.display()));
                continue;
            }
        };
        let has_observations = record_doc
            .get("observations")
            .and_then(|v| v.as_sequence())
            .map(|seq| !seq.is_empty())
            .unwrap_or(false);
        if !has_observations {
            failures.push(format!("{} has no observations", record_path.display()));
        }
    }

    (count, failures)
}

fn is_safe_relative_yaml_path(raw: &str) -> bool {
    let path = Path::new(raw);
    !path.is_absolute()
        && path.extension().and_then(|e| e.to_str()) == Some("yaml")
        && path
            .components()
            .all(|component| matches!(component, std::path::Component::Normal(_)))
}

fn p2_checks(workspace: &Path, target: &str, ad: &Path) -> Vec<GateCheck> {
    let mut checks = Vec::new();

    let identified_path = ad.join("third-party").join("identified.yaml");
    let identified_ok = identified_path.exists() && validate_yaml_parseable(&identified_path);
    checks.push(GateCheck {
        id: "P2_01".into(),
        description: "third-party/identified.yaml exists and is parseable".into(),
        passed: identified_ok,
        detail: if identified_ok {
            None
        } else if identified_path.exists() {
            Some("third-party/identified.yaml is not valid YAML".into())
        } else {
            Some("third-party/identified.yaml not found".into())
        },
    });

    if identified_path.exists() {
        if let Ok(content) = fs::read_to_string(&identified_path) {
            if let Ok(doc) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                match doc.get("libraries").and_then(|v| v.as_sequence()) {
                    Some(libs) if libs.is_empty() => {
                        checks.push(GateCheck {
                            id: "P2_02".into(),
                            description: "third-party review records no identified libraries"
                                .into(),
                            passed: true,
                            detail: Some("libraries is empty; third-party checks waived".into()),
                        });
                        checks.push(GateCheck {
                            id: "P2_03".into(),
                            description:
                                "third-party: function_classification coverage > 0% (skipped)"
                                    .into(),
                            passed: true,
                            detail: Some("No third-party libraries recorded".into()),
                        });
                    }
                    Some(libs) => {
                        let high_conf = libs
                            .iter()
                            .filter(|lib| {
                                lib.get("confidence")
                                    .and_then(|c| c.as_str())
                                    .map(|c| c == "high" || c == "medium")
                                    .unwrap_or(false)
                            })
                            .count();
                        checks.push(GateCheck {
                            id: "P2_02".into(),
                            description: "third-party: >= 1 library with confidence >= medium"
                                .into(),
                            passed: high_conf >= 1,
                            detail: Some(format!(
                                "found {} libraries with confidence >= medium",
                                high_conf
                            )),
                        });

                        let total_classified: usize = libs
                            .iter()
                            .filter_map(|lib| {
                                lib.get("function_classifications")
                                    .and_then(|v| v.as_sequence())
                            })
                            .flatten()
                            .count();
                        let coverage_ok = total_classified > 0;
                        checks.push(GateCheck {
                            id: "P2_03".into(),
                            description: "third-party: function_classification coverage > 0%"
                                .into(),
                            passed: coverage_ok,
                            detail: Some(format!(
                                "{} functions classified across all libraries",
                                total_classified
                            )),
                        });
                    }
                    None => {
                        checks.push(GateCheck {
                            id: "P2_02".into(),
                            description: "third-party/identified.yaml has a libraries sequence"
                                .into(),
                            passed: false,
                            detail: Some("libraries field missing or not a sequence".into()),
                        });
                        checks.push(GateCheck {
                            id: "P2_03".into(),
                            description: "third-party: function_classification coverage > 0%"
                                .into(),
                            passed: false,
                            detail: Some("libraries field missing or not a sequence".into()),
                        });
                    }
                }
            } else {
                checks.push(GateCheck {
                    id: "P2_02".into(),
                    description: "third-party: >= 1 library with confidence >= medium".into(),
                    passed: false,
                    detail: Some("failed to parse identified.yaml".into()),
                });
                checks.push(GateCheck {
                    id: "P2_03".into(),
                    description: "third-party: function_classification coverage > 0%".into(),
                    passed: false,
                    detail: Some("failed to parse identified.yaml".into()),
                });
            }
        } else {
            checks.push(GateCheck {
                id: "P2_02".into(),
                description: "third-party: >= 1 library with confidence >= medium".into(),
                passed: false,
                detail: Some("failed to read identified.yaml".into()),
            });
            checks.push(GateCheck {
                id: "P2_03".into(),
                description: "third-party: function_classification coverage > 0%".into(),
                passed: false,
                detail: Some("failed to read identified.yaml".into()),
            });
        }
    } else {
        checks.push(GateCheck {
            id: "P2_02".into(),
            description: "third-party: >= 1 library with confidence >= medium".into(),
            passed: false,
            detail: Some("No third-party/identified.yaml".into()),
        });
        checks.push(GateCheck {
            id: "P2_03".into(),
            description: "third-party: function_classification coverage > 0%".into(),
            passed: false,
            detail: Some("No third-party/identified.yaml".into()),
        });
    }

    let pristine_failures = third_party_pristine_failures(workspace, target, ad);
    checks.push(GateCheck {
        id: "P2_04".into(),
        description: "third-party pristine source directories exist with source_path records"
            .into(),
        passed: pristine_failures.is_empty(),
        detail: if pristine_failures.is_empty() {
            None
        } else {
            Some(pristine_failures.join("; "))
        },
    });

    checks
}

fn third_party_pristine_failures(workspace: &Path, _target: &str, ad: &Path) -> Vec<String> {
    let identified_path = ad.join("third-party").join("identified.yaml");
    let content = match fs::read_to_string(&identified_path) {
        Ok(content) => content,
        Err(_) => return vec!["third-party/identified.yaml not readable".into()],
    };
    let doc = match serde_yaml::from_str::<serde_yaml::Value>(&content) {
        Ok(doc) => doc,
        Err(_) => return vec!["third-party/identified.yaml not parseable".into()],
    };
    let libs = match doc.get("libraries").and_then(|v| v.as_sequence()) {
        Some(libs) if !libs.is_empty() => libs,
        Some(_) => return vec![],
        None => return vec!["libraries field missing or not a sequence".into()],
    };

    let worktree = git_status::discover_worktree(workspace).ok().flatten();
    let mut failures = Vec::new();

    for lib in libs {
        let name = lib
            .get("library")
            .and_then(|v| v.as_str())
            .unwrap_or("<unknown>");
        if lib.get("source_path").and_then(|v| v.as_str()).is_none() {
            failures.push(format!("{name} missing source_path"));
        }
        let Some(pristine_path) = lib.get("pristine_path").and_then(|v| v.as_str()) else {
            failures.push(format!("{name} missing pristine_path"));
            continue;
        };
        let path = ad.join(pristine_path);
        if !path.is_dir() {
            failures.push(format!(
                "{name} pristine directory missing: {}",
                path.display()
            ));
            continue;
        }
        if let Some(worktree) = worktree.as_ref() {
            match directory_tracking_failures(worktree, &path) {
                Ok(mut inner) => failures.append(&mut inner),
                Err(err) => failures.push(format!("{}: {err}", path.display())),
            }
        }
    }

    failures
}

fn p3_checks(workspace: &Path, target: &str, ad: &Path) -> Vec<GateCheck> {
    let mut checks = Vec::new();

    let renames_path = ad.join("metadata").join("renames.yaml");
    checks.push(GateCheck {
        id: "P3_01".into(),
        description: "metadata/renames.yaml exists with non-empty renames".into(),
        passed: validate_yaml_sequence_nonempty(&renames_path, "renames"),
        detail: if renames_path.exists() {
            None
        } else {
            Some("metadata/renames.yaml not found".into())
        },
    });

    let signatures_path = ad.join("metadata").join("signatures.yaml");
    checks.push(GateCheck {
        id: "P3_02".into(),
        description: "metadata/signatures.yaml exists with non-empty signatures".into(),
        passed: validate_yaml_sequence_nonempty(&signatures_path, "signatures"),
        detail: if signatures_path.exists() {
            None
        } else {
            Some("metadata/signatures.yaml not found".into())
        },
    });

    let coverage_failures = hotpath_metadata_coverage_failures(ad);
    checks.push(GateCheck {
        id: "P3_03".into(),
        description: "P3 metadata covers every runtime hotpath function".into(),
        passed: coverage_failures.is_empty(),
        detail: if coverage_failures.is_empty() {
            None
        } else {
            Some(coverage_failures.join("; "))
        },
    });

    let _ = (workspace, target);
    checks
}

fn hotpath_metadata_coverage_failures(ad: &Path) -> Vec<String> {
    let hotpath_addrs = yaml_addr_set(
        &ad.join("runtime").join("hotpaths").join("call-chain.yaml"),
        "functions",
    );
    if hotpath_addrs.is_empty() {
        return vec!["runtime/hotpaths/call-chain.yaml has no functions".into()];
    }
    let rename_addrs = yaml_addr_set(&ad.join("metadata").join("renames.yaml"), "renames");
    let signature_addrs = yaml_addr_set(&ad.join("metadata").join("signatures.yaml"), "signatures");
    hotpath_addrs
        .into_iter()
        .filter_map(|addr| {
            if !rename_addrs.contains(&addr) {
                Some(format!("{addr} missing rename"))
            } else if !signature_addrs.contains(&addr) {
                Some(format!("{addr} missing signature"))
            } else {
                None
            }
        })
        .collect()
}

fn p4_checks(_workspace: &Path, _target: &str, ad: &Path) -> Vec<GateCheck> {
    let mut checks = Vec::new();

    let functions_dir = ad.join("substitution").join("functions");
    let substitution_records = substitution_record_paths(&functions_dir);
    checks.push(GateCheck {
        id: "P4_01".into(),
        description: "substitution/functions contains function substitution records".into(),
        passed: !substitution_records.is_empty(),
        detail: Some(format!(
            "{} substitution records found",
            substitution_records.len()
        )),
    });

    let fixture_failures = substitution_fixture_failures(&substitution_records);
    checks.push(GateCheck {
        id: "P4_02".into(),
        description: "each substitution record has at least one fixture".into(),
        passed: fixture_failures.is_empty(),
        detail: if fixture_failures.is_empty() {
            None
        } else {
            Some(fixture_failures.join("; "))
        },
    });

    let metadata_failures = substitution_metadata_failures(ad, &substitution_records);
    checks.push(GateCheck {
        id: "P4_03".into(),
        description: "substitutions reference P3 functions with names and signatures".into(),
        passed: metadata_failures.is_empty(),
        detail: if metadata_failures.is_empty() {
            None
        } else {
            Some(metadata_failures.join("; "))
        },
    });

    checks
}

fn substitution_record_paths(functions_dir: &Path) -> Vec<std::path::PathBuf> {
    if !functions_dir.exists() {
        return vec![];
    }
    let mut paths: Vec<_> = std::fs::read_dir(functions_dir)
        .map(|entries| {
            entries
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.path().join("substitution.yaml"))
                .filter(|path| path.exists())
                .collect()
        })
        .unwrap_or_default();
    paths.sort();
    paths
}

fn substitution_fixture_failures(records: &[std::path::PathBuf]) -> Vec<String> {
    records
        .iter()
        .filter_map(|path| {
            let content = fs::read_to_string(path).ok()?;
            let doc = serde_yaml::from_str::<serde_yaml::Value>(&content).ok()?;
            let ok = doc
                .get("fixtures")
                .and_then(|v| v.as_sequence())
                .map(|seq| !seq.is_empty())
                .unwrap_or(false);
            if ok {
                None
            } else {
                Some(format!("{} has no fixtures", path.display()))
            }
        })
        .collect()
}

fn substitution_metadata_failures(ad: &Path, records: &[std::path::PathBuf]) -> Vec<String> {
    let rename_addrs = yaml_addr_set(&ad.join("metadata").join("renames.yaml"), "renames");
    let signature_addrs = yaml_addr_set(&ad.join("metadata").join("signatures.yaml"), "signatures");
    let mut failures = Vec::new();
    for path in records {
        let content = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(err) => {
                failures.push(format!("{} unreadable: {err}", path.display()));
                continue;
            }
        };
        let doc = match serde_yaml::from_str::<serde_yaml::Value>(&content) {
            Ok(doc) => doc,
            Err(err) => {
                failures.push(format!("{} invalid YAML: {err}", path.display()));
                continue;
            }
        };
        let Some(addr) = doc.get("addr").and_then(|v| v.as_str()) else {
            failures.push(format!("{} missing addr", path.display()));
            continue;
        };
        if !rename_addrs.contains(addr) {
            failures.push(format!("{addr} missing metadata rename"));
        }
        if !signature_addrs.contains(addr) {
            failures.push(format!("{addr} missing metadata signature"));
        }
    }
    failures
}

fn yaml_addr_set(path: &Path, sequence: &str) -> std::collections::HashSet<String> {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(_) => return std::collections::HashSet::new(),
    };
    let doc = match serde_yaml::from_str::<serde_yaml::Value>(&content) {
        Ok(doc) => doc,
        Err(_) => return std::collections::HashSet::new(),
    };
    doc.get(sequence)
        .and_then(|v| v.as_sequence())
        .map(|seq| {
            seq.iter()
                .filter_map(|entry| {
                    entry
                        .get("addr")
                        .and_then(|addr| addr.as_str())
                        .map(|addr| addr.to_string())
                })
                .collect()
        })
        .unwrap_or_default()
}
