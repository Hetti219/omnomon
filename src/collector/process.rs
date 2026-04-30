use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System, Users};

use super::{Collector, GpuProcessInfo};

#[derive(Clone, Default)]
pub struct ProcessSnapshot {
    pub pid: u32,
    pub parent_pid: Option<u32>,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub memory_bytes: u64,
    pub user: String,
    pub state: ProcessState,
    pub gpu_usage: Option<f32>,
    pub gpu_memory: Option<u64>,
    pub cumulative_cpu_time: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessState {
    Run,
    Sleep,
    Idle,
    Zombie,
    Stop,
    Other,
}

impl Default for ProcessState {
    fn default() -> Self {
        ProcessState::Other
    }
}

impl ProcessState {
    pub fn label(self) -> &'static str {
        match self {
            ProcessState::Run => "Run",
            ProcessState::Sleep => "Sleep",
            ProcessState::Idle => "Idle",
            ProcessState::Zombie => "Zomb",
            ProcessState::Stop => "Stop",
            ProcessState::Other => "Other",
        }
    }
}

pub struct ProcessCollector {
    system: System,
    users: Users,
    total_memory: u64,
    num_cores: f32,
}

impl ProcessCollector {
    pub fn new() -> Self {
        let mut system = System::new();
        system.refresh_memory();
        let total_memory = system.total_memory().max(1);
        let num_cores = system.cpus().len().max(1) as f32;
        Self {
            system,
            users: Users::new_with_refreshed_list(),
            total_memory,
            num_cores,
        }
    }

    pub fn kill(&self, pid: u32, signal: sysinfo::Signal) -> bool {
        if let Some(p) = self.system.process(Pid::from_u32(pid)) {
            p.kill_with(signal).unwrap_or(false)
        } else {
            false
        }
    }
}

impl Default for ProcessCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl Collector for ProcessCollector {
    type Snapshot = Vec<ProcessSnapshot>;

    fn collect(&mut self) -> Option<Vec<ProcessSnapshot>> {
        self.system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::everything(),
        );
        self.system.refresh_memory();
        self.total_memory = self.system.total_memory().max(1);
        self.users.refresh();

        let mut out = Vec::with_capacity(self.system.processes().len());
        for (pid, proc_) in self.system.processes() {
            let user = proc_
                .user_id()
                .and_then(|uid| self.users.get_user_by_id(uid))
                .map(|u| u.name().to_string())
                .unwrap_or_else(|| "?".into());
            let mem_bytes = proc_.memory();
            let mem_pct = (mem_bytes as f32 / self.total_memory as f32) * 100.0;
            let state = match proc_.status() {
                sysinfo::ProcessStatus::Run => ProcessState::Run,
                sysinfo::ProcessStatus::Sleep => ProcessState::Sleep,
                sysinfo::ProcessStatus::Idle => ProcessState::Idle,
                sysinfo::ProcessStatus::Zombie => ProcessState::Zombie,
                sysinfo::ProcessStatus::Stop => ProcessState::Stop,
                _ => ProcessState::Other,
            };
            out.push(ProcessSnapshot {
                pid: pid.as_u32(),
                parent_pid: proc_.parent().map(|p| p.as_u32()),
                name: proc_.name().to_string_lossy().into_owned(),
                cpu_usage: proc_.cpu_usage() / self.num_cores,
                memory_usage: mem_pct,
                memory_bytes: mem_bytes,
                user,
                state,
                gpu_usage: None,
                gpu_memory: None,
                cumulative_cpu_time: proc_.run_time(),
            });
        }
        Some(out)
    }

    fn name(&self) -> &'static str {
        "process"
    }
}

pub fn merge_gpu_into_processes(
    processes: &mut [ProcessSnapshot],
    gpu_processes: &[GpuProcessInfo],
) {
    use std::collections::HashMap;
    let mut gpu_map: HashMap<u32, &GpuProcessInfo> = HashMap::new();
    for gp in gpu_processes {
        gpu_map.insert(gp.pid, gp);
    }
    for p in processes.iter_mut() {
        if let Some(g) = gpu_map.get(&p.pid) {
            p.gpu_usage = g.gpu_util.map(|u| u as f32);
            p.gpu_memory = Some(g.memory_used);
        }
    }
}
