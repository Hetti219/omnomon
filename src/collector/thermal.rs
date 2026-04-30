use std::fs;
use std::path::Path;

use super::Collector;

#[derive(Clone, Default)]
pub struct ThermalSnapshot {
    pub zones: Vec<ThermalZone>,
    pub fans: Vec<FanInfo>,
}

#[derive(Clone, Default)]
pub struct ThermalZone {
    pub name: String,
    pub temp: f32,
    pub critical: Option<f32>,
}

#[derive(Clone, Default)]
pub struct FanInfo {
    pub name: String,
    pub rpm: u32,
    pub min_rpm: Option<u32>,
    pub max_rpm: Option<u32>,
}

pub struct ThermalCollector;

impl ThermalCollector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ThermalCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl Collector for ThermalCollector {
    type Snapshot = ThermalSnapshot;

    fn collect(&mut self) -> Option<ThermalSnapshot> {
        Some(ThermalSnapshot {
            zones: read_thermal_zones(),
            fans: read_fans(),
        })
    }

    fn name(&self) -> &'static str {
        "thermal"
    }
}

pub fn read_thermal_zones() -> Vec<ThermalZone> {
    let mut zones = Vec::new();
    let base = Path::new("/sys/class/thermal");
    let entries = match fs::read_dir(base) {
        Ok(e) => e,
        Err(_) => return zones,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let fname = match path.file_name().and_then(|s| s.to_str()) {
            Some(s) => s.to_string(),
            None => continue,
        };
        if !fname.starts_with("thermal_zone") {
            continue;
        }
        let temp = read_sysfs_value::<i64>(&path.join("temp")).map(|t| t as f32 / 1000.0);
        let zone_type =
            read_sysfs_string(&path.join("type")).unwrap_or_else(|| "unknown".to_string());
        let trip_point =
            read_sysfs_value::<i64>(&path.join("trip_point_0_temp")).map(|t| t as f32 / 1000.0);
        if let Some(temp) = temp {
            zones.push(ThermalZone {
                name: zone_type,
                temp,
                critical: trip_point,
            });
        }
    }
    zones.sort_by(|a, b| a.name.cmp(&b.name));
    zones
}

pub fn read_fans() -> Vec<FanInfo> {
    let mut fans = Vec::new();
    let base = Path::new("/sys/class/hwmon");
    let entries = match fs::read_dir(base) {
        Ok(e) => e,
        Err(_) => return fans,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let chip_name = read_sysfs_string(&path.join("name")).unwrap_or_default();
        for i in 1..=8 {
            let fan_path = path.join(format!("fan{}_input", i));
            if let Some(rpm) = read_sysfs_value::<u32>(&fan_path) {
                let label = read_sysfs_string(&path.join(format!("fan{}_label", i)))
                    .unwrap_or_else(|| {
                        if chip_name.is_empty() {
                            format!("Fan {}", i)
                        } else {
                            format!("{} fan{}", chip_name, i)
                        }
                    });
                fans.push(FanInfo {
                    name: label,
                    rpm,
                    min_rpm: read_sysfs_value(&path.join(format!("fan{}_min", i))),
                    max_rpm: read_sysfs_value(&path.join(format!("fan{}_max", i))),
                });
            }
        }
    }
    fans
}

fn read_sysfs_value<T: std::str::FromStr>(path: &Path) -> Option<T> {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
}

fn read_sysfs_string(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}
