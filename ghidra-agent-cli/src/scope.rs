use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::schema::{load_yaml, save_yaml};
use crate::workspace::artifact_dir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeYaml {
    pub target: String,
    pub mode: String,
    #[serde(default)]
    pub entries: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

pub fn load_scope(workspace: &Path, target: &str) -> Result<ScopeYaml> {
    let path = artifact_dir(workspace, target).join("scope.yaml");
    if !path.exists() {
        return Ok(ScopeYaml {
            target: target.to_string(),
            mode: "full".to_string(),
            entries: vec![],
            note: None,
        });
    }
    load_yaml(&path)
}

pub fn set_scope(
    workspace: &Path,
    target: &str,
    mode: &str,
    entries: Vec<String>,
    note: Option<String>,
) -> Result<()> {
    let scope = ScopeYaml {
        target: target.to_string(),
        mode: mode.to_string(),
        entries,
        note,
    };
    save_yaml(&artifact_dir(workspace, target).join("scope.yaml"), &scope)
}

pub fn add_entry(workspace: &Path, target: &str, entry: &str) -> Result<()> {
    let mut scope = load_scope(workspace, target)?;
    if !scope.entries.contains(&entry.to_string()) {
        scope.entries.push(entry.to_string());
    }
    save_yaml(&artifact_dir(workspace, target).join("scope.yaml"), &scope)
}

pub fn remove_entry(workspace: &Path, target: &str, entry: &str) -> Result<()> {
    let mut scope = load_scope(workspace, target)?;
    scope.entries.retain(|e| e != entry);
    save_yaml(&artifact_dir(workspace, target).join("scope.yaml"), &scope)
}
