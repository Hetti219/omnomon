use std::io::Stdout;

use crossterm::event::{Event as CtEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::collector::process::merge_gpu_into_processes;
use crate::collector::{
    BatteryCollector, Collector, CpuCollector, DiskCollector, GpuCollector, MemoryCollector,
    NetworkCollector, ProcessCollector, SystemInfoCollector, ThermalCollector,
};
use crate::config::{CliArgs, ConfigFile, ResolvedConfig};
use crate::event::{AppEvent, EventChannel};
use crate::state::{AppState, ProcessSortColumn, SystemSnapshot, Tab};
use crate::ui;

pub struct DataManager {
    pub cpu: CpuCollector,
    pub memory: MemoryCollector,
    pub gpu: Option<GpuCollector>,
    pub disk: DiskCollector,
    pub network: NetworkCollector,
    pub process: ProcessCollector,
    pub battery: Option<BatteryCollector>,
    pub thermal: ThermalCollector,
    pub system_info: SystemInfoCollector,
}

impl DataManager {
    pub fn new(no_gpu: bool) -> Self {
        let gpu = if no_gpu {
            None
        } else {
            GpuCollector::try_new().ok()
        };
        Self {
            cpu: CpuCollector::new(),
            memory: MemoryCollector::new(),
            gpu,
            disk: DiskCollector::new(),
            network: NetworkCollector::new(),
            process: ProcessCollector::new(),
            battery: BatteryCollector::try_new().ok(),
            thermal: ThermalCollector::new(),
            system_info: SystemInfoCollector::new(),
        }
    }

    pub fn collect_all(&mut self) -> SystemSnapshot {
        let cpu = self.cpu.collect().unwrap_or_default();
        let memory = self.memory.collect().unwrap_or_default();
        let gpu = self.gpu.as_mut().and_then(|g| g.collect());
        let disks = self.disk.collect().unwrap_or_default();
        let network = self.network.collect().unwrap_or_default();
        let mut processes = self.process.collect().unwrap_or_default();
        if let Some(g) = &gpu {
            merge_gpu_into_processes(&mut processes, &g.processes);
        }
        let battery = self.battery.as_mut().and_then(|b| b.collect());
        let thermal = self.thermal.collect().unwrap_or_default();
        let system_info = self.system_info.collect().unwrap_or_default();

        SystemSnapshot {
            cpu,
            memory,
            gpu,
            disks,
            network,
            processes,
            battery,
            thermal,
            system_info,
        }
    }
}

pub struct App {
    pub state: AppState,
    pub data: DataManager,
}

impl App {
    pub fn from_args(args: CliArgs) -> Self {
        let cfg_file = ConfigFile::load(args.config.as_ref());
        let cfg = ResolvedConfig::from(&args, &cfg_file);
        let state = AppState::new(&cfg);
        let data = DataManager::new(cfg.no_gpu);
        Self { state, data }
    }

    pub fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let events = EventChannel::new(self.state.refresh_rate);
        self.tick();
        loop {
            terminal.draw(|f| ui::render(f, &self.state))?;
            match events.rx.recv()? {
                AppEvent::Input(ev) => {
                    if self.handle_input(ev) {
                        break;
                    }
                }
                AppEvent::Tick => {
                    self.tick();
                }
            }
        }
        Ok(())
    }

    fn tick(&mut self) {
        let snap = self.data.collect_all();

        if !self.state.interface_resolved && !snap.network.is_empty() {
            let want = self.state.default_interface.as_str();
            if !want.is_empty() && !want.eq_ignore_ascii_case("auto") {
                if let Some(idx) = snap.network.iter().position(|n| n.interface == want) {
                    self.state.selected_network_interface = idx;
                }
            }
            self.state.interface_resolved = true;
        }

        self.state.cpu_history.push(snap.cpu.average_usage);
        if self.state.per_core_history.len() != snap.cpu.cores.len() {
            let cap = self.state.cpu_history.capacity();
            self.state.per_core_history =
                (0..snap.cpu.cores.len()).map(|_| crate::history::RingBuffer::new(cap)).collect();
        }
        for (i, core) in snap.cpu.cores.iter().enumerate() {
            self.state.per_core_history[i].push(core.usage);
        }
        if let Some(gpu) = &snap.gpu {
            self.state.gpu_util_history.push(gpu.utilization as f32);
            self.state.gpu_vram_history.push(gpu.memory_used as f64);
        }
        let mem_pct = if snap.memory.total > 0 {
            (snap.memory.used as f64 / snap.memory.total as f64) * 100.0
        } else {
            0.0
        };
        self.state.mem_history.push(mem_pct);
        let swap_pct = if snap.memory.swap_total > 0 {
            (snap.memory.swap_used as f64 / snap.memory.swap_total as f64) * 100.0
        } else {
            0.0
        };
        self.state.swap_history.push(swap_pct);

        let active_idx = self
            .state
            .selected_network_interface
            .min(snap.network.len().saturating_sub(1));
        if let Some(net) = snap.network.get(active_idx) {
            self.state.net_rx_history.push(net.rx_bytes_per_sec);
            self.state.net_tx_history.push(net.tx_bytes_per_sec);
        }
        let read_total: f64 = snap.disks.iter().map(|d| d.read_bytes_per_sec).sum();
        let write_total: f64 = snap.disks.iter().map(|d| d.write_bytes_per_sec).sum();
        self.state.disk_read_history.push(read_total);
        self.state.disk_write_history.push(write_total);

        if let Some(bat) = &snap.battery {
            self.state.battery_history.push(bat.charge_percent);
            self.state.battery_rate_history.push(bat.energy_rate);
        }

        self.state.current_snapshot = Some(snap);
    }

    /// Returns true to quit.
    fn handle_input(&mut self, ev: CtEvent) -> bool {
        match ev {
            CtEvent::Key(k) => self.handle_key(k),
            CtEvent::Resize(_, _) => false,
            _ => false,
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind != KeyEventKind::Press {
            return false;
        }
        // Process filter editing mode handled separately
        if self.state.process_filter_editing && self.state.current_tab == Tab::Processes {
            match key.code {
                KeyCode::Esc => {
                    self.state.process_filter_editing = false;
                    self.state.process_filter.clear();
                }
                KeyCode::Enter => {
                    self.state.process_filter_editing = false;
                }
                KeyCode::Backspace => {
                    self.state.process_filter.pop();
                }
                KeyCode::Char(c) => {
                    self.state.process_filter.push(c);
                }
                _ => {}
            }
            return false;
        }

        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return true,
            KeyCode::Char('?') => {
                self.state.show_help = !self.state.show_help;
            }
            KeyCode::Esc => {
                self.state.show_help = false;
            }
            KeyCode::Char('1') => self.state.current_tab = Tab::Dashboard,
            KeyCode::Char('2') => self.state.current_tab = Tab::Cpu,
            KeyCode::Char('3') => self.state.current_tab = Tab::Gpu,
            KeyCode::Char('4') => self.state.current_tab = Tab::Memory,
            KeyCode::Char('5') => self.state.current_tab = Tab::Disk,
            KeyCode::Char('6') => self.state.current_tab = Tab::Network,
            KeyCode::Char('7') => self.state.current_tab = Tab::Processes,
            KeyCode::Char('8') => self.state.current_tab = Tab::Battery,
            KeyCode::Char('9') => self.state.current_tab = Tab::Thermal,
            KeyCode::Tab => self.state.next_tab(),
            KeyCode::BackTab => self.state.prev_tab(),
            KeyCode::Char('+') | KeyCode::Char('=') => self.state.cycle_graph_window(true),
            KeyCode::Char('-') => self.state.cycle_graph_window(false),
            KeyCode::Char('r') => {
                self.tick();
            }
            _ => {
                if self.state.current_tab == Tab::Processes {
                    self.handle_process_key(key);
                } else if self.state.current_tab == Tab::Network {
                    if let KeyCode::Char('n') = key.code {
                        if let Some(snap) = &self.state.current_snapshot {
                            if !snap.network.is_empty() {
                                self.state.selected_network_interface =
                                    (self.state.selected_network_interface + 1)
                                        % snap.network.len();
                            }
                        }
                    }
                }
            }
        }
        false
    }

    fn handle_process_key(&mut self, key: KeyEvent) {
        let total = self
            .state
            .current_snapshot
            .as_ref()
            .map(|s| s.processes.len())
            .unwrap_or(0);
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.state.process_selected_index =
                    self.state.process_selected_index.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.state.process_selected_index + 1 < total {
                    self.state.process_selected_index += 1;
                }
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.state.process_selected_index = 0;
            }
            KeyCode::End | KeyCode::Char('G') => {
                self.state.process_selected_index = total.saturating_sub(1);
            }
            KeyCode::Char('/') => {
                self.state.process_filter_editing = true;
            }
            KeyCode::Char('s') => {
                self.state.process_sort = self.state.process_sort.next();
            }
            KeyCode::Char('S') => {
                self.state.process_sort_ascending = !self.state.process_sort_ascending;
            }
            KeyCode::Char('K') => {
                if let Some(pid) = self.selected_pid() {
                    self.data.process.kill(pid, sysinfo::Signal::Term);
                }
            }
            KeyCode::Char('D') => {
                if let Some(pid) = self.selected_pid() {
                    self.data.process.kill(pid, sysinfo::Signal::Kill);
                }
            }
            _ => {}
        }
    }

    fn selected_pid(&self) -> Option<u32> {
        let snap = self.state.current_snapshot.as_ref()?;
        let mut procs: Vec<&crate::collector::ProcessSnapshot> = snap
            .processes
            .iter()
            .filter(|p| {
                self.state.process_filter.is_empty()
                    || p.name
                        .to_lowercase()
                        .contains(&self.state.process_filter.to_lowercase())
            })
            .collect();
        sort_processes(&mut procs, self.state.process_sort, self.state.process_sort_ascending);
        procs.get(self.state.process_selected_index).map(|p| p.pid)
    }
}

pub fn sort_processes(
    procs: &mut Vec<&crate::collector::ProcessSnapshot>,
    col: ProcessSortColumn,
    ascending: bool,
) {
    procs.sort_by(|a, b| {
        let ord = match col {
            ProcessSortColumn::Cpu => a
                .cpu_usage
                .partial_cmp(&b.cpu_usage)
                .unwrap_or(std::cmp::Ordering::Equal),
            ProcessSortColumn::Memory => a
                .memory_usage
                .partial_cmp(&b.memory_usage)
                .unwrap_or(std::cmp::Ordering::Equal),
            ProcessSortColumn::Pid => a.pid.cmp(&b.pid),
            ProcessSortColumn::Name => a.name.cmp(&b.name),
            ProcessSortColumn::Gpu => a
                .gpu_memory
                .unwrap_or(0)
                .cmp(&b.gpu_memory.unwrap_or(0)),
        };
        if ascending {
            ord
        } else {
            ord.reverse()
        }
    });
}
