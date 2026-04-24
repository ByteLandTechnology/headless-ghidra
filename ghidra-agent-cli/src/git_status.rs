use anyhow::{Context, Result, anyhow};
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct GitWorktree {
    pub root: PathBuf,
}

#[derive(Debug, Clone)]
pub struct GitFileStatus {
    pub display: String,
    pub tracked_or_staged: bool,
}

pub fn discover_worktree(workspace: &Path) -> Result<Option<GitWorktree>> {
    let inside = match Command::new("git")
        .arg("-C")
        .arg(workspace)
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
    {
        Ok(output) => output,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(anyhow!("failed to run git: {err}")),
    };

    if !inside.status.success() {
        return Ok(None);
    }
    if String::from_utf8_lossy(&inside.stdout).trim() != "true" {
        return Err(anyhow!("git repository has no workdir"));
    }

    let top = Command::new("git")
        .arg("-C")
        .arg(workspace)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("failed to resolve git worktree root")?;
    if !top.status.success() {
        let stderr = String::from_utf8_lossy(&top.stderr).trim().to_string();
        return Err(anyhow!(
            "failed to resolve git worktree root{}",
            if stderr.is_empty() {
                String::new()
            } else {
                format!(": {stderr}")
            }
        ));
    }

    let root = PathBuf::from(String::from_utf8_lossy(&top.stdout).trim());
    Ok(Some(GitWorktree {
        root: std::fs::canonicalize(&root).unwrap_or(root),
    }))
}

pub fn repo_relative_path(worktree: &GitWorktree, path: &Path) -> PathBuf {
    let canonical_path = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    canonical_path
        .strip_prefix(&worktree.root)
        .unwrap_or(path)
        .to_path_buf()
}

pub fn status_file(worktree: &GitWorktree, repo_rel: &Path) -> GitFileStatus {
    let output = Command::new("git")
        .arg("-C")
        .arg(&worktree.root)
        .args(["status", "--porcelain=v1", "--untracked-files=all", "--"])
        .arg(repo_rel)
        .output();

    let Ok(output) = output else {
        return GitFileStatus {
            display: "WT_NEW".into(),
            tracked_or_staged: false,
        };
    };
    if !output.status.success() {
        return GitFileStatus {
            display: "WT_NEW".into(),
            tracked_or_staged: false,
        };
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let Some(line) = stdout.lines().next() else {
        return GitFileStatus {
            display: "CURRENT".into(),
            tracked_or_staged: true,
        };
    };

    let mut chars = line.chars();
    let index = chars.next().unwrap_or(' ');
    let worktree_status = chars.next().unwrap_or(' ');
    let tracked_or_staged = matches!(index, 'A' | 'M' | 'R' | 'T');
    GitFileStatus {
        display: porcelain_status_name(index, worktree_status),
        tracked_or_staged,
    }
}

fn porcelain_status_name(index: char, worktree_status: char) -> String {
    match (index, worktree_status) {
        ('?', '?') => "WT_NEW".into(),
        ('A', _) => "INDEX_NEW".into(),
        ('M', _) => "INDEX_MODIFIED".into(),
        ('R', _) => "INDEX_RENAMED".into(),
        ('T', _) => "INDEX_TYPECHANGE".into(),
        ('D', _) => "INDEX_DELETED".into(),
        (_, 'M') => "WT_MODIFIED".into(),
        (_, 'D') => "WT_DELETED".into(),
        (_, 'T') => "WT_TYPECHANGE".into(),
        (x, y) => format!("PORCELAIN_STATUS_{x}{y}"),
    }
}
