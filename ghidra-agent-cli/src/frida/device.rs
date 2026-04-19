use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    #[serde(rename = "deviceType")]
    pub device_type: String,
    pub serial: Option<String>,
}

#[derive(Debug, Clone)]
pub enum DeviceSelector {
    Local,
    LocalPid(u32),
    Usb(String),
    Network(String),
}

// Manual impl needed because enums with non-default fields can't derive Default
#[allow(clippy::derivable_impls)]
impl Default for DeviceSelector {
    fn default() -> Self {
        Self::Local
    }
}

impl DeviceSelector {
    pub fn to_frida_args(&self) -> Vec<String> {
        match self {
            Self::Local => vec![],
            Self::LocalPid(pid) => vec!["-p".to_string(), pid.to_string()],
            Self::Usb(serial) => {
                if serial.is_empty() {
                    vec!["-U".to_string()]
                } else {
                    vec!["-U".to_string(), "-s".to_string(), serial.clone()]
                }
            }
            Self::Network(host) => vec!["-H".to_string(), host.clone()],
        }
    }

    pub fn parse(s: &str) -> Self {
        if let Some(serial) = s.strip_prefix("usb:") {
            Self::Usb(serial.to_string())
        } else if let Some(host) = s.strip_prefix("network:") {
            Self::Network(host.to_string())
        } else if let Ok(pid) = s.parse::<u32>() {
            Self::LocalPid(pid)
        } else {
            Self::Local
        }
    }
}

pub fn list_devices() -> Result<Vec<DeviceInfo>> {
    // Try to probe USB device availability by running frida with -U
    // If frida-server is running on USB, this will succeed or timeout quickly
    let mut devices = Vec::new();

    // Check local process
    devices.push(DeviceInfo {
        id: "local".to_string(),
        name: "Local System".to_string(),
        device_type: "local".to_string(),
        serial: None,
    });

    // Try to detect USB - spawn a quick probe
    let probe_result = Command::new("frida")
        .args([
            "-U",
            "-l",
            "/dev/null",
            "-F",
            "--no-interactive",
            "--timeout",
            "1",
        ])
        .output();

    if let Ok(output) = probe_result {
        // If it doesn't immediately fail with "no USB" error, we have a USB device
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.contains("no USB") && !stderr.contains("could not find") {
            devices.push(DeviceInfo {
                id: "usb".to_string(),
                name: "USB Device".to_string(),
                device_type: "usb".to_string(),
                serial: None,
            });
        }
    }

    Ok(devices)
}

pub fn check_frida_available() -> Result<String> {
    let output = Command::new("frida").arg("--version").output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Frida not found. Install from https://frida.re"
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn run_frida_with_device(
    selector: &DeviceSelector,
    script: &str,
    spawn_target: Option<&str>,
    extra_args: &[&str],
    timeout_secs: u64,
) -> Result<(String, String)> {
    let mut cmd = Command::new("frida");
    cmd.args(selector.to_frida_args());

    if let Some(target) = spawn_target {
        cmd.arg("-f").arg(target);
    }

    cmd.arg("-l").arg(script);

    // Add -- separator for target arguments (everything after -- goes to spawned process)
    if !extra_args.is_empty() {
        cmd.arg("--");
        for arg in extra_args {
            cmd.arg(arg);
        }
    }

    let output = tokio_process_wrap(&mut cmd, timeout_secs)?;

    Ok(output)
}

fn tokio_process_wrap(
    cmd: &mut std::process::Command,
    timeout_secs: u64,
) -> Result<(String, String)> {
    use std::io::Read;
    use std::time::Duration;

    let mut child = cmd.spawn()?;

    let timeout = Duration::from_secs(timeout_secs);
    let start = std::time::Instant::now();

    let mut stdout_buf = Vec::new();
    let mut stderr_buf = Vec::new();

    loop {
        if start.elapsed() > timeout {
            child.kill()?;
            return Err(anyhow::anyhow!("Process timed out after {}s", timeout_secs));
        }

        match child.try_wait()? {
            Some(_s) => {
                child
                    .stdout
                    .take()
                    .map(|mut f| f.read_to_end(&mut stdout_buf).ok());
                child
                    .stderr
                    .take()
                    .map(|mut f| f.read_to_end(&mut stderr_buf).ok());
                break;
            }
            None => {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }
    }

    let stdout = String::from_utf8_lossy(&stdout_buf).to_string();
    let stderr = String::from_utf8_lossy(&stderr_buf).to_string();

    Ok((stdout, stderr))
}
