use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::schema::{load_yaml, save_yaml};
use crate::workspace::artifact_dir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressEntry {
    pub fn_id: String,
    pub addr: String,
    pub state: String,
    pub backend: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decompiled_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressYaml {
    pub target: String,
    pub functions: Vec<ProgressEntry>,
}

pub fn load_progress(workspace: &Path, target: &str) -> Result<ProgressYaml> {
    let path = artifact_dir(workspace, target)
        .join("decompilation")
        .join("progress.yaml");
    if !path.exists() {
        return Ok(ProgressYaml {
            target: target.to_string(),
            functions: vec![],
        });
    }
    load_yaml(&path)
}

pub fn save_progress(workspace: &Path, target: &str, data: &ProgressYaml) -> Result<()> {
    save_yaml(
        &artifact_dir(workspace, target)
            .join("decompilation")
            .join("progress.yaml"),
        data,
    )
}

pub fn mark_function_decompiled(
    workspace: &Path,
    target: &str,
    fn_id: &str,
    addr: &str,
    backend: &str,
) -> Result<ProgressEntry> {
    let mut progress = load_progress(workspace, target)?;
    let entry = ProgressEntry {
        fn_id: fn_id.to_string(),
        addr: addr.to_string(),
        state: "decompiled".to_string(),
        backend: backend.to_string(),
        verification: None,
        decompiled_at: Some(chrono::Utc::now().to_rfc3339()),
    };

    if let Some(existing) = progress
        .functions
        .iter_mut()
        .find(|f| f.fn_id == fn_id || f.addr == addr)
    {
        *existing = entry.clone();
    } else {
        progress.functions.push(entry.clone());
    }

    save_progress(workspace, target, &progress)?;
    Ok(entry)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NextBatchYaml {
    pub target: String,
    pub strategy: String,
    pub batch: Vec<BatchEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchEntry {
    pub fn_id: String,
    pub addr: String,
    pub reason: String,
}

pub fn compute_next_batch(
    workspace: &Path,
    target: &str,
    max: usize,
    strategy: &str,
) -> Result<NextBatchYaml> {
    let progress = load_progress(workspace, target)?;
    let scope = crate::scope::load_scope(workspace, target)?;
    let callgraph = crate::baseline::load_callgraph(workspace, target)?;
    let baseline = crate::baseline::load_functions(workspace, target).ok();

    let done_addrs: std::collections::HashSet<String> =
        progress.functions.iter().map(|f| f.addr.clone()).collect();

    let remaining: Vec<&String> = scope
        .entries
        .iter()
        .filter(|e| !done_addrs.contains(e.as_str()))
        .collect();

    let mut candidates: Vec<&String> = match strategy {
        "breadth-first" => remaining,
        "callgraph-leaves" => {
            let callees: std::collections::HashSet<&str> =
                callgraph.edges.iter().map(|e| e.from.as_str()).collect();
            remaining
                .into_iter()
                .filter(|addr| !callees.contains(addr.as_str()))
                .collect()
        }
        "callgraph-roots" => {
            let callers: std::collections::HashSet<&str> =
                callgraph.edges.iter().map(|e| e.to.as_str()).collect();
            remaining
                .into_iter()
                .filter(|addr| !callers.contains(addr.as_str()))
                .collect()
        }
        "size-ascending" => {
            if let Some(ref b) = baseline {
                let mut with_size: Vec<(&String, u64)> = remaining
                    .iter()
                    .map(|addr| {
                        let size = b
                            .functions
                            .iter()
                            .find(|f| &f.addr == *addr)
                            .map(|f| f.size)
                            .unwrap_or(0);
                        (*addr, size)
                    })
                    .collect();
                with_size.sort_by_key(|(_, s)| *s);
                with_size.into_iter().map(|(a, _)| a).collect()
            } else {
                remaining
            }
        }
        _ => remaining,
    };

    candidates.truncate(max);

    // Build addr -> fn_id map from already-assigned functions
    let existing_ids: std::collections::HashMap<&str, &str> = progress
        .functions
        .iter()
        .map(|f| (f.addr.as_str(), f.fn_id.as_str()))
        .collect();

    // Assign IDs: reuse existing for already-assigned addrs, assign new for fresh ones
    let mut next_new_id = progress.functions.len() + 1;
    let batch: Vec<BatchEntry> = candidates
        .into_iter()
        .map(|addr| {
            let fn_id = if let Some(existing) = existing_ids.get(addr.as_str()) {
                existing.to_string()
            } else {
                let id = format!("fn_{:03}", next_new_id);
                next_new_id += 1;
                id
            };
            BatchEntry {
                fn_id,
                addr: addr.to_string(),
                reason: format!("{strategy} selection, not yet decompiled"),
            }
        })
        .collect();

    Ok(NextBatchYaml {
        target: target.to_string(),
        strategy: strategy.to_string(),
        batch,
    })
}

pub fn save_next_batch(workspace: &Path, target: &str, data: &NextBatchYaml) -> Result<()> {
    save_yaml(
        &artifact_dir(workspace, target)
            .join("decompilation")
            .join("next-batch.yaml"),
        data,
    )
}

pub fn load_next_batch(workspace: &Path, target: &str) -> Result<NextBatchYaml> {
    let path = artifact_dir(workspace, target)
        .join("decompilation")
        .join("next-batch.yaml");
    if !path.exists() {
        return Err(anyhow!("missing next batch file at {}", path.display()));
    }

    let batch: NextBatchYaml =
        load_yaml(&path).with_context(|| format!("invalid next batch file {}", path.display()))?;

    if batch.batch.is_empty() {
        return Err(anyhow!(
            "next batch file {} contains no batch entries",
            path.display()
        ));
    }
    if batch.target != target {
        return Err(anyhow!(
            "next batch target mismatch: expected '{}', found '{}'",
            target,
            batch.target
        ));
    }

    Ok(batch)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn mark_function_decompiled_updates_existing_entry() {
        let tmp = TempDir::new().expect("tempdir");
        let workspace = tmp.path();
        let target = "sample";

        save_progress(
            workspace,
            target,
            &ProgressYaml {
                target: target.to_string(),
                functions: vec![ProgressEntry {
                    fn_id: "fn_001".to_string(),
                    addr: "0x1000".to_string(),
                    state: "queued".to_string(),
                    backend: "manual".to_string(),
                    verification: Some("pending".to_string()),
                    decompiled_at: None,
                }],
            },
        )
        .expect("seed progress");

        let updated = mark_function_decompiled(workspace, target, "fn_001", "0x1000", "ghidra")
            .expect("mark decompiled");
        let reloaded = load_progress(workspace, target).expect("reload progress");

        assert_eq!(updated.state, "decompiled");
        assert_eq!(reloaded.functions.len(), 1);
        assert_eq!(reloaded.functions[0].fn_id, "fn_001");
        assert_eq!(reloaded.functions[0].addr, "0x1000");
        assert_eq!(reloaded.functions[0].backend, "ghidra");
        assert_eq!(reloaded.functions[0].state, "decompiled");
        assert!(reloaded.functions[0].verification.is_none());
        assert!(reloaded.functions[0].decompiled_at.is_some());
    }
}
