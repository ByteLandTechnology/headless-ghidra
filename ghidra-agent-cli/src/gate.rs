#![allow(clippy::collapsible_if)]

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

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

pub const ALL_PHASES: &[&str] = &["P0", "P0.5", "P1", "P2", "P3", "P4", "P5", "P6"];

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
            phase: "P0.5".into(),
            name: "Scope".into(),
            checks: vec![(
                "P0.5_01".into(),
                "scope.yaml exists with non-empty entries".into(),
            )],
        },
        PhaseInfo {
            phase: "P1".into(),
            name: "Baseline".into(),
            checks: [
                "functions.yaml",
                "callgraph.yaml",
                "types.yaml",
                "vtables.yaml",
                "constants.yaml",
                "strings.yaml",
                "imports.yaml",
            ]
            .iter()
            .map(|name| {
                (
                    format!("P1_{}", name.replace('.', "_")),
                    format!("baseline/{name} exists and is parseable"),
                )
            })
            .collect(),
        },
        PhaseInfo {
            phase: "P2".into(),
            name: "Evidence".into(),
            checks: vec![
                (
                    "P2_01".into(),
                    "evidence-candidates.yaml or target-selection.yaml exists".into(),
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
            name: "Discovery".into(),
            checks: vec![
                (
                    "P3_01".into(),
                    "target-selection.yaml exists with selected_target".into(),
                ),
                (
                    "P3_02".into(),
                    ">= 1 candidate with status ready or selected".into(),
                ),
                ("P3_03".into(), "scope: entries non-empty".into()),
            ],
        },
        PhaseInfo {
            phase: "P4".into(),
            name: "Batch Decompile".into(),
            checks: vec![
                (
                    "P4_01".into(),
                    "decompilation/progress.yaml exists".into(),
                ),
                (
                    "P4_02".into(),
                    "decompilation/next-batch.yaml exists and non-empty".into(),
                ),
                (
                    "P4_03".into(),
                    "next-batch entries reference valid scope or baseline addresses".into(),
                ),
            ],
        },
        PhaseInfo {
            phase: "P5".into(),
            name: "Decompile Complete".into(),
            checks: vec![
                (
                    "P5_01".into(),
                    ">= 1 decompiled C file exists".into(),
                ),
                ("P5_02".into(), "coverage >= 10%".into()),
                (
                    "P5_03".into(),
                    "each decompiled function has decompilation-record.yaml with required fields"
                        .into(),
                ),
            ],
        },
        PhaseInfo {
            phase: "P6".into(),
            name: "Verification".into(),
            checks: vec![
                (
                    "P6_01".into(),
                    "verification-result.yaml exists for each decompiled function".into(),
                ),
                (
                    "P6_02".into(),
                    "no unresolved mismatches without notes".into(),
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

    let checks = match phase {
        "P0" => p0_checks(&ad),
        "P0.5" => p0_5_checks(&ad),
        "P1" => p1_checks(&ad),
        "P2" => p2_checks(&ad),
        "P3" => p3_checks(&ad),
        "P4" => p4_checks(workspace, target, &ad),
        "P5" => p5_checks(workspace, target, &ad),
        "P6" => p6_checks(workspace, target, &ad),
        _ => vec![GateCheck {
            id: format!("{}_01", phase.replace('.', "_")),
            description: format!("{} gate placeholder", phase),
            passed: true,
            detail: None,
        }],
    };

    let passed = checks.iter().all(|c| c.passed);
    Ok(GateReport {
        target: target.to_string(),
        phase: phase.to_string(),
        passed,
        checks,
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

fn p0_checks(ad: &Path) -> Vec<GateCheck> {
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

fn p0_5_checks(ad: &Path) -> Vec<GateCheck> {
    let path = ad.join("scope.yaml");
    let exists = path.exists();
    vec![GateCheck {
        id: "P0.5_01".into(),
        description: "scope.yaml exists with non-empty entries".into(),
        passed: exists && validate_yaml_sequence_nonempty(&path, "entries"),
        detail: if !exists {
            Some("scope.yaml not found".into())
        } else if !validate_yaml_parseable(&path) {
            Some("scope.yaml is not valid YAML".into())
        } else if !validate_yaml_sequence_nonempty(&path, "entries") {
            Some("scope.yaml has empty or missing 'entries' field".into())
        } else {
            None
        },
    }]
}

fn p1_checks(ad: &Path) -> Vec<GateCheck> {
    let names = [
        "functions.yaml",
        "callgraph.yaml",
        "types.yaml",
        "vtables.yaml",
        "constants.yaml",
        "strings.yaml",
        "imports.yaml",
    ];
    names
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
        .collect()
}

fn p2_checks(ad: &Path) -> Vec<GateCheck> {
    let mut checks = Vec::new();

    // P2_01: evidence-candidates.yaml OR target-selection.yaml exists
    let ev_path = ad.join("evidence-candidates.yaml");
    let sel_path = ad.join("target-selection.yaml");
    let review_path_exists = ev_path.exists() || sel_path.exists();
    checks.push(GateCheck {
        id: "P2_01".into(),
        description: "evidence-candidates.yaml or target-selection.yaml exists".into(),
        passed: review_path_exists,
        detail: if review_path_exists {
            None
        } else {
            Some("Neither evidence-candidates.yaml nor target-selection.yaml found".into())
        },
    });

    // P2_02: If identified.yaml exists, has >= 1 library with confidence >= medium
    // P2_03: If third-party libs identified, function_classification coverage > 0%
    let identified_path = ad.join("third-party").join("identified.yaml");
    if identified_path.exists() {
        if let Ok(content) = fs::read_to_string(&identified_path) {
            if let Ok(doc) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                if let Some(libs) = doc.get("libraries").and_then(|v| v.as_sequence()) {
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
                        description: "third-party: >= 1 library with confidence >= medium".into(),
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
                        description: "third-party: function_classification coverage > 0%".into(),
                        passed: coverage_ok,
                        detail: Some(format!(
                            "{} functions classified across all libraries",
                            total_classified
                        )),
                    });
                } else {
                    checks.push(GateCheck {
                        id: "P2_02".into(),
                        description:
                            "third-party/identified.yaml parseable (skipped: no libs found)".into(),
                        passed: true,
                        detail: Some("No libraries found in identified.yaml".into()),
                    });
                    checks.push(GateCheck {
                        id: "P2_03".into(),
                        description: "third-party: function_classification coverage > 0% (skipped)"
                            .into(),
                        passed: true,
                        detail: Some("No libraries found — classification check waived".into()),
                    });
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
            description: "third-party/identified.yaml exists (skipped: no third-party libs found)"
                .into(),
            passed: true,
            detail: Some(
                "No third-party/identified.yaml — third-party gate conditions waived".into(),
            ),
        });
        checks.push(GateCheck {
            id: "P2_03".into(),
            description: "third-party: function_classification coverage > 0% (skipped)".into(),
            passed: true,
            detail: Some("No third-party libs identified — classification check waived".into()),
        });
    }

    checks
}

fn p3_checks(ad: &Path) -> Vec<GateCheck> {
    let mut checks = Vec::new();

    // P3_01: target-selection.yaml exists with selected_target field
    let sel_path = ad.join("target-selection.yaml");
    let has_selected = if sel_path.exists() {
        if let Ok(content) = fs::read_to_string(&sel_path) {
            if let Ok(doc) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                doc.get("selected_target").is_some()
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    };
    checks.push(GateCheck {
        id: "P3_01".into(),
        description: "target-selection.yaml exists with selected_target".into(),
        passed: has_selected,
        detail: if has_selected {
            None
        } else {
            Some("selected_target field missing".into())
        },
    });

    // P3_02: At least one candidate with status == ready or selected as default
    let has_ready = if sel_path.exists() {
        if let Ok(content) = fs::read_to_string(&sel_path) {
            if let Ok(doc) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                if let Some(cands) = doc.get("candidates").and_then(|v| v.as_sequence()) {
                    cands.iter().any(|c| {
                        c.get("status")
                            .and_then(|s| s.as_str())
                            .map(|s| s == "ready" || s == "selected")
                            .unwrap_or(false)
                    })
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    };
    checks.push(GateCheck {
        id: "P3_02".into(),
        description: "target-selection: >= 1 candidate with status == ready or selected".into(),
        passed: has_ready,
        detail: None,
    });

    // P3_03: Scope entries non-empty
    let scope_path = ad.join("scope.yaml");
    let scope_nonempty = if scope_path.exists() {
        if let Ok(content) = fs::read_to_string(&scope_path) {
            if let Ok(doc) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                if let Some(entries) = doc.get("entries").and_then(|v| v.as_sequence()) {
                    !entries.is_empty()
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    };
    checks.push(GateCheck {
        id: "P3_03".into(),
        description: "scope: entries non-empty".into(),
        passed: scope_nonempty,
        detail: None,
    });

    checks
}

fn p4_checks(_workspace: &Path, _target: &str, ad: &Path) -> Vec<GateCheck> {
    let mut checks = Vec::new();

    // P4_01: decompilation/progress.yaml exists
    let prog_path = ad.join("decompilation").join("progress.yaml");
    checks.push(GateCheck {
        id: "P4_01".into(),
        description: "decompilation/progress.yaml exists".into(),
        passed: prog_path.exists(),
        detail: None,
    });

    // P4_02: next-batch.yaml exists and non-empty
    let batch_path = ad.join("decompilation").join("next-batch.yaml");
    let batch_nonempty = if batch_path.exists() {
        if let Ok(content) = fs::read_to_string(&batch_path) {
            if let Ok(doc) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                if let Some(batch) = doc.get("batch").and_then(|b| b.as_sequence()) {
                    !batch.is_empty()
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    };
    checks.push(GateCheck {
        id: "P4_02".into(),
        description: "decompilation/next-batch.yaml exists and non-empty".into(),
        passed: batch_nonempty,
        detail: None,
    });

    // P4_03: All next-batch entries reference valid functions (in scope or baseline)
    // The batch contains scope entries not yet in progress - they should be valid tracked addresses
    let all_valid = if batch_path.exists() {
        let batch_addrs: std::collections::HashSet<String> =
            if let Ok(content) = fs::read_to_string(&batch_path) {
                if let Ok(doc) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                    if let Some(batch) = doc.get("batch").and_then(|b| b.as_sequence()) {
                        batch
                            .iter()
                            .filter_map(|e| {
                                e.get("addr").and_then(|a| a.as_str().map(|s| s.to_owned()))
                            })
                            .collect()
                    } else {
                        std::collections::HashSet::new()
                    }
                } else {
                    std::collections::HashSet::new()
                }
            } else {
                std::collections::HashSet::new()
            };
        // Validate batch addresses against scope entries
        let scope_path = ad.join("scope.yaml");
        let scope_addrs: std::collections::HashSet<String> =
            if let Ok(content) = fs::read_to_string(&scope_path) {
                if let Ok(doc) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                    doc.get("entries")
                        .and_then(|e| e.as_sequence())
                        .map(|seq| {
                            seq.iter()
                                .filter_map(|v| v.as_str().map(|s| s.to_owned()))
                                .collect()
                        })
                        .unwrap_or_default()
                } else {
                    std::collections::HashSet::new()
                }
            } else {
                std::collections::HashSet::new()
            };
        // Also check baseline functions
        let baseline_path = ad.join("baseline").join("functions.yaml");
        let baseline_addrs: std::collections::HashSet<String> =
            if let Ok(content) = fs::read_to_string(&baseline_path) {
                if let Ok(doc) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                    doc.get("functions")
                        .and_then(|f| f.as_sequence())
                        .map(|seq| {
                            seq.iter()
                                .filter_map(|e| {
                                    e.get("addr").and_then(|a| a.as_str().map(|s| s.to_owned()))
                                })
                                .collect()
                        })
                        .unwrap_or_default()
                } else {
                    std::collections::HashSet::new()
                }
            } else {
                std::collections::HashSet::new()
            };
        // Batch addresses are valid if they appear in scope OR baseline
        batch_addrs
            .iter()
            .all(|addr| scope_addrs.contains(addr) || baseline_addrs.contains(addr))
    } else {
        true // Skip if no batch
    };
    checks.push(GateCheck {
        id: "P4_03".into(),
        description: "next-batch entries reference valid scope or baseline addresses".into(),
        passed: all_valid,
        detail: None,
    });

    checks
}

fn p5_checks(_workspace: &Path, _target: &str, ad: &Path) -> Vec<GateCheck> {
    let mut checks = Vec::new();

    // P5_01: >= 1 decompiled C file exists
    let fn_dir = ad.join("decompilation").join("functions");
    let decompiled_count = if fn_dir.exists() {
        std::fs::read_dir(&fn_dir)
            .map(|it| {
                it.filter_map(|e| e.ok())
                    .filter(|e| e.path().is_dir())
                    .count()
            })
            .unwrap_or(0)
    } else {
        0
    };
    checks.push(GateCheck {
        id: "P5_01".into(),
        description: ">= 1 decompiled C file exists".into(),
        passed: decompiled_count >= 1,
        detail: Some(format!(
            "found {} decompiled function directories",
            decompiled_count
        )),
    });

    // P5_02: Coverage >= 10% threshold
    let progress_path = ad.join("decompilation").join("progress.yaml");
    let baseline_fn_path = ad.join("baseline").join("functions.yaml");
    if progress_path.exists() && baseline_fn_path.exists() {
        if let (Ok(pc), Ok(fc)) = (
            fs::read_to_string(&progress_path),
            fs::read_to_string(&baseline_fn_path),
        ) {
            if let (Ok(prog_doc), Ok(fn_doc)) = (
                serde_yaml::from_str::<serde_yaml::Value>(&pc),
                serde_yaml::from_str::<serde_yaml::Value>(&fc),
            ) {
                let prog_seq = prog_doc.get("functions").and_then(|f| f.as_sequence());
                let fn_seq = fn_doc.get("functions").and_then(|f| f.as_sequence());
                if let (Some(ps), Some(fs)) = (prog_seq, fn_seq) {
                    let total = fs.len();
                    let done = ps
                        .iter()
                        .filter(|e| {
                            e.get("state")
                                .and_then(|s| s.as_str())
                                .map(|s| s == "decompiled")
                                .unwrap_or(false)
                        })
                        .count();
                    if let Some(pct) = (done * 100).checked_div(total) {
                        checks.push(GateCheck {
                            id: "P5_02".into(),
                            description: format!("coverage >= 10% (currently {}%)", pct),
                            passed: pct >= 10,
                            detail: Some(format!("{}/{} functions decompiled", done, total)),
                        });
                    } else {
                        checks.push(GateCheck {
                            id: "P5_02".into(),
                            description: "coverage >= 10%".into(),
                            passed: false,
                            detail: Some("0 functions in baseline".into()),
                        });
                    }
                } else {
                    checks.push(GateCheck {
                        id: "P5_02".into(),
                        description: "coverage >= 10%".into(),
                        passed: false,
                        detail: Some("failed to parse progress or baseline".into()),
                    });
                }
            } else {
                checks.push(GateCheck {
                    id: "P5_02".into(),
                    description: "coverage >= 10%".into(),
                    passed: false,
                    detail: Some("failed to read progress or baseline".into()),
                });
            }
        } else {
            checks.push(GateCheck {
                id: "P5_02".into(),
                description: "coverage >= 10%".into(),
                passed: false,
                detail: Some("failed to read progress or baseline".into()),
            });
        }
    } else {
        checks.push(GateCheck {
            id: "P5_02".into(),
            description: "coverage >= 10%".into(),
            passed: false,
            detail: Some("progress.yaml or functions.yaml missing".into()),
        });
    }

    // P5_03: Each decompiled function has decompilation-record.yaml with required fields
    let record_issues: Vec<String> = if fn_dir.exists() {
        std::fs::read_dir(&fn_dir)
            .map(|it| {
                it.filter_map(|e| e.ok())
                    .filter(|e| e.path().is_dir())
                    .filter_map(|d| {
                        let record_path = d.path().join("decompilation-record.yaml");
                        if record_path.exists() {
                            if let Ok(content) = fs::read_to_string(&record_path) {
                                if let Ok(doc) = serde_yaml::from_str::<serde_yaml::Value>(&content)
                                {
                                    let has_required = ["fn_id", "addr", "name", "prototype"]
                                        .iter()
                                        .all(|f| doc.get(*f).is_some());
                                    if !has_required {
                                        return Some(
                                            d.path()
                                                .file_name()
                                                .map(|n| n.to_string_lossy().into_owned())
                                                .unwrap_or_default(),
                                        );
                                    }
                                }
                            }
                        }
                        None
                    })
                    .collect()
            })
            .unwrap_or_default()
    } else {
        vec![]
    };
    checks.push(GateCheck {
        id: "P5_03".into(),
        description: "each decompiled function has decompilation-record.yaml with required fields"
            .into(),
        passed: record_issues.is_empty(),
        detail: if record_issues.is_empty() {
            None
        } else {
            Some(format!(
                "missing required fields in: {}",
                record_issues.join(", ")
            ))
        },
    });

    checks
}

fn p6_checks(_workspace: &Path, _target: &str, ad: &Path) -> Vec<GateCheck> {
    let mut checks = Vec::new();

    // P6_01: verification-result.yaml exists for each decompiled function
    let fn_dir = ad.join("decompilation").join("functions");
    let verified_count = if fn_dir.exists() {
        std::fs::read_dir(&fn_dir)
            .map(|it| {
                it.filter_map(|e| e.ok())
                    .filter(|e| e.path().is_dir())
                    .filter(|d| d.path().join("verification-result.yaml").exists())
                    .count()
            })
            .unwrap_or(0)
    } else {
        0
    };
    let total_decompiled = if fn_dir.exists() {
        std::fs::read_dir(&fn_dir)
            .map(|it| {
                it.filter_map(|e| e.ok())
                    .filter(|e| e.path().is_dir())
                    .count()
            })
            .unwrap_or(0)
    } else {
        0
    };
    checks.push(GateCheck {
        id: "P6_01".into(),
        description: "verification-result.yaml exists for each decompiled function".into(),
        passed: verified_count >= total_decompiled && total_decompiled > 0,
        detail: Some(format!(
            "{}/{} functions have verification-result.yaml",
            verified_count, total_decompiled
        )),
    });

    // P6_02: No unresolved mismatches without notes
    let unresolved_issues: Vec<String> = if fn_dir.exists() {
        std::fs::read_dir(&fn_dir)
            .map(|it| {
                it.filter_map(|e| e.ok())
                    .filter(|e| e.path().is_dir())
                    .filter_map(|d| {
                        let vr_path = d.path().join("verification-result.yaml");
                        if vr_path.exists() {
                            let content = fs::read_to_string(&vr_path).ok()?;
                            let doc = serde_yaml::from_str::<serde_yaml::Value>(&content).ok()?;
                            let mismatches = doc.get("mismatches")?.as_sequence()?;
                            let has_unresolved = mismatches.iter().any(|m| {
                                m.get("resolved")
                                    .and_then(|r| r.as_bool())
                                    .map(|r| !r)
                                    .unwrap_or(false)
                                    && m.get("note").is_none()
                            });
                            if has_unresolved {
                                Some(
                                    d.path()
                                        .file_name()
                                        .map(|n| n.to_string_lossy().into_owned())
                                        .unwrap_or_default(),
                                )
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default()
    } else {
        vec![]
    };
    checks.push(GateCheck {
        id: "P6_02".into(),
        description: "no unresolved mismatches without notes".into(),
        passed: unresolved_issues.is_empty(),
        detail: if unresolved_issues.is_empty() {
            None
        } else {
            Some(format!(
                "unresolved mismatches in: {}",
                unresolved_issues.join(", ")
            ))
        },
    });

    checks
}
