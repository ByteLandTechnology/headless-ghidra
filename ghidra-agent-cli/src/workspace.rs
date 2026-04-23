use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::schema::{load_yaml, save_yaml};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineState {
    pub target: String,
    pub phase: String,
    pub binary: Option<String>,
    #[serde(default)]
    pub created_at: String,
}

pub fn target_dir(workspace: &Path, target: &str) -> PathBuf {
    workspace.join("targets").join(target)
}

pub fn artifact_dir(workspace: &Path, target: &str) -> PathBuf {
    workspace.join("artifacts").join(target)
}

pub fn detect_workspace(explicit: Option<&Path>) -> Result<PathBuf> {
    if let Some(p) = explicit {
        return Ok(p.to_path_buf());
    }
    let cwd = std::env::current_dir()?;
    let mut dir = cwd.as_path();
    loop {
        if dir.join("targets").is_dir() || dir.join(".git").exists() {
            return Ok(dir.to_path_buf());
        }
        dir = dir
            .parent()
            .ok_or_else(|| anyhow!("cannot detect workspace root"))?;
    }
}

pub fn init_target(workspace: &Path, target: &str, binary: &Path) -> Result<()> {
    // targets/<id>/ — config + Ghidra project (committed + gitignored subdirs)
    let td = target_dir(workspace, target);
    std::fs::create_dir_all(&td)?;
    std::fs::create_dir_all(td.join("ghidra-projects"))?;

    // artifacts/<id>/ — all analysis data (committed)
    let ad = artifact_dir(workspace, target);
    std::fs::create_dir_all(ad.join("baseline"))?;
    std::fs::create_dir_all(ad.join("runtime").join("project"))?;
    std::fs::create_dir_all(ad.join("runtime").join("fixtures"))?;
    std::fs::create_dir_all(ad.join("runtime").join("run-records"))?;
    std::fs::create_dir_all(ad.join("runtime").join("hotpaths"))?;
    std::fs::create_dir_all(ad.join("third-party").join("pristine"))?;
    std::fs::create_dir_all(ad.join("third-party").join("compat"))?;
    std::fs::create_dir_all(ad.join("metadata").join("apply-records"))?;
    std::fs::create_dir_all(ad.join("substitution").join("template"))?;
    std::fs::create_dir_all(ad.join("substitution").join("functions"))?;
    std::fs::create_dir_all(ad.join("decompilation").join("functions"))?;
    std::fs::create_dir_all(ad.join("gates"))?;
    std::fs::create_dir_all(ad.join("scripts"))?;
    std::fs::create_dir_all(ad.join("intake"))?;

    let state = PipelineState {
        target: target.to_string(),
        phase: "P0".to_string(),
        binary: Some(binary.display().to_string()),
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    save_yaml(&ad.join("pipeline-state.yaml"), &state)?;

    let scope = crate::scope::ScopeYaml {
        target: target.to_string(),
        mode: "full".to_string(),
        entries: vec![],
        note: None,
    };
    save_yaml(&ad.join("scope.yaml"), &scope)?;

    Ok(())
}

pub fn load_pipeline_state(workspace: &Path, target: &str) -> Result<PipelineState> {
    load_yaml(&artifact_dir(workspace, target).join("pipeline-state.yaml"))
}

pub fn set_phase(workspace: &Path, target: &str, phase: &str) -> Result<()> {
    let path = artifact_dir(workspace, target).join("pipeline-state.yaml");
    let mut state: PipelineState = load_yaml(&path)?;
    state.phase = phase.to_string();
    save_yaml(&path, &state)?;
    Ok(())
}
