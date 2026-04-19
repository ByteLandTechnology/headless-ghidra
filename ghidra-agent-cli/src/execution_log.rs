use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::schema::{load_yaml, save_yaml};
use crate::workspace::artifact_dir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub script: String,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inputs_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outputs: Option<Vec<String>>,
    #[serde(default)]
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionLogYaml {
    pub target: String,
    pub entries: Vec<LogEntry>,
}

pub fn load_execution_log(workspace: &Path, target: &str) -> Result<ExecutionLogYaml> {
    let path = artifact_dir(workspace, target).join("execution-log.yaml");
    if !path.exists() {
        return Ok(ExecutionLogYaml {
            target: target.to_string(),
            entries: vec![],
        });
    }
    load_yaml(&path)
}

pub fn save_execution_log(workspace: &Path, target: &str, data: &ExecutionLogYaml) -> Result<()> {
    save_yaml(
        &artifact_dir(workspace, target).join("execution-log.yaml"),
        data,
    )
}

pub fn append_entry(workspace: &Path, target: &str, entry: LogEntry) -> Result<()> {
    let mut log = load_execution_log(workspace, target)?;
    log.entries.push(entry);
    save_execution_log(workspace, target, &log)
}
