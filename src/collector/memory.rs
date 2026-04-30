use sysinfo::{MemoryRefreshKind, System};

use super::Collector;

#[derive(Clone, Default)]
pub struct MemorySnapshot {
    pub total: u64,
    pub used: u64,
    pub available: u64,
    pub cached: u64,
    pub buffers: u64,
    pub swap_total: u64,
    pub swap_used: u64,
}

pub struct MemoryCollector {
    system: System,
}

impl MemoryCollector {
    pub fn new() -> Self {
        let mut system = System::new();
        system.refresh_memory_specifics(MemoryRefreshKind::everything());
        Self { system }
    }
}

impl Default for MemoryCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl Collector for MemoryCollector {
    type Snapshot = MemorySnapshot;

    fn collect(&mut self) -> Option<MemorySnapshot> {
        self.system
            .refresh_memory_specifics(MemoryRefreshKind::everything());
        let total = self.system.total_memory();
        let used = self.system.used_memory();
        let available = self.system.available_memory();
        let (cached, buffers) = read_meminfo_extra();
        Some(MemorySnapshot {
            total,
            used,
            available,
            cached,
            buffers,
            swap_total: self.system.total_swap(),
            swap_used: self.system.used_swap(),
        })
    }
}

fn read_meminfo_extra() -> (u64, u64) {
    let mut cached = 0u64;
    let mut buffers = 0u64;
    if let Ok(s) = std::fs::read_to_string("/proc/meminfo") {
        for line in s.lines() {
            let mut parts = line.split_whitespace();
            let key = parts.next().unwrap_or("");
            let val: u64 = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0);
            match key {
                "Cached:" => cached = val * 1024,
                "Buffers:" => buffers = val * 1024,
                _ => {}
            }
        }
    }
    (cached, buffers)
}
