use sysinfo::{Components, CpuRefreshKind, RefreshKind, System};

use super::Collector;

#[derive(Clone, Default)]
pub struct CpuSnapshot {
    pub cores: Vec<CoreInfo>,
    pub average_usage: f32,
    pub package_temp: Option<f32>,
}

#[derive(Clone, Default)]
pub struct CoreInfo {
    pub usage: f32,
    pub frequency_mhz: u64,
    pub temperature: Option<f32>,
}

pub struct CpuCollector {
    system: System,
    components: Components,
}

impl CpuCollector {
    pub fn new() -> Self {
        let system = System::new_with_specifics(
            RefreshKind::nothing().with_cpu(CpuRefreshKind::everything()),
        );
        Self {
            system,
            components: Components::new_with_refreshed_list(),
        }
    }
}

impl Default for CpuCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl Collector for CpuCollector {
    type Snapshot = CpuSnapshot;

    fn collect(&mut self) -> Option<CpuSnapshot> {
        self.system
            .refresh_cpu_specifics(CpuRefreshKind::everything());
        self.components.refresh(false);

        let mut per_core_temps: Vec<Option<f32>> = Vec::new();
        let mut package_temp: Option<f32> = None;
        for c in self.components.list() {
            let label = c.label().to_lowercase();
            if let Some(temp) = c.temperature() {
                if label.contains("package id 0") || label.contains("package") {
                    package_temp = package_temp.or(Some(temp));
                } else if let Some(idx) = parse_core_index(&label) {
                    while per_core_temps.len() <= idx {
                        per_core_temps.push(None);
                    }
                    per_core_temps[idx] = Some(temp);
                }
            }
        }

        let cores: Vec<CoreInfo> = self
            .system
            .cpus()
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let physical_idx = i / 2;
                CoreInfo {
                    usage: c.cpu_usage(),
                    frequency_mhz: c.frequency(),
                    temperature: per_core_temps.get(physical_idx).copied().flatten(),
                }
            })
            .collect();

        let average_usage = if cores.is_empty() {
            0.0
        } else {
            cores.iter().map(|c| c.usage).sum::<f32>() / cores.len() as f32
        };

        Some(CpuSnapshot {
            cores,
            average_usage,
            package_temp,
        })
    }

    fn name(&self) -> &'static str {
        "cpu"
    }
}

fn parse_core_index(label: &str) -> Option<usize> {
    // Examples: "Core 0", "core 3", "coretemp Core 1"
    let lower = label.to_lowercase();
    let pos = lower.find("core ")?;
    let rest = &lower[pos + 5..];
    let num: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    num.parse().ok()
}
