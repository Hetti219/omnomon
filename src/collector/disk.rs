use std::collections::HashMap;
use std::time::Instant;

use sysinfo::Disks;

use super::Collector;

#[derive(Clone, Default)]
pub struct DiskSnapshot {
    pub name: String,
    pub mount_point: String,
    pub fs_type: String,
    pub total: u64,
    pub used: u64,
    pub read_bytes_per_sec: f64,
    pub write_bytes_per_sec: f64,
}

pub struct DiskCollector {
    disks: Disks,
    last: Option<Instant>,
    prev: HashMap<String, (u64, u64)>,
}

impl DiskCollector {
    pub fn new() -> Self {
        Self {
            disks: Disks::new_with_refreshed_list(),
            last: None,
            prev: HashMap::new(),
        }
    }
}

impl Default for DiskCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl Collector for DiskCollector {
    type Snapshot = Vec<DiskSnapshot>;

    fn collect(&mut self) -> Option<Vec<DiskSnapshot>> {
        self.disks.refresh(true);
        let now = Instant::now();
        let dt = self
            .last
            .map(|t| now.duration_since(t).as_secs_f64().max(0.001))
            .unwrap_or(1.0);
        self.last = Some(now);

        let mut out = Vec::new();
        let mut new_prev = HashMap::new();
        for d in self.disks.list() {
            let name = d.name().to_string_lossy().into_owned();
            let mount = d.mount_point().to_string_lossy().into_owned();
            let fs_type = d.file_system().to_string_lossy().into_owned();
            let total = d.total_space();
            let used = total.saturating_sub(d.available_space());
            let usage = d.usage();
            let key = format!("{}|{}", name, mount);

            let (read_per_sec, write_per_sec) = if let Some((prev_r, prev_w)) = self.prev.get(&key)
            {
                let dr = usage.total_read_bytes.saturating_sub(*prev_r) as f64 / dt;
                let dw = usage.total_written_bytes.saturating_sub(*prev_w) as f64 / dt;
                (dr, dw)
            } else {
                (0.0, 0.0)
            };
            new_prev.insert(key, (usage.total_read_bytes, usage.total_written_bytes));

            out.push(DiskSnapshot {
                name,
                mount_point: mount,
                fs_type,
                total,
                used,
                read_bytes_per_sec: read_per_sec,
                write_bytes_per_sec: write_per_sec,
            });
        }
        self.prev = new_prev;
        Some(out)
    }

    fn name(&self) -> &'static str {
        "disk"
    }
}
