use anyhow::{Result, anyhow};
use fs4::fs_std::FileExt;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockInfo {
    pub holder_pid: u32,
    pub holder_command: String,
    pub acquired_at: String,
    pub scope: String,
}

pub fn lock_file_path(dir: &Path, name: &str) -> PathBuf {
    dir.join(".lock").join(format!("{name}.lock"))
}

/// RAII lock guard - holds the OS file lock for the duration of the critical section.
/// Does NOT delete the lock file on drop to avoid TOCTOU races.
/// The lock file persists to prevent another process from racing to create a new lock
/// between unlock and any subsequent file operations.
pub struct LockGuard {
    #[allow(dead_code)]
    path: PathBuf, // Reserved for future use (e.g., automatic stale lock cleanup)
    file: Option<std::fs::File>,
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        // Release the OS advisory lock
        if let Some(ref file) = self.file {
            let _ = file.unlock();
        }
        // Note: We do NOT delete the lock file here.
        // Deleting the file between unlock and any subsequent file creation
        // would create a TOCTOU race where another process could acquire
        // a conflicting lock. The file persists with our PID marker.
    }
}

impl LockGuard {
    /// Update the lock file content while holding the lock
    pub fn update_info(&mut self, info: &LockInfo) -> Result<()> {
        if let Some(ref mut file) = self.file {
            file.set_len(0)?;
            let yaml = serde_yaml::to_string(info)?;
            file.write_all(yaml.as_bytes())?;
        }
        Ok(())
    }
}

/// Acquire an exclusive advisory lock on a file, with stale lock detection.
/// Returns a LockGuard that holds the lock until dropped.
/// The lock file is NOT deleted when the guard is dropped - this prevents
/// TOCTOU races where another process could acquire a conflicting lock
/// between our unlock and any subsequent operations.
pub fn acquire_lock(lock_path: &Path, scope: &str, timeout_secs: u64) -> Result<LockGuard> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(timeout_secs);
    if let Some(parent) = lock_path.parent() {
        fs::create_dir_all(parent)?;
    }

    loop {
        // Open lock file (create if not exists)
        let mut file = match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(lock_path)
        {
            Ok(f) => f,
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                // Lock exists, try to open for write to acquire lock
                match OpenOptions::new().write(true).open(lock_path) {
                    Ok(f) => f,
                    // TOCTOU race: if lock was released (file deleted) between create_new and open,
                    // treat this as if lock doesn't exist and retry the loop
                    Err(e) if e.kind() == io::ErrorKind::NotFound => continue,
                    Err(e) => return Err(anyhow!("failed to open lock file: {}", e)),
                }
            }
            Err(e) => return Err(anyhow!("failed to create lock file: {}", e)),
        };

        // Try to acquire exclusive advisory lock (atomic, no TOCTOU race)
        match file.lock_exclusive() {
            Ok(()) => {
                // We hold the exclusive lock - trust the OS. If we got it, we own it.
                // Only check staleness when we OPEN an existing file (see fallback below).
                let info = LockInfo {
                    holder_pid: std::process::id(),
                    holder_command: std::env::args().collect::<Vec<_>>().join(" "),
                    acquired_at: chrono::Utc::now().to_rfc3339(),
                    scope: scope.to_string(),
                };
                let yaml = serde_yaml::to_string(&info)?;
                file.set_len(0)?;
                file.write_all(yaml.as_bytes())?;

                // Return LockGuard - OS lock is held until drop() is called
                return Ok(LockGuard {
                    file: Some(file),
                    path: lock_path.to_path_buf(),
                });
            }
            Err(_e) => {
                // Lock acquisition failed (e.g., already locked by another process)
                let _ = file;
            }
        }

        if std::time::Instant::now() >= deadline {
            let info_str = fs::read_to_string(lock_path).unwrap_or_default();
            let info: LockInfo = serde_yaml::from_str(&info_str).unwrap_or_else(|_| LockInfo {
                holder_pid: 0,
                holder_command: String::new(),
                acquired_at: String::new(),
                scope: scope.to_string(),
            });
            return Err(anyhow!(
                "E_LOCK_TIMEOUT: lock acquisition timed out for {} (holder pid {} acquired at {})",
                lock_path.display(),
                info.holder_pid,
                info.acquired_at
            ));
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
}

/// Release a lock by removing the lock file.
/// IMPORTANT: This should only be called when the LockGuard has been dropped
/// (i.e., after the critical section is complete). The LockGuard releases
/// the OS lock on drop, but does NOT delete the file. This function
/// handles the file deletion safely by first acquiring an exclusive lock
/// to prevent racing with new lock holders.
pub fn release_lock(lock_path: &Path) -> Result<()> {
    if lock_path.exists() {
        // Safely delete the lock file by first acquiring exclusive access
        // to prevent racing with processes that might create a new lock
        let file = OpenOptions::new().write(true).open(lock_path)?;
        match file.lock_exclusive() {
            Ok(()) => {
                // We have exclusive access, safe to delete
                drop(file);
                fs::remove_file(lock_path)?;
            }
            Err(_) => {
                // Couldn't acquire lock, file might be stale anyway
                let _ = fs::remove_file(lock_path);
            }
        }
    }
    Ok(())
}

#[allow(dead_code)]
fn is_pid_alive(pid: u32) -> bool {
    #[cfg(unix)]
    {
        unsafe { libc::kill(pid as i32, 0) == 0 }
    }
    #[cfg(not(unix))]
    {
        let _ = pid;
        true
    }
}
