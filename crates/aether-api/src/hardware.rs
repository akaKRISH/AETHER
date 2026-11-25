//! Read-only diagnostics for a known local NVIDIA driver tool.

use serde::Serialize;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[derive(Debug, Serialize)]
pub struct GpuSnapshot {
    pub status: &'static str,
    /// A missing observation leaves the device list empty.
    pub devices: Vec<Gpu>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Gpu {
    name: String,
    driver: String,
    memory_total_mib: u32,
    memory_free_mib: u32,
    utilization_percent: u8,
}

pub fn gpu_snapshot() -> GpuSnapshot {
    let Some(tool) = driver_tool().filter(|path| path.is_absolute() && path.is_file()) else {
        return GpuSnapshot {
            status: "driver_tool_unavailable",
            devices: Vec::new(),
        };
    };
    match query(tool) {
        Ok(devices) if !devices.is_empty() => GpuSnapshot {
            status: "observed",
            devices,
        },
        _ => GpuSnapshot {
            status: "telemetry_unavailable",
            devices: Vec::new(),
        },
    }
}

fn driver_tool() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        Some(PathBuf::from(std::env::var_os("SystemRoot")?).join("System32/nvidia-smi.exe"))
    }
    #[cfg(target_os = "linux")]
    {
        Some(PathBuf::from("/usr/bin/nvidia-smi"))
    }
    #[cfg(not(any(windows, target_os = "linux")))]
    {
        None
    }
}

fn query(tool: PathBuf) -> io::Result<Vec<Gpu>> {
    const LIMIT: u64 = 16_384;
    let mut command = Command::new(tool);
    command
        .env_clear()
        .args([
            "--query-gpu=name,driver_version,memory.total,memory.free,utilization.gpu",
            "--format=csv,noheader,nounits",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // NVIDIA's Windows driver loader needs ProgramFiles to initialize NVML.
        for key in ["SystemRoot", "ProgramFiles"] {
            if let Some(directory) = std::env::var_os(key) {
                command.env(key, directory);
            }
        }
        command.creation_flags(0x0800_0000);
    }
    let mut child = command.spawn()?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| io::Error::other("driver output pipe unavailable"))?;
    let reader = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        stdout.take(LIMIT + 1).read_to_end(&mut bytes)?;
        Ok::<_, io::Error>(bytes)
    });
    let started = Instant::now();
    let result = loop {
        match child.try_wait() {
            Ok(Some(status)) if status.success() => break Ok(()),
            Ok(Some(_)) => break Err(io::Error::other("driver query failed")),
            Err(error) => break Err(error),
            Ok(None) if started.elapsed() >= Duration::from_secs(2) => {
                break Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "driver query timed out",
                ));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(10)),
        }
    };
    if result.is_err() {
        let _ = child.kill();
        child.wait()?;
    }
    let bytes = reader
        .join()
        .map_err(|_| io::Error::other("driver reader failed"))??;
    result?;
    if bytes.len() as u64 > LIMIT {
        return Err(io::Error::other(
            "driver output exceeds the diagnostic limit",
        ));
    }
    let text = std::str::from_utf8(&bytes).map_err(io::Error::other)?;
    text.lines()
        .map(|line| parse_device(line).ok_or_else(|| io::Error::other("invalid driver telemetry")))
        .collect()
}

fn parse_device(line: &str) -> Option<Gpu> {
    let mut fields = line.rsplitn(5, ',').map(str::trim);
    let utilization_percent: u8 = fields.next()?.parse().ok()?;
    let memory_free_mib: u32 = fields.next()?.parse().ok()?;
    let memory_total_mib: u32 = fields.next()?.parse().ok()?;
    let driver = fields.next()?;
    let name = fields.next()?;
    if utilization_percent > 100
        || memory_free_mib > memory_total_mib
        || name.is_empty()
        || driver.is_empty()
    {
        return None;
    }
    Some(Gpu {
        name: name.to_owned(),
        driver: driver.to_owned(),
        memory_total_mib,
        memory_free_mib,
        utilization_percent,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gpu_parser_never_interprets_unavailable_fields_as_zero() {
        let device = parse_device("NVIDIA Example, 100.00, 6144, 6123, 4").unwrap();
        assert_eq!(device.memory_free_mib, 6123);
        assert_eq!(device.utilization_percent, 4);
        assert!(parse_device("NVIDIA Example, 100.00, 6144, N/A, 4").is_none());
        assert!(parse_device("NVIDIA Example, 100.00, 6144, 6145, 4").is_none());
        assert!(parse_device("NVIDIA Example, 100.00, 6144, 6123, 101").is_none());
    }
}
