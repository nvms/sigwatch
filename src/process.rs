use anyhow::{bail, Result};
use procmod_core::{MemoryRegion, Module, Process};

pub fn attach(pid: u32) -> Result<Process> {
    Process::attach(pid).map_err(|e| anyhow::anyhow!("failed to attach to pid {pid}: {e}"))
}

pub fn find_pid_by_name(name: &str) -> Result<u32> {
    #[cfg(target_os = "linux")]
    {
        find_pid_linux(name)
    }
    #[cfg(target_os = "macos")]
    {
        find_pid_macos(name)
    }
    #[cfg(target_os = "windows")]
    {
        find_pid_windows(name)
    }
}

#[cfg(target_os = "linux")]
fn find_pid_linux(name: &str) -> Result<u32> {
    use std::fs;
    for entry in fs::read_dir("/proc")? {
        let entry = entry?;
        let pid_str = entry.file_name();
        let Some(pid_str) = pid_str.to_str() else {
            continue;
        };
        let Ok(pid) = pid_str.parse::<u32>() else {
            continue;
        };
        let comm_path = format!("/proc/{pid}/comm");
        if let Ok(comm) = fs::read_to_string(&comm_path) {
            if comm.trim() == name {
                return Ok(pid);
            }
        }
    }
    bail!("no process found with name: {name}")
}

#[cfg(target_os = "macos")]
fn find_pid_macos(name: &str) -> Result<u32> {
    let output = std::process::Command::new("pgrep")
        .arg("-x")
        .arg(name)
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let pid = stdout
        .lines()
        .next()
        .and_then(|line| line.trim().parse::<u32>().ok());
    match pid {
        Some(p) => Ok(p),
        None => bail!("no process found with name: {name}"),
    }
}

#[cfg(target_os = "windows")]
fn find_pid_windows(name: &str) -> Result<u32> {
    let output = std::process::Command::new("tasklist")
        .arg("/FI")
        .arg(format!("IMAGENAME eq {name}"))
        .arg("/FO")
        .arg("CSV")
        .arg("/NH")
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() >= 2 {
            let pid_str = fields[1].trim_matches('"').trim();
            if let Ok(pid) = pid_str.parse::<u32>() {
                return Ok(pid);
            }
        }
    }
    bail!("no process found with name: {name}")
}

pub fn read_value<T: Copy>(process: &Process, address: usize) -> Result<T> {
    // safety: caller must ensure T is valid for any bit pattern
    unsafe {
        process
            .read::<T>(address)
            .map_err(|e| anyhow::anyhow!("read failed at 0x{address:X}: {e}"))
    }
}

pub fn read_bytes(process: &Process, address: usize, len: usize) -> Result<Vec<u8>> {
    process
        .read_bytes(address, len)
        .map_err(|e| anyhow::anyhow!("read_bytes failed at 0x{address:X}: {e}"))
}

pub fn modules(process: &Process) -> Result<Vec<Module>> {
    process
        .modules()
        .map_err(|e| anyhow::anyhow!("module enumeration failed: {e}"))
}

pub fn regions(process: &Process) -> Result<Vec<MemoryRegion>> {
    process
        .regions()
        .map_err(|e| anyhow::anyhow!("region query failed: {e}"))
}
