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

/// Vendors a pristine source tree for a third-party library.
/// Copies the source to artifacts/<target>/third-party/sources/<lib>@<ver>/,
/// optionally commits to git, and updates identified.yaml with vendored_path.
pub fn vendor_pristine(
    workspace: &Path,
    target: &str,
    library: &str,
    source_path: &Path,
    commit: bool,
) -> Result<PathBuf> {
    let mut tp = load_third_party(workspace, target)?;
    let lib = tp
        .libraries
        .iter_mut()
        .find(|l| l.library == library)
        .ok_or_else(|| anyhow::anyhow!("third-party library '{}' not found", library))?;

    let vendored_dir = artifact_dir(workspace, target)
        .join("third-party")
        .join("sources")
        .join(format!("{}@{}", lib.library, lib.version));

    // Copy source tree
    if vendored_dir.exists() {
        std::fs::remove_dir_all(&vendored_dir).with_context(|| {
            format!("failed to remove existing vendored dir {:?}", vendored_dir)
        })?;
    }
    copy_dir_recursive(source_path, &vendored_dir).with_context(|| {
        format!(
            "failed to copy source from {:?} to {:?}",
            source_path, vendored_dir
        )
    })?;

    // Optionally commit
    if commit {
        let repo = git2::Repository::discover(workspace)
            .with_context(|| "failed to discover git repository")?;
        let mut index = repo.index()?;
        let workdir = repo.workdir().with_context(|| "failed to get workdir")?;
        add_dir_to_index(&mut index, &vendored_dir, workdir)?;
        index.write()?;
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;
        let sig = git2::Signature::now("ghidra-agent-cli", "ghidra-agent-cli@local")
            .with_context(|| "failed to create git signature")?;
        repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            &format!(
                "vendor: add pristine source for {}@{}\n\nVendored from {:?}",
                lib.library, lib.version, source_path
            ),
            &tree,
            &[&repo.head()?.peel_to_commit()?],
        )?;
    }

    // Update identified.yaml with vendored_path
    let rel_path = vendored_dir
        .strip_prefix(artifact_dir(workspace, target))
        .unwrap_or(&vendored_dir);
    lib.vendored_path = Some(rel_path.to_string_lossy().to_string());
    save_third_party(workspace, target, &tp)?;

    Ok(vendored_dir)
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

fn add_dir_to_index(index: &mut git2::Index, dir: &Path, repo_workdir: &Path) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            add_dir_to_index(index, &path, repo_workdir)?;
        } else {
            // git2::Index::add_path requires paths relative to the repository root (workdir)
            let relative = path.strip_prefix(repo_workdir).unwrap_or(&path);
            index.add_path(relative)?;
        }
    }
    Ok(())
}
