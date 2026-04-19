use anyhow::{Context, Result, anyhow};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct RuntimeOverrides {
    pub config_dir: Option<PathBuf>,
    pub data_dir: Option<PathBuf>,
    pub state_dir: Option<PathBuf>,
    pub cache_dir: Option<PathBuf>,
    pub log_dir: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct RuntimeLocations {
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub state_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub log_dir: Option<PathBuf>,
    pub scope: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeDirectorySummary {
    pub config_dir: String,
    pub data_dir: String,
    pub state_dir: String,
    pub cache_dir: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_dir: Option<String>,
    pub scope: String,
    pub override_mechanisms: Vec<String>,
}

impl RuntimeLocations {
    pub fn summary(&self) -> RuntimeDirectorySummary {
        RuntimeDirectorySummary {
            config_dir: self.config_dir.display().to_string(),
            data_dir: self.data_dir.display().to_string(),
            state_dir: self.state_dir.display().to_string(),
            cache_dir: self.cache_dir.display().to_string(),
            log_dir: self.log_dir.as_ref().map(|p| p.display().to_string()),
            scope: self.scope.clone(),
            override_mechanisms: vec![
                "--config-dir".into(),
                "--data-dir".into(),
                "--state-dir".into(),
                "--cache-dir".into(),
                "--log-dir".into(),
            ],
        }
    }

    pub fn context_file(&self) -> PathBuf {
        self.state_dir.join("active-context.toml")
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ActiveContextState {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub selectors: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub ambient_cues: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Default)]
pub struct InvocationContextOverrides {
    pub selectors: BTreeMap<String, String>,
    pub current_directory: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EffectiveContextView {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub effective_values: BTreeMap<String, String>,
    pub precedence_rule: String,
    pub persisted_context_present: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextInspection {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persisted_context: Option<ActiveContextState>,
    pub effective_context: EffectiveContextView,
    pub context_file: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextPersistenceResult {
    pub status: String,
    pub message: String,
    pub active_context: ActiveContextState,
    pub context_file: String,
}

pub fn resolve_runtime_locations(
    overrides: &RuntimeOverrides,
    log_enabled: bool,
) -> Result<RuntimeLocations> {
    let project_dirs = ProjectDirs::from("com", "byteland", "GhidraAgentCli")
        .ok_or_else(|| anyhow!("failed to resolve platform project directories"))?;

    let data_dir = overrides
        .data_dir
        .clone()
        .unwrap_or_else(|| project_dirs.data_dir().to_path_buf());
    let state_dir = overrides
        .state_dir
        .clone()
        .unwrap_or_else(|| data_dir.join("state"));
    let log_dir = if overrides.log_dir.is_some() || log_enabled {
        Some(
            overrides
                .log_dir
                .clone()
                .unwrap_or_else(|| state_dir.join("logs")),
        )
    } else {
        None
    };

    Ok(RuntimeLocations {
        config_dir: overrides
            .config_dir
            .clone()
            .unwrap_or_else(|| project_dirs.config_dir().to_path_buf()),
        data_dir: data_dir.clone(),
        state_dir,
        cache_dir: overrides
            .cache_dir
            .clone()
            .unwrap_or_else(|| project_dirs.cache_dir().to_path_buf()),
        log_dir,
        scope: if overrides.config_dir.is_some() || overrides.data_dir.is_some() {
            "explicit_override".to_string()
        } else {
            "user_scoped_default".to_string()
        },
    })
}

pub fn parse_selectors(values: &[String]) -> Result<BTreeMap<String, String>> {
    let mut selectors = BTreeMap::new();
    for value in values {
        let (key, val) = value
            .split_once('=')
            .ok_or_else(|| anyhow!("selector '{}' must use KEY=VALUE", value))?;
        selectors.insert(key.trim().to_string(), val.trim().to_string());
    }
    Ok(selectors)
}

pub fn build_context_state(
    name: Option<String>,
    selectors: BTreeMap<String, String>,
    current_directory: Option<PathBuf>,
) -> ActiveContextState {
    let mut ambient_cues = BTreeMap::new();
    if let Some(cd) = current_directory {
        ambient_cues.insert("current_directory".to_string(), cd.display().to_string());
    }
    ActiveContextState {
        name,
        selectors,
        ambient_cues,
    }
}

pub fn load_active_context(runtime: &RuntimeLocations) -> Result<Option<ActiveContextState>> {
    let f = runtime.context_file();
    if !f.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&f).with_context(|| format!("failed to read {}", f.display()))?;
    Ok(Some(toml::from_str(&raw).with_context(|| {
        format!("failed to parse {}", f.display())
    })?))
}

pub fn persist_active_context(
    runtime: &RuntimeLocations,
    state: &ActiveContextState,
) -> Result<ContextPersistenceResult> {
    if let Some(parent) = runtime.context_file().parent() {
        fs::create_dir_all(parent)?;
    }
    let serialized = toml::to_string_pretty(state).context("failed to serialize Active Context")?;
    let cf = runtime.context_file();
    fs::write(&cf, serialized).with_context(|| format!("failed to write {}", cf.display()))?;
    Ok(ContextPersistenceResult {
        status: "ok".to_string(),
        message: "Active Context updated".to_string(),
        active_context: state.clone(),
        context_file: cf.display().to_string(),
    })
}

pub fn resolve_effective_context(
    persisted: Option<&ActiveContextState>,
    overrides: &InvocationContextOverrides,
) -> EffectiveContextView {
    let mut effective = BTreeMap::new();
    if let Some(p) = persisted {
        effective.extend(p.selectors.clone());
        effective.extend(p.ambient_cues.clone());
    }
    effective.extend(overrides.selectors.clone());
    if let Some(cd) = &overrides.current_directory {
        effective.insert("current_directory".to_string(), cd.display().to_string());
    }
    EffectiveContextView {
        name: persisted.and_then(|p| p.name.clone()),
        effective_values: effective,
        precedence_rule: "explicit invocation values override the persisted Active Context for one invocation only".to_string(),
        persisted_context_present: persisted.is_some(),
    }
}

pub fn inspect_context(
    runtime: &RuntimeLocations,
    overrides: &InvocationContextOverrides,
) -> Result<ContextInspection> {
    let persisted = load_active_context(runtime)?;
    let effective = resolve_effective_context(persisted.as_ref(), overrides);
    Ok(ContextInspection {
        persisted_context: persisted,
        effective_context: effective,
        context_file: runtime.context_file().display().to_string(),
    })
}
