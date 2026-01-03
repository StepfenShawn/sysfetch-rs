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
    pub os_arch: String,
    pub kernel_version: String,
    pub hostname: String,
    pub username: String,
    pub uptime: String,
    pub cpus: Vec<CpuInfo>,
    pub memory_total: u64,
    pub memory_used: u64,
    pub gpus: Vec<GpuInfo>,
    pub local_ip: String,
    pub shell: String,
    pub terminal: String,
}

impl SystemInfo {
    /// Collect system information
    pub fn collect() -> AppResult<Self> {
        let mut sys = System::new_all();
        sys.refresh_all();

        // Basic system information
        let os_name = System::name().unwrap_or_else(|| "Unknown".to_string());
        let os_version = System::os_version().unwrap_or_else(|| "Unknown".to_string());
        let os_arch = std::env::consts::ARCH.into();
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

        // GPU information
        let gpus = get_gpu_info_list();

        // Local IP address
        let local_ip = get_local_ip();

        // Shell and Terminal information
        let shell = get_shell_info();
        let terminal = get_terminal_info();

        Ok(Self {
            os_name,
            os_version,
            os_arch,
            kernel_version,
            hostname,
            username,
            uptime,
            cpus,
            memory_total,
            memory_used,
            gpus,
            local_ip,
            shell,
            terminal,
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

/// Get shell information
fn get_shell_info() -> String {
    // Try to get shell from environment variables
    if let Ok(shell) = env::var("SHELL") {
        // Extract shell name from path
        if let Some(shell_name) = shell.split('/').last() {
            return shell_name.to_string();
        }
        return shell;
    }

    // Windows specific shell detection
    if cfg!(target_os = "windows") {
        // Check for PowerShell
        if env::var("PSModulePath").is_ok() {
            return "PowerShell".to_string();
        }

        // Check for Command Prompt
        if let Ok(comspec) = env::var("COMSPEC") {
            if let Some(shell_name) = comspec.split('\\').last() {
                return shell_name.replace(".exe", "");
            }
        }

        return "cmd".to_string();
    }

    // Unix-like systems fallback
    if let Ok(output) = Command::new("ps")
        .args(&["-p", &std::process::id().to_string(), "-o", "comm="])
        .output()
    {
        let shell = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !shell.is_empty() {
            return shell;
        }
    }

    "Unknown Shell".to_string()
}

/// Get terminal information
fn get_terminal_info() -> String {
    // Check common terminal environment variables
    let terminal_vars = [
        "TERM_PROGRAM",          // macOS Terminal, iTerm2, etc.
        "TERMINAL_EMULATOR",     // Some Linux terminals
        "KONSOLE_VERSION",       // KDE Konsole
        "GNOME_TERMINAL_SCREEN", // GNOME Terminal
        "XTERM_VERSION",         // xterm
        "ALACRITTY_SOCKET",      // Alacritty
        "KITTY_WINDOW_ID",       // Kitty
        "WEZTERM_EXECUTABLE",    // WezTerm
    ];

    for var in &terminal_vars {
        if let Ok(value) = env::var(var) {
            match *var {
                "TERM_PROGRAM" => return value,
                "TERMINAL_EMULATOR" => return value,
                "KONSOLE_VERSION" => return "Konsole".to_string(),
                "GNOME_TERMINAL_SCREEN" => return "GNOME Terminal".to_string(),
                "XTERM_VERSION" => return format!("xterm {}", value),
                "ALACRITTY_SOCKET" => return "Alacritty".to_string(),
                "KITTY_WINDOW_ID" => return "Kitty".to_string(),
                "WEZTERM_EXECUTABLE" => return "WezTerm".to_string(),
                _ => {}
            }
        }
    }

    // Windows specific terminal detection
    if cfg!(target_os = "windows") {
        // Check for Windows Terminal
        if env::var("WT_SESSION").is_ok() {
            return "Windows Terminal".to_string();
        }

        // Check for ConEmu
        if env::var("ConEmuPID").is_ok() {
            return "ConEmu".to_string();
        }

        // Check for Cmder
        if env::var("CMDER_ROOT").is_ok() {
            return "Cmder".to_string();
        }

        // Try to detect through parent process on Windows
        if let Ok(output) = Command::new("wmic")
            .args(&[
                "process",
                "where",
                &format!("ProcessId={}", std::process::id()),
                "get",
                "ParentProcessId",
                "/format:value",
            ])
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if let Some(ppid_str) = line.strip_prefix("ParentProcessId=") {
                    if let Ok(ppid) = ppid_str.trim().parse::<u32>() {
                        if let Ok(parent_output) = Command::new("wmic")
                            .args(&[
                                "process",
                                "where",
                                &format!("ProcessId={}", ppid),
                                "get",
                                "Name",
                                "/format:value",
                            ])
                            .output()
                        {
                            let parent_str = String::from_utf8_lossy(&parent_output.stdout);
                            for parent_line in parent_str.lines() {
                                if let Some(name) = parent_line.strip_prefix("Name=") {
                                    let name = name.trim();
                                    if !name.is_empty() {
                                        return match name {
                                            "WindowsTerminal.exe" => "Windows Terminal".to_string(),
                                            "ConEmu64.exe" | "ConEmu.exe" => "ConEmu".to_string(),
                                            "cmd.exe" => "Command Prompt".to_string(),
                                            "powershell.exe" => "PowerShell".to_string(),
                                            "pwsh.exe" => "PowerShell Core".to_string(),
                                            _ => name.replace(".exe", ""),
                                        };
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        return "Command Prompt".to_string();
    }

    // Unix-like systems: try to get terminal from TERM or parent process
    if let Ok(term) = env::var("TERM") {
        // Common terminal identifiers
        match term.as_str() {
            "xterm-256color" | "xterm" => {
                // Try to get more specific terminal info
                if let Ok(output) = Command::new("ps")
                    .args(&["-o", "comm=", "-p", &std::process::id().to_string()])
                    .output()
                {
                    let parent = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !parent.is_empty() && parent != "sh" && parent != "bash" {
                        return parent;
                    }
                }
                return "xterm".to_string();
            }
            "screen" => return "GNU Screen".to_string(),
            "tmux" => return "tmux".to_string(),
            _ => {
                if term.contains("kitty") {
                    return "Kitty".to_string();
                } else if term.contains("alacritty") {
                    return "Alacritty".to_string();
                }
            }
        }
    }

    "Unknown Terminal".to_string()
}
