use anyhow::{anyhow, Result};
#[cfg(any(target_os = "macos", target_os = "linux"))]
use nix::sys::signal::{kill, Signal};
#[cfg(any(target_os = "macos", target_os = "linux"))]
use nix::unistd::Pid;
use std::path::PathBuf;
use std::process::{Command, Stdio};

// TODO(b5): instead of using u32's for Process Identifiers, use a proper Pid type
// something along the lines of:

// #[cfg(unix)]
// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub struct Pid(nix::unistd::Pid);

// #[cfg(not(unix))]
// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub struct Pid; // TODO: fill in for each platform when supported

// // #[cfg(unix)]
// impl From nix::Pid for Pid {
//  // ..
// }

// impl std::fmt::Display for Pid {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", self.to_string())
//     }
// }

pub fn daemonize(bin_path: PathBuf) -> Result<()> {
    daemonize_process(bin_path)
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn daemonize_process(bin_path: PathBuf) -> Result<()> {
    Err(anyhow!(
        "deamonizing processes is not supported on your operating system"
    ))
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn daemonize_process(bin_path: PathBuf) -> Result<()> {
    // ¯\_(ツ)_/¯
    let status = Command::new("bash")
        .arg("-c")
        // TODO(b5): might be nice to capture output in a log file at some point?
        .arg(format!(
            "nohup {} > /dev/null 2>&1 &",
            bin_path.to_str().unwrap(),
        ))
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .status()?;

    if !status.success() {
        Err(anyhow::anyhow!("couldn't daemonize binary"))?;
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn daemonize_process(bin_path: PathBuf) -> Result<()> {
    Err(anyhow!("deamonizing processes on windows is not supported"))
}

pub fn stop(pid: u32) -> Result<()> {
    stop_process(pid)
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn stop_process(pid: u32) -> Result<()> {
    Err(anyhow!(
        "stopping processes is not supported on your operating system"
    ))
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn stop_process(pid: u32) -> Result<()> {
    let id = Pid::from_raw(pid.try_into()?);
    kill(id, Signal::SIGINT).map_err(|e| anyhow!("killing process, error number: {}", e))
}

#[cfg(target_os = "windows")]
fn stop_process(pid: u32) -> Result<()> {
    Err(anyhow!("stopping processes on windows is not supported"))
}
