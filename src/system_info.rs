use crate::AppResult;
use local_ip_address::local_ip;
use std::env;
use std::process::Command;
use sysinfo::System;

/// CPU information structure
#[derive(Debug, Clone)]
pub struct CpuInfo {
    pub model: String,
    pub cores: usize,
    pub frequency: u64, // MHz
}

/// GPU information structure
#[derive(Debug, Clone)]
pub struct GpuInfo {
    pub name: String,
    pub vendor: String,
}

/// System information structure
#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub hostname: String,
    pub username: String,
    pub uptime: String,
    pub cpus: Vec<CpuInfo>,
    pub memory_total: u64,
    pub memory_used: u64,
    pub disk_total: u64,
    pub disk_used: u64,
    pub gpus: Vec<GpuInfo>,
    pub local_ip: String,
}

impl SystemInfo {
    /// Collect system information
    pub fn collect() -> AppResult<Self> {
        let mut sys = System::new_all();
        sys.refresh_all();

        // Basic system information
        let os_name = System::name().unwrap_or_else(|| "Unknown".to_string());
        let os_version = System::os_version().unwrap_or_else(|| "Unknown".to_string());
        let kernel_version = System::kernel_version().unwrap_or_else(|| "Unknown".to_string());
        let hostname = System::host_name().unwrap_or_else(|| "Unknown".to_string());
        let username = env::var("USER")
            .or_else(|_| env::var("USERNAME"))
            .unwrap_or_else(|_| "Unknown".to_string());

        // Uptime
        let uptime_seconds = System::uptime();
        let uptime = format_uptime(uptime_seconds);

        // CPU information
        let cpus = collect_cpu_info(&sys);

        // Memory information
        let memory_total = sys.total_memory();
        let memory_used = sys.used_memory();

        // Disk information
        let disk_total = 0u64;
        let disk_used = 0u64;

        // GPU information
        let gpus = get_gpu_info_list();

        // Local IP address
        let local_ip = get_local_ip();

        Ok(Self {
            os_name,
            os_version,
            kernel_version,
            hostname,
            username,
            uptime,
            cpus,
            memory_total,
            memory_used,
            disk_total,
            disk_used,
            gpus,
            local_ip,
        })
    }
}

/// Format uptime
fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;

    if days > 0 {
        format!("{}d {}h {}m", days, hours, minutes)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}

/// Format byte size
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_info_collection() {
        let info = SystemInfo::collect().unwrap();
        assert!(!info.os_name.is_empty());
        assert!(!info.cpus.is_empty());
    }

    #[test]
    fn test_format_uptime() {
        assert_eq!(format_uptime(3661), "1h 1m");
        assert_eq!(format_uptime(90061), "1d 1h 1m");
        assert_eq!(format_uptime(61), "1m");
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1048576), "1.0 MB");
        assert_eq!(format_bytes(1073741824), "1.0 GB");
    }
}

/// Collect CPU information
fn collect_cpu_info(sys: &System) -> Vec<CpuInfo> {
    let mut cpu_map = std::collections::HashMap::new();

    // Group CPUs by model to handle multi-core processors
    for cpu in sys.cpus() {
        let model = cpu.brand().to_string();
        let frequency = cpu.frequency();

        let entry = cpu_map.entry(model.clone()).or_insert(CpuInfo {
            model,
            cores: 0,
            frequency,
        });
        entry.cores += 1;
    }

    cpu_map.into_values().collect()
}

/// Get GPU information list
fn get_gpu_info_list() -> Vec<GpuInfo> {
    if cfg!(target_os = "windows") {
        get_gpu_info_windows_list()
    } else if cfg!(target_os = "linux") {
        get_gpu_info_linux_list()
    } else if cfg!(target_os = "macos") {
        get_gpu_info_macos_list()
    } else {
        vec![GpuInfo {
            name: "Unknown GPU".to_string(),
            vendor: "Unknown".to_string(),
        }]
    }
}

/// Get GPU information on Windows system (multiple GPUs)
fn get_gpu_info_windows_list() -> Vec<GpuInfo> {
    let mut gpus = Vec::new();

    match Command::new("wmic")
        .args(&[
            "path",
            "win32_VideoController",
            "get",
            "name,AdapterCompatibility",
            "/format:value",
        ])
        .output()
    {
        Ok(output) => {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let mut current_gpu = GpuInfo {
                name: String::new(),
                vendor: String::new(),
            };

            for line in output_str.lines() {
                let line = line.trim();
                if line.starts_with("AdapterCompatibility=")
                    && !line.trim_end_matches("AdapterCompatibility=").is_empty()
                {
                    current_gpu.vendor = line
                        .trim_start_matches("AdapterCompatibility=")
                        .trim()
                        .to_string();
                } else if line.starts_with("Name=") && !line.trim_end_matches("Name=").is_empty() {
                    current_gpu.name = line.trim_start_matches("Name=").trim().to_string();

                    // If we have both name and vendor, add to list
                    if !current_gpu.name.is_empty() {
                        gpus.push(current_gpu.clone());
                        current_gpu = GpuInfo {
                            name: String::new(),
                            vendor: String::new(),
                        };
                    }
                }
            }
        }
        Err(_) => {}
    }

    if gpus.is_empty() {
        gpus.push(GpuInfo {
            name: "Unknown GPU".to_string(),
            vendor: "Unknown".to_string(),
        });
    }

    gpus
}

/// Get GPU information on Linux system (multiple GPUs)
fn get_gpu_info_linux_list() -> Vec<GpuInfo> {
    let mut gpus = Vec::new();

    match Command::new("lspci").args(&["-mm"]).output() {
        Ok(output) => {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if line.contains("VGA compatible controller") || line.contains("3D controller") {
                    let parts: Vec<&str> = line.split('"').collect();
                    if parts.len() >= 6 {
                        gpus.push(GpuInfo {
                            name: format!("{} {}", parts[3], parts[5]),
                            vendor: parts[3].to_string(),
                        });
                    }
                }
            }
        }
        Err(_) => {}
    }

    if gpus.is_empty() {
        gpus.push(GpuInfo {
            name: "Unknown GPU".to_string(),
            vendor: "Unknown".to_string(),
        });
    }

    gpus
}

/// Get GPU information on macOS system (multiple GPUs)
fn get_gpu_info_macos_list() -> Vec<GpuInfo> {
    let mut gpus = Vec::new();

    match Command::new("system_profiler")
        .args(&["SPDisplaysDataType", "-json"])
        .output()
    {
        Ok(output) => {
            let output_str = String::from_utf8_lossy(&output.stdout);

            // Simple parsing to find all GPU names
            let mut pos = 0;
            while let Some(start) = output_str[pos..].find("\"_name\" : \"") {
                let start = pos + start + 11;
                if let Some(end) = output_str[start..].find('"') {
                    let gpu_name = output_str[start..start + end].to_string();
                    gpus.push(GpuInfo {
                        name: gpu_name.clone(),
                        vendor: if gpu_name.to_lowercase().contains("nvidia") {
                            "NVIDIA".to_string()
                        } else if gpu_name.to_lowercase().contains("amd")
                            || gpu_name.to_lowercase().contains("radeon")
                        {
                            "AMD".to_string()
                        } else if gpu_name.to_lowercase().contains("intel") {
                            "Intel".to_string()
                        } else {
                            "Unknown".to_string()
                        },
                    });
                    pos = start + end;
                } else {
                    break;
                }
            }
        }
        Err(_) => {}
    }

    if gpus.is_empty() {
        gpus.push(GpuInfo {
            name: "Unknown GPU".to_string(),
            vendor: "Unknown".to_string(),
        });
    }

    gpus
}

/// Get local IP address
fn get_local_ip() -> String {
    match local_ip() {
        Ok(ip) => ip.to_string(),
        Err(_) => "Unknown IP".to_string(),
    }
}
