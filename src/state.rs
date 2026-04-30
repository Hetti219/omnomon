use std::time::{Duration, Instant};

use crate::collector::{
    BatterySnapshot, CpuSnapshot, DiskSnapshot, GpuSnapshot, MemorySnapshot, NetworkSnapshot,
    ProcessSnapshot, SystemInfo, ThermalSnapshot,
};
use crate::config::ResolvedConfig;
use crate::history::RingBuffer;
use crate::ui::theme::Theme;

#[derive(Clone)]
pub struct SystemSnapshot {
    pub timestamp: Instant,
    pub cpu: CpuSnapshot,
    pub memory: MemorySnapshot,
    pub gpu: Option<GpuSnapshot>,
    pub disks: Vec<DiskSnapshot>,
    pub network: Vec<NetworkSnapshot>,
    pub processes: Vec<ProcessSnapshot>,
    pub battery: Option<BatterySnapshot>,
    pub thermal: ThermalSnapshot,
    pub system_info: SystemInfo,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Tab {
    Dashboard,
    Cpu,
    Gpu,
    Memory,
    Disk,
    Network,
    Processes,
    Battery,
    Thermal,
}

impl Tab {
    pub const ALL: [Tab; 9] = [
        Tab::Dashboard,
        Tab::Cpu,
        Tab::Gpu,
        Tab::Memory,
        Tab::Disk,
        Tab::Network,
        Tab::Processes,
        Tab::Battery,
        Tab::Thermal,
    ];

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|&t| t == self).unwrap_or(0)
    }

    pub fn from_index(i: usize) -> Tab {
        Self::ALL[i % Self::ALL.len()]
    }

    pub fn label(self) -> &'static str {
        match self {
            Tab::Dashboard => "Dashboard",
            Tab::Cpu => "CPU",
            Tab::Gpu => "GPU",
            Tab::Memory => "Memory",
            Tab::Disk => "Disk",
            Tab::Network => "Network",
            Tab::Processes => "Processes",
            Tab::Battery => "Battery",
            Tab::Thermal => "Thermal",
        }
    }

    pub fn from_name(name: &str) -> Tab {
        match name.to_ascii_lowercase().as_str() {
            "cpu" => Tab::Cpu,
            "gpu" => Tab::Gpu,
            "memory" | "mem" => Tab::Memory,
            "disk" => Tab::Disk,
            "network" | "net" => Tab::Network,
            "processes" | "process" | "proc" => Tab::Processes,
            "battery" => Tab::Battery,
            "thermal" => Tab::Thermal,
            _ => Tab::Dashboard,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ProcessSortColumn {
    Cpu,
    Memory,
    Pid,
    Name,
    Gpu,
}

impl ProcessSortColumn {
    pub fn next(self) -> Self {
        match self {
            ProcessSortColumn::Cpu => ProcessSortColumn::Memory,
            ProcessSortColumn::Memory => ProcessSortColumn::Pid,
            ProcessSortColumn::Pid => ProcessSortColumn::Name,
            ProcessSortColumn::Name => ProcessSortColumn::Gpu,
            ProcessSortColumn::Gpu => ProcessSortColumn::Cpu,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ProcessSortColumn::Cpu => "CPU%",
            ProcessSortColumn::Memory => "MEM%",
            ProcessSortColumn::Pid => "PID",
            ProcessSortColumn::Name => "NAME",
            ProcessSortColumn::Gpu => "GPU",
        }
    }

    pub fn from_name(name: &str) -> Self {
        match name.to_ascii_lowercase().as_str() {
            "memory" | "mem" => ProcessSortColumn::Memory,
            "pid" => ProcessSortColumn::Pid,
            "name" => ProcessSortColumn::Name,
            "gpu" => ProcessSortColumn::Gpu,
            _ => ProcessSortColumn::Cpu,
        }
    }
}

pub struct AppState {
    pub current_tab: Tab,
    pub current_snapshot: Option<SystemSnapshot>,

    pub cpu_history: RingBuffer<f32>,
    pub per_core_history: Vec<RingBuffer<f32>>,
    pub gpu_util_history: RingBuffer<f32>,
    pub gpu_vram_history: RingBuffer<f64>,
    pub mem_history: RingBuffer<f64>,
    pub swap_history: RingBuffer<f64>,
    pub net_rx_history: RingBuffer<f64>,
    pub net_tx_history: RingBuffer<f64>,
    pub disk_read_history: RingBuffer<f64>,
    pub disk_write_history: RingBuffer<f64>,
    pub battery_history: RingBuffer<f32>,
    pub battery_rate_history: RingBuffer<f64>,

    pub process_sort: ProcessSortColumn,
    pub process_sort_ascending: bool,
    pub process_filter: String,
    pub process_filter_editing: bool,
    pub process_selected_index: usize,
    pub process_scroll_offset: usize,
    pub process_tree_view: bool,

    pub show_help: bool,
    pub selected_network_interface: usize,
    pub graph_time_window: Duration,
    pub theme: Theme,

    pub refresh_rate: Duration,
    pub fahrenheit: bool,
    pub show_gpu_column: bool,
    pub show_battery: bool,
    pub show_thermal: bool,
    pub show_disk: bool,
}

impl AppState {
    pub fn new(cfg: &ResolvedConfig) -> Self {
        let cap = (cfg.graph_time_window.as_secs() as usize).max(60);
        Self {
            current_tab: Tab::from_name(&cfg.default_tab),
            current_snapshot: None,
            cpu_history: RingBuffer::new(cap),
            per_core_history: Vec::new(),
            gpu_util_history: RingBuffer::new(cap),
            gpu_vram_history: RingBuffer::new(cap),
            mem_history: RingBuffer::new(cap),
            swap_history: RingBuffer::new(cap),
            net_rx_history: RingBuffer::new(cap),
            net_tx_history: RingBuffer::new(cap),
            disk_read_history: RingBuffer::new(cap),
            disk_write_history: RingBuffer::new(cap),
            battery_history: RingBuffer::new(cap),
            battery_rate_history: RingBuffer::new(cap),
            process_sort: ProcessSortColumn::from_name(&cfg.default_sort),
            process_sort_ascending: false,
            process_filter: String::new(),
            process_filter_editing: false,
            process_selected_index: 0,
            process_scroll_offset: 0,
            process_tree_view: false,
            show_help: false,
            selected_network_interface: 0,
            graph_time_window: cfg.graph_time_window,
            theme: Theme::by_name(&cfg.theme_name),
            refresh_rate: cfg.refresh_rate,
            fahrenheit: cfg.fahrenheit,
            show_gpu_column: cfg.show_gpu_column,
            show_battery: cfg.show_battery,
            show_thermal: cfg.show_thermal,
            show_disk: cfg.show_disk,
        }
    }

    pub fn cycle_graph_window(&mut self, zoom_in: bool) {
        let new_window = if zoom_in {
            match self.graph_time_window.as_secs() {
                300 => Duration::from_secs(60),
                60 => Duration::from_secs(30),
                _ => Duration::from_secs(30),
            }
        } else {
            match self.graph_time_window.as_secs() {
                30 => Duration::from_secs(60),
                60 => Duration::from_secs(300),
                _ => Duration::from_secs(300),
            }
        };
        self.graph_time_window = new_window;
        let cap = new_window.as_secs() as usize;
        self.cpu_history.resize(cap);
        self.gpu_util_history.resize(cap);
        self.gpu_vram_history.resize(cap);
        self.mem_history.resize(cap);
        self.swap_history.resize(cap);
        self.net_rx_history.resize(cap);
        self.net_tx_history.resize(cap);
        self.disk_read_history.resize(cap);
        self.disk_write_history.resize(cap);
        self.battery_history.resize(cap);
        self.battery_rate_history.resize(cap);
        for h in &mut self.per_core_history {
            h.resize(cap);
        }
    }

    pub fn next_tab(&mut self) {
        self.current_tab = Tab::from_index(self.current_tab.index() + 1);
    }
    pub fn prev_tab(&mut self) {
        let idx = self.current_tab.index();
        let n = Tab::ALL.len();
        self.current_tab = Tab::from_index((idx + n - 1) % n);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{CliArgs, ConfigFile};

    fn cfg() -> ResolvedConfig {
        let args = CliArgs {
            rate: 1000,
            theme: None,
            config: None,
            no_gpu: false,
            fahrenheit: false,
            verbose: false,
        };
        ResolvedConfig::from(&args, &ConfigFile::default())
    }

    #[test]
    fn tab_cycle_forward() {
        let mut s = AppState::new(&cfg());
        assert_eq!(s.current_tab, Tab::Dashboard);
        s.next_tab();
        assert_eq!(s.current_tab, Tab::Cpu);
    }

    #[test]
    fn tab_cycle_back() {
        let mut s = AppState::new(&cfg());
        s.prev_tab();
        assert_eq!(s.current_tab, Tab::Thermal);
    }

    #[test]
    fn sort_cycles() {
        let mut col = ProcessSortColumn::Cpu;
        col = col.next();
        assert_eq!(col, ProcessSortColumn::Memory);
    }
}
