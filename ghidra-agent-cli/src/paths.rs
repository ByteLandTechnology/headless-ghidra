use anyhow::Result;
use directories::ProjectDirs;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeDirectorySummary {
    pub config_dir: String,
    pub data_dir: String,
    pub state_dir: String,
    pub cache_dir: String,
    pub log_dir: String,
    pub scope: String,
    pub override_mechanisms: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct RuntimeOverrides {
    pub config_dir: Option<PathBuf>,
    pub data_dir: Option<PathBuf>,
    pub state_dir: Option<PathBuf>,
    pub cache_dir: Option<PathBuf>,
    pub log_dir: Option<PathBuf>,
}

pub fn resolve_runtime_locations(
    overrides: &RuntimeOverrides,
    _log_enabled: bool,
) -> Result<RuntimeDirectorySummary> {
    let dirs = ProjectDirs::from("com", "byteland", "GhidraAgentCli").map(|d| {
        (
            d.config_dir().to_path_buf(),
            d.data_dir().to_path_buf(),
            d.cache_dir().to_path_buf(),
        )
    });

    let (config, data, cache) = dirs.unwrap_or_else(|| {
        let base = std::env::current_dir().unwrap_or_default();
        (base.join("config"), base.join("data"), base.join("cache"))
    });

    let config_dir = overrides.config_dir.clone().unwrap_or(config);
    let data_dir = overrides.data_dir.clone().unwrap_or(data);
    let state_dir = overrides
        .state_dir
        .clone()
        .unwrap_or_else(|| data_dir.join("state"));
    let cache_dir = overrides.cache_dir.clone().unwrap_or(cache);
    let log_dir = overrides
        .log_dir
        .clone()
        .unwrap_or_else(|| state_dir.join("logs"));

    Ok(RuntimeDirectorySummary {
        config_dir: config_dir.display().to_string(),
        data_dir: data_dir.display().to_string(),
        state_dir: state_dir.display().to_string(),
        cache_dir: cache_dir.display().to_string(),
        log_dir: log_dir.display().to_string(),
        scope: if overrides.config_dir.is_some() || overrides.data_dir.is_some() {
            "explicit_override".to_string()
        } else {
            "user_scoped_default".to_string()
        },
        override_mechanisms: vec![
            "--config-dir".into(),
            "--data-dir".into(),
            "--state-dir".into(),
            "--cache-dir".into(),
            "--log-dir".into(),
        ],
    })
}
