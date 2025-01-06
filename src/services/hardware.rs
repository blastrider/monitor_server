use crate::models::errors::SystemError;
use log::{error, warn};
use std::{ffi::CString, fs};

pub fn get_system_version() -> String {
    fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|content| {
            content
                .lines()
                .find(|line| line.starts_with("PRETTY_NAME="))
                .and_then(|line| line.split('=').nth(1))
                .map(|value| value.trim_matches('"').to_string())
        })
        .unwrap_or_else(|| "Unknown System".to_string())
}

pub fn get_kernel_version() -> String {
    std::process::Command::new("uname")
        .arg("-r")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|version| version.trim().to_string())
        .unwrap_or_else(|| "Unknown Kernel".to_string())
}

pub fn get_uptime() -> Result<String, SystemError> {
    fs::read_to_string("/proc/uptime")
        .map_err(|_| {
            error!("{}", SystemError::UptimeUnavailable.message());
            SystemError::UptimeUnavailable
        })
        .and_then(|content| {
            let mut parts = content.split_whitespace();
            if let Some(uptime_seconds) = parts.next().and_then(|s| s.parse::<f64>().ok()) {
                let days = (uptime_seconds / 86400.0).floor() as u64;
                let hours = ((uptime_seconds % 86400.0) / 3600.0).floor() as u64;
                let minutes = ((uptime_seconds % 3600.0) / 60.0).floor() as u64;
                Ok(format!(
                    "{} days, {} hours, {} minutes",
                    days, hours, minutes
                ))
            } else {
                warn!("Uptime format invalid in /proc/uptime");
                Err(SystemError::UptimeUnavailable)
            }
        })
}

pub fn get_memory_info() -> Result<(u64, u64), SystemError> {
    let meminfo = fs::read_to_string("/proc/meminfo").map_err(|_| {
        error!("{}", SystemError::MemoryInfoUnavailable.message());
        SystemError::MemoryInfoUnavailable
    })?;
    let total_memory = extract_memory_value(&meminfo, "MemTotal")?;
    let free_memory = extract_memory_value(&meminfo, "MemAvailable")?;
    Ok((total_memory - free_memory, total_memory))
}

fn extract_memory_value(meminfo: &str, key: &str) -> Result<u64, SystemError> {
    meminfo
        .lines()
        .find(|line| line.starts_with(key))
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|value| value.parse::<u64>().ok())
        .map(|kb| kb * 1024)
        .ok_or_else(|| {
            warn!("Key {} not found in /proc/meminfo", key);
            SystemError::MemoryInfoUnavailable
        })
}

pub fn get_disk_info(path: &str) -> Result<(u64, u64), SystemError> {
    let c_path = CString::new(path).map_err(|_| {
        error!("{}", SystemError::DiskInfoUnavailable.message());
        SystemError::DiskInfoUnavailable
    })?;
    let mut statvfs: libc::statvfs = unsafe { std::mem::zeroed() };

    (unsafe { libc::statvfs(c_path.as_ptr(), &mut statvfs) } == 0)
        .then(|| {
            (
                statvfs.f_bavail as u64 * statvfs.f_frsize as u64,
                statvfs.f_blocks as u64 * statvfs.f_frsize as u64,
            )
        })
        .ok_or_else(|| {
            error!("{}", SystemError::DiskInfoUnavailable.message());
            SystemError::DiskInfoUnavailable
        })
}

pub fn get_temperature() -> Result<String, SystemError> {
    let entries = fs::read_dir("/sys/class/thermal/").map_err(|_| {
        warn!("Temperature sensors directory not found. This may be a VM environment.");
        SystemError::TemperatureSensorsUnavailable
    });

    let entries = match entries {
        Ok(e) => e,
        Err(_) => return Ok("Unavailable (VM environment)".to_string()),
    };

    let temperatures: Vec<f64> = entries
        .flatten()
        .filter_map(|entry| {
            fs::read_to_string(entry.path().join("temp"))
                .ok()
                .and_then(|s| s.trim().parse::<i64>().ok())
                .map(|temp| temp as f64 / 1000.0)
        })
        .collect();

    if temperatures.is_empty() {
        warn!("No temperature data found in /sys/class/thermal/. Returning default value.");
        Ok("Unavailable (VM environment)".to_string())
    } else {
        let avg_temp = temperatures.iter().sum::<f64>() / temperatures.len() as f64;
        Ok(format!("{:.2} °C", avg_temp))
    }
}

pub fn get_network_traffic() -> Result<(u64, u64), SystemError> {
    fs::read_to_string("/proc/net/dev")
        .map_err(|_| {
            error!("{}", SystemError::NetworkTrafficUnavailable.message());
            SystemError::NetworkTrafficUnavailable
        })?
        .lines()
        .skip(2)
        .map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            Ok((
                parts
                    .get(1)
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(0),
                parts
                    .get(9)
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(0),
            ))
        })
        .try_fold((0, 0), |(rx_total, tx_total), result| {
            result.map(|(rx, tx)| (rx_total + rx, tx_total + tx))
        })
}
