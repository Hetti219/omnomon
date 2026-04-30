use std::time::Duration;

pub mod battery;
pub mod cpu;
pub mod disk;
pub mod gpu;
pub mod memory;
pub mod network;
pub mod process;
pub mod system_info;
pub mod thermal;

pub use battery::{BatteryCollector, BatterySnapshot, BatteryState};
pub use cpu::{CoreInfo, CpuCollector, CpuSnapshot};
pub use disk::{DiskCollector, DiskSnapshot};
pub use gpu::{GpuCollector, GpuProcessInfo, GpuProcessType, GpuSnapshot};
pub use memory::{MemoryCollector, MemorySnapshot};
pub use network::{NetworkCollector, NetworkSnapshot};
pub use process::{ProcessCollector, ProcessSnapshot, ProcessState};
pub use system_info::{SystemInfo, SystemInfoCollector};
pub use thermal::{FanInfo, ThermalCollector, ThermalSnapshot, ThermalZone};

pub trait Collector {
    type Snapshot: Clone;
    fn collect(&mut self) -> Option<Self::Snapshot>;
    fn name(&self) -> &'static str;
}

pub fn duration_secs(d: Duration) -> u64 {
    d.as_secs()
}
