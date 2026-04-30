use std::time::Duration;

use sysinfo::{CpuRefreshKind, RefreshKind, System};

use super::Collector;

#[derive(Clone, Default)]
pub struct SystemInfo {
    pub hostname: String,
    pub os_name: String,
    pub kernel_version: String,
    pub uptime: Duration,
    pub load_avg: (f64, f64, f64),
    pub cpu_model: String,
    pub cpu_cores_physical: usize,
    pub cpu_cores_logical: usize,
    pub cpu_arch: String,
}

pub struct SystemInfoCollector {
    info: SystemInfo,
}

impl SystemInfoCollector {
    pub fn new() -> Self {
        let mut sys = System::new_with_specifics(
            RefreshKind::nothing().with_cpu(CpuRefreshKind::nothing().with_cpu_usage()),
        );
        sys.refresh_cpu_specifics(CpuRefreshKind::nothing().with_cpu_usage());

        let hostname = System::host_name().unwrap_or_else(|| "unknown".into());
        let os_name = System::long_os_version()
            .or_else(System::name)
            .unwrap_or_else(|| "Linux".into());
        let kernel_version = System::kernel_version().unwrap_or_else(|| "?".into());
        let cpu_model = sys
            .cpus()
            .first()
            .map(|c| c.brand().trim().to_string())
            .unwrap_or_else(|| "?".into());
        let cpu_cores_logical = sys.cpus().len();
        let cpu_cores_physical =
            System::physical_core_count().unwrap_or(cpu_cores_logical / 2);
        let cpu_arch = System::cpu_arch();
        Self {
            info: SystemInfo {
                hostname,
                os_name,
                kernel_version,
                uptime: Duration::from_secs(0),
                load_avg: (0.0, 0.0, 0.0),
                cpu_model,
                cpu_cores_physical,
                cpu_cores_logical,
                cpu_arch,
            },
        }
    }
}

impl Default for SystemInfoCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl Collector for SystemInfoCollector {
    type Snapshot = SystemInfo;

    fn collect(&mut self) -> Option<SystemInfo> {
        let mut info = self.info.clone();
        info.uptime = Duration::from_secs(System::uptime());
        let la = System::load_average();
        info.load_avg = (la.one, la.five, la.fifteen);
        Some(info)
    }
}
