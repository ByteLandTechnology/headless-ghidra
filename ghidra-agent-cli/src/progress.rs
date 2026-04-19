use anyhow::Result;
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
