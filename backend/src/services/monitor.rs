use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use sysinfo::{Disks, Networks, Pid, ProcessesToUpdate, Signal, System};

use crate::api::system::{DiskInfo, NetworkInfo, SystemInfo, SystemStats};
use crate::api::process::ProcessInfo;
use crate::error::{AppError, AppResult};

/// Configurable refresh intervals to avoid excessive system calls
const PROCESS_REFRESH_INTERVAL: Duration = Duration::from_secs(2);
const SYSTEM_REFRESH_INTERVAL: Duration = Duration::from_secs(1);
const DISK_REFRESH_INTERVAL: Duration = Duration::from_secs(5);
const NETWORK_REFRESH_INTERVAL: Duration = Duration::from_secs(2);

pub struct SystemMonitor {
    system: Arc<Mutex<MonitorState>>,
}

struct MonitorState {
    system: System,
    disks: Disks,
    networks: Networks,
    last_process_refresh: Instant,
    last_system_refresh: Instant,
    last_disk_refresh: Instant,
    last_network_refresh: Instant,
}

impl SystemMonitor {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        let now = Instant::now();

        Self {
            system: Arc::new(Mutex::new(MonitorState {
                system,
                disks: Disks::new_with_refreshed_list(),
                networks: Networks::new_with_refreshed_list(),
                last_process_refresh: now,
                last_system_refresh: now,
                last_disk_refresh: now,
                last_network_refresh: now,
            })),
        }
    }

    pub fn get_system_info(&self) -> SystemInfo {
        let state = self.system.lock().unwrap();

        SystemInfo {
            hostname: System::host_name().unwrap_or_else(|| "unknown".to_string()),
            os: System::name().unwrap_or_else(|| "unknown".to_string()),
            os_version: System::os_version().unwrap_or_else(|| "unknown".to_string()),
            kernel: System::kernel_version().unwrap_or_else(|| "unknown".to_string()),
            uptime: System::uptime(),
            cpu_count: state.system.cpus().len(),
            total_memory: state.system.total_memory(),
        }
    }

    pub fn get_stats(&self) -> SystemStats {
        let mut state = self.system.lock().unwrap();
        let now = Instant::now();

        // Refresh CPU and memory only if needed
        if now.duration_since(state.last_system_refresh) >= SYSTEM_REFRESH_INTERVAL {
            state.system.refresh_cpu_usage();
            state.system.refresh_memory();
            state.last_system_refresh = now;
        }

        // Refresh disks only if needed
        if now.duration_since(state.last_disk_refresh) >= DISK_REFRESH_INTERVAL {
            state.disks.refresh(true);
            state.last_disk_refresh = now;
        }

        // Refresh networks only if needed
        if now.duration_since(state.last_network_refresh) >= NETWORK_REFRESH_INTERVAL {
            state.networks.refresh(true);
            state.last_network_refresh = now;
        }

        let cpu_usage = state.system.global_cpu_usage();
        let memory_used = state.system.used_memory();
        let memory_total = state.system.total_memory();

        let disk_infos: Vec<DiskInfo> = state.disks.iter().map(|disk| {
            let total = disk.total_space();
            let available = disk.available_space();
            let used = total.saturating_sub(available);

            DiskInfo {
                name: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                total,
                used,
                available,
                percent: if total > 0 { (used as f32 / total as f32) * 100.0 } else { 0.0 },
            }
        }).collect();

        let network_infos: Vec<NetworkInfo> = state.networks.iter().map(|(name, data)| {
            NetworkInfo {
                name: name.to_string(),
                rx_bytes: data.total_received(),
                tx_bytes: data.total_transmitted(),
                rx_packets: data.total_packets_received(),
                tx_packets: data.total_packets_transmitted(),
            }
        }).collect();

        let load_avg = System::load_average();

        SystemStats {
            cpu_usage,
            memory_used,
            memory_total,
            memory_percent: if memory_total > 0 {
                (memory_used as f32 / memory_total as f32) * 100.0
            } else {
                0.0
            },
            swap_used: state.system.used_swap(),
            swap_total: state.system.total_swap(),
            disks: disk_infos,
            networks: network_infos,
            load_avg: [load_avg.one, load_avg.five, load_avg.fifteen],
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    pub fn get_processes(&self) -> Vec<ProcessInfo> {
        let mut state = self.system.lock().unwrap();
        let now = Instant::now();

        // Only refresh processes if enough time has passed
        if now.duration_since(state.last_process_refresh) >= PROCESS_REFRESH_INTERVAL {
            // Refresh processes (sysinfo 0.33 compatible API)
            state
                .system
                .refresh_processes(ProcessesToUpdate::All, true);
            state.last_process_refresh = now;
        }

        state.system.processes().iter().map(|(pid, process)| {
            let cmd_vec: Vec<String> = process.cmd()
                .iter()
                .map(|s| s.to_string_lossy().to_string())
                .collect();

            ProcessInfo {
                pid: pid.as_u32(),
                name: process.name().to_string_lossy().to_string(),
                cmd: cmd_vec,
                cpu_usage: process.cpu_usage(),
                memory: process.memory(),
                status: format!("{:?}", process.status()),
                user: process
                    .user_id()
                    .map(|u| u.to_string())
                    .unwrap_or_else(|| "unknown".to_string()),
                start_time: process.start_time(),
            }
        }).collect()
    }

    pub fn kill_process(&self, pid: u32) -> AppResult<()> {
        let mut state = self.system.lock().unwrap();
        let pid = Pid::from_u32(pid);

        // Refresh process list before action (sysinfo 0.33 compatible API)
        state
            .system
            .refresh_processes(ProcessesToUpdate::All, false);

        if let Some(process) = state.system.process(pid) {
            if process.kill_with(Signal::Kill).is_some() {
                Ok(())
            } else {
                Err(AppError::System("Failed to kill process".to_string()))
            }
        } else {
            Err(AppError::NotFound(format!("Process {} not found", pid)))
        }
    }

    pub fn stop_process(&self, pid: u32) -> AppResult<()> {
        #[cfg(not(target_os = "windows"))]
        {
            let mut state = self.system.lock().unwrap();
            let pid = Pid::from_u32(pid);

            state
                .system
                .refresh_processes(ProcessesToUpdate::All, false);

            if let Some(process) = state.system.process(pid) {
                if process.kill_with(Signal::Stop).is_some() {
                    Ok(())
                } else {
                    Err(AppError::System("Failed to stop process".to_string()))
                }
            } else {
                Err(AppError::NotFound(format!("Process {} not found", pid)))
            }
        }

        #[cfg(target_os = "windows")]
        {
            Err(AppError::System("Process stop/resume is not supported on Windows".to_string()))
        }
    }

    pub fn resume_process(&self, pid: u32) -> AppResult<()> {
        #[cfg(not(target_os = "windows"))]
        {
            let mut state = self.system.lock().unwrap();
            let pid = Pid::from_u32(pid);

            state
                .system
                .refresh_processes(ProcessesToUpdate::All, false);

            if let Some(process) = state.system.process(pid) {
                if process.kill_with(Signal::Continue).is_some() {
                    Ok(())
                } else {
                    Err(AppError::System("Failed to resume process".to_string()))
                }
            } else {
                Err(AppError::NotFound(format!("Process {} not found", pid)))
            }
        }

        #[cfg(target_os = "windows")]
        {
            Err(AppError::System("Process stop/resume is not supported on Windows".to_string()))
        }
    }

    /// Force refresh all data (useful for manual refresh)
    pub fn force_refresh_all(&self) {
        let mut state = self.system.lock().unwrap();
        let now = Instant::now();

        state.system.refresh_all();
        state.disks.refresh(true);
        state.networks.refresh(true);

        state.last_process_refresh = now;
        state.last_system_refresh = now;
        state.last_disk_refresh = now;
        state.last_network_refresh = now;
    }
}

impl Clone for SystemMonitor {
    fn clone(&self) -> Self {
        Self {
            system: Arc::clone(&self.system),
        }
    }
}

impl Default for SystemMonitor {
    fn default() -> Self {
        Self::new()
    }
}
