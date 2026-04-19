#[derive(Debug)]
pub enum FridaError {
    DeviceNotFound {
        selector: String,
        available: Vec<String>,
    },
    ScriptTimeout {
        script: String,
        timeout_secs: u64,
    },
    SpawnFailed {
        target: String,
        reason: String,
    },
    UsbPermissionError {
        hint: String,
    },
    ScriptExecutionFailed {
        script: String,
        stderr: String,
    },
    ParseError(String),
}

impl std::error::Error for FridaError {}

impl std::fmt::Display for FridaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DeviceNotFound {
                selector,
                available,
            } => {
                write!(
                    f,
                    "Device not found: {}. Available: {}",
                    selector,
                    available.join(", ")
                )
            }
            Self::ScriptTimeout {
                script,
                timeout_secs,
            } => {
                write!(f, "Script {} timed out after {}s", script, timeout_secs)
            }
            Self::SpawnFailed { target, reason } => {
                write!(f, "Failed to spawn '{}': {}", target, reason)
            }
            Self::UsbPermissionError { hint } => {
                write!(f, "USB permission denied. {}", hint)
            }
            Self::ScriptExecutionFailed { script, stderr } => {
                write!(f, "Script {} failed: {}", script, stderr)
            }
            Self::ParseError(msg) => {
                write!(f, "Parse error: {}", msg)
            }
        }
    }
}
