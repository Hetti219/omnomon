use super::Collector;

#[derive(Clone, Default)]
pub struct GpuSnapshot {
    pub name: String,
    pub utilization: u32,
    pub memory_used: u64,
    pub memory_total: u64,
    pub temperature: u32,
    pub fan_speed: Option<u32>,
    pub power_draw: f64,
    pub power_limit: f64,
    pub clock_graphics: u32,
    pub clock_memory: u32,
    pub clock_sm: u32,
    pub encoder_util: u32,
    pub decoder_util: u32,
    pub driver_version: String,
    pub cuda_version: Option<String>,
    pub pcie_gen: u32,
    pub pcie_width: u32,
    pub processes: Vec<GpuProcessInfo>,
}

#[derive(Clone, Debug)]
pub struct GpuProcessInfo {
    pub pid: u32,
    pub name: String,
    pub memory_used: u64,
    pub gpu_util: Option<u32>,
    pub process_type: GpuProcessType,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GpuProcessType {
    Graphics,
    Compute,
    Both,
}

#[cfg(feature = "nvidia")]
pub struct GpuCollector {
    nvml: nvml_wrapper::Nvml,
    driver_version: String,
    cuda_version: Option<String>,
}

#[cfg(not(feature = "nvidia"))]
pub struct GpuCollector;

#[cfg(feature = "nvidia")]
impl GpuCollector {
    pub fn try_new() -> Result<Self, String> {
        let nvml = nvml_wrapper::Nvml::init().map_err(|e| e.to_string())?;
        let driver_version = nvml.sys_driver_version().unwrap_or_else(|_| "?".into());
        let cuda_version = nvml.sys_cuda_driver_version().ok().map(|v| {
            let major = nvml_wrapper::cuda_driver_version_major(v);
            let minor = nvml_wrapper::cuda_driver_version_minor(v);
            format!("{}.{}", major, minor)
        });
        Ok(Self {
            nvml,
            driver_version,
            cuda_version,
        })
    }
}

#[cfg(not(feature = "nvidia"))]
impl GpuCollector {
    pub fn try_new() -> Result<Self, String> {
        Err("nvidia feature not compiled in".into())
    }
}

impl Collector for GpuCollector {
    type Snapshot = GpuSnapshot;

    #[cfg(feature = "nvidia")]
    fn collect(&mut self) -> Option<GpuSnapshot> {
        use nvml_wrapper::enum_wrappers::device::{Clock, TemperatureSensor};
        use nvml_wrapper::enums::device::UsedGpuMemory;

        let device = self.nvml.device_by_index(0).ok()?;
        let name = device.name().unwrap_or_else(|_| "NVIDIA GPU".into());
        let util = device.utilization_rates().ok();
        let mem = device.memory_info().ok();
        let temp = device.temperature(TemperatureSensor::Gpu).unwrap_or(0);
        let fan = device.fan_speed(0).ok();
        let power_draw = device.power_usage().ok().map(|p| p as f64 / 1000.0).unwrap_or(0.0);
        let power_limit = device
            .enforced_power_limit()
            .ok()
            .map(|p| p as f64 / 1000.0)
            .unwrap_or(0.0);
        let clock_graphics = device.clock_info(Clock::Graphics).unwrap_or(0);
        let clock_memory = device.clock_info(Clock::Memory).unwrap_or(0);
        let clock_sm = device.clock_info(Clock::SM).unwrap_or(0);
        let encoder_util = device.encoder_utilization().ok().map(|u| u.utilization).unwrap_or(0);
        let decoder_util = device.decoder_utilization().ok().map(|u| u.utilization).unwrap_or(0);
        let pcie_gen = device.current_pcie_link_gen().unwrap_or(0);
        let pcie_width = device.current_pcie_link_width().unwrap_or(0);

        let mut procs: Vec<GpuProcessInfo> = Vec::new();

        let to_used = |u: UsedGpuMemory| -> u64 {
            match u {
                UsedGpuMemory::Used(v) => v,
                UsedGpuMemory::Unavailable => 0,
            }
        };

        let resolve_name = |pid: u32| -> String {
            self.nvml
                .sys_process_name(pid, 64)
                .unwrap_or_else(|_| format!("pid {}", pid))
        };

        if let Ok(list) = device.running_compute_processes() {
            for p in list {
                procs.push(GpuProcessInfo {
                    pid: p.pid,
                    name: resolve_name(p.pid),
                    memory_used: to_used(p.used_gpu_memory),
                    gpu_util: None,
                    process_type: GpuProcessType::Compute,
                });
            }
        }
        if let Ok(list) = device.running_graphics_processes() {
            for p in list {
                let mem_used = to_used(p.used_gpu_memory);
                if let Some(existing) = procs.iter_mut().find(|x| x.pid == p.pid) {
                    existing.process_type = GpuProcessType::Both;
                    existing.memory_used = existing.memory_used.max(mem_used);
                } else {
                    procs.push(GpuProcessInfo {
                        pid: p.pid,
                        name: resolve_name(p.pid),
                        memory_used: mem_used,
                        gpu_util: None,
                        process_type: GpuProcessType::Graphics,
                    });
                }
            }
        }

        Some(GpuSnapshot {
            name,
            utilization: util.as_ref().map(|u| u.gpu).unwrap_or(0),
            memory_used: mem.as_ref().map(|m| m.used).unwrap_or(0),
            memory_total: mem.as_ref().map(|m| m.total).unwrap_or(0),
            temperature: temp,
            fan_speed: fan,
            power_draw,
            power_limit,
            clock_graphics,
            clock_memory,
            clock_sm,
            encoder_util,
            decoder_util,
            driver_version: self.driver_version.clone(),
            cuda_version: self.cuda_version.clone(),
            pcie_gen,
            pcie_width,
            processes: procs,
        })
    }

    #[cfg(not(feature = "nvidia"))]
    fn collect(&mut self) -> Option<GpuSnapshot> {
        None
    }
}
