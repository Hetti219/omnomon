use std::time::Duration;

use battery::units::{
    electric_potential::volt, power::watt, ratio::percent,
    thermodynamic_temperature::degree_celsius, time::second,
};
use battery::{Manager, State};

use super::Collector;

#[derive(Clone, Default)]
pub struct BatterySnapshot {
    pub charge_percent: f32,
    pub state: BatteryState,
    pub time_to_full: Option<Duration>,
    pub time_to_empty: Option<Duration>,
    pub energy_rate: f64,
    pub voltage: f64,
    pub health_percent: f32,
    pub cycle_count: Option<u32>,
    pub temperature: Option<f32>,
    pub ac_connected: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BatteryState {
    Charging,
    Discharging,
    Full,
    Empty,
    Unknown,
}

impl Default for BatteryState {
    fn default() -> Self {
        BatteryState::Unknown
    }
}

impl BatteryState {
    pub fn label(self) -> &'static str {
        match self {
            BatteryState::Charging => "Charging",
            BatteryState::Discharging => "Discharging",
            BatteryState::Full => "Full",
            BatteryState::Empty => "Empty",
            BatteryState::Unknown => "Unknown",
        }
    }
}

pub struct BatteryCollector {
    manager: Manager,
}

impl BatteryCollector {
    pub fn try_new() -> Result<Self, String> {
        let manager = Manager::new().map_err(|e| e.to_string())?;
        let has_any = manager
            .batteries()
            .map(|mut it| it.next().is_some())
            .unwrap_or(false);
        if !has_any {
            return Err("no battery present".into());
        }
        Ok(Self { manager })
    }
}

impl Collector for BatteryCollector {
    type Snapshot = BatterySnapshot;

    fn collect(&mut self) -> Option<BatterySnapshot> {
        let mut iter = self.manager.batteries().ok()?;
        let battery = iter.next()?.ok()?;

        let state = match battery.state() {
            State::Charging => BatteryState::Charging,
            State::Discharging => BatteryState::Discharging,
            State::Full => BatteryState::Full,
            State::Empty => BatteryState::Empty,
            _ => BatteryState::Unknown,
        };

        let charge_percent = battery.state_of_charge().get::<percent>();
        let health_percent = battery.state_of_health().get::<percent>();
        let energy_rate = battery.energy_rate().get::<watt>() as f64;
        let voltage = battery.voltage().get::<volt>() as f64;
        let time_to_full = battery
            .time_to_full()
            .map(|t| Duration::from_secs(t.get::<second>() as u64));
        let time_to_empty = battery
            .time_to_empty()
            .map(|t| Duration::from_secs(t.get::<second>() as u64));
        let temperature = battery.temperature().map(|t| t.get::<degree_celsius>());
        let ac_connected = matches!(state, BatteryState::Charging | BatteryState::Full);

        Some(BatterySnapshot {
            charge_percent,
            state,
            time_to_full,
            time_to_empty,
            energy_rate,
            voltage,
            health_percent,
            cycle_count: battery.cycle_count(),
            temperature,
            ac_connected: ac_connected || ac_present_sysfs(),
        })
    }
}

fn ac_present_sysfs() -> bool {
    if let Ok(entries) = std::fs::read_dir("/sys/class/power_supply") {
        for entry in entries.flatten() {
            let path = entry.path();
            let type_path = path.join("type");
            if let Ok(t) = std::fs::read_to_string(&type_path) {
                if t.trim() == "Mains" {
                    if let Ok(online) = std::fs::read_to_string(path.join("online")) {
                        return online.trim() == "1";
                    }
                }
            }
        }
    }
    false
}
