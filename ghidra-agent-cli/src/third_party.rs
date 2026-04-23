use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::schema::{load_yaml, save_yaml};
use crate::workspace::artifact_dir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThirdPartyLib {
    pub library: String,
    pub version: String,
    pub confidence: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upstream_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vendored_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pristine_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
    #[serde(default)]
    pub function_classifications: Vec<FunctionClassification>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionClassification {
    pub addr: String,
    pub classification: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThirdPartyYaml {
    pub target: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub review_note: Option<String>,
    pub libraries: Vec<ThirdPartyLib>,
}

pub fn load_third_party(workspace: &Path, target: &str) -> Result<ThirdPartyYaml> {
    load_yaml(
        &artifact_dir(workspace, target)
            .join("third-party")
            .join("identified.yaml"),
    )
}

pub fn save_third_party(workspace: &Path, target: &str, data: &ThirdPartyYaml) -> Result<()> {
    save_yaml(
        &artifact_dir(workspace, target)
            .join("third-party")
            .join("identified.yaml"),
        data,
    )
}

/// Records a pristine source tree for a third-party library.
/// Copies the source to artifacts/<target>/third-party/pristine/<lib>@<ver>/,
/// and updates identified.yaml with pristine_path and source_path. The commit
/// argument is accepted for legacy CLI compatibility but is intentionally ignored.
pub fn vendor_pristine(
    workspace: &Path,
    target: &str,
    library: &str,
    source_path: &Path,
    _commit: bool,
) -> Result<PathBuf> {
    let mut tp = load_third_party(workspace, target)?;
    let lib = tp
        .libraries
        .iter_mut()
        .find(|l| l.library == library)
        .ok_or_else(|| anyhow::anyhow!("third-party library '{}' not found", library))?;

    let pristine_name = format!(
        "{}@{}",
        safe_dir_component(&lib.library)?,
        safe_dir_component(&lib.version)?
    );
    let pristine_dir = artifact_dir(workspace, target)
        .join("third-party")
        .join("pristine")
        .join(pristine_name);

    // Copy source tree
    if pristine_dir.exists() {
        std::fs::remove_dir_all(&pristine_dir).with_context(|| {
            format!("failed to remove existing pristine dir {:?}", pristine_dir)
        })?;
    }
    copy_dir_recursive(source_path, &pristine_dir).with_context(|| {
        format!(
            "failed to copy source from {:?} to {:?}",
            source_path, pristine_dir
        )
    })?;

    let rel_path = pristine_dir
        .strip_prefix(artifact_dir(workspace, target))
        .unwrap_or(&pristine_dir);
    lib.vendored_path = None;
    lib.pristine_path = Some(rel_path.to_string_lossy().to_string());
    lib.source_path = Some(source_path.display().to_string());
    save_third_party(workspace, target, &tp)?;

    Ok(pristine_dir)
}

fn safe_dir_component(raw: &str) -> Result<String> {
    let component: String = raw
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.') {
                c
            } else {
                '_'
            }
        })
        .collect();
    if component.is_empty() || component == "." || component == ".." {
        anyhow::bail!("invalid artifact path component '{}'", raw);
    }
    Ok(component)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
