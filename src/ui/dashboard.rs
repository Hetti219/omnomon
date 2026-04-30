use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph, Row, Table};
use ratatui::Frame;

use super::block;
use crate::app::sort_processes;
use crate::collector::{BatteryState, ProcessSnapshot};
use crate::state::{AppState, ProcessSortColumn, SystemSnapshot};
use crate::util::{
    bar, battery_color, format_bytes, format_duration, format_rate, format_temp, temp_color,
    usage_color,
};

pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
    let snap = match &state.current_snapshot {
        Some(s) => s,
        None => {
            f.render_widget(Paragraph::new("Collecting…"), area);
            return;
        }
    };

    if area.width < 100 {
        render_stacked(f, area, state, snap);
    } else {
        render_grid(f, area, state, snap);
    }
}

fn render_grid(f: &mut Frame, area: Rect, state: &AppState, snap: &SystemSnapshot) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(18),
            Constraint::Percentage(17),
            Constraint::Percentage(35),
        ])
        .split(area);

    let split_h = |r: Rect| {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(r)
    };

    let r1 = split_h(rows[0]);
    let r2 = split_h(rows[1]);
    let r3 = split_h(rows[2]);
    let r4 = split_h(rows[3]);

    cpu_panel(f, r1[0], state, snap);
    gpu_panel(f, r1[1], state, snap);
    memory_panel(f, r2[0], state, snap);
    if state.show_battery {
        battery_panel(f, r2[1], state, snap);
    } else {
        network_panel(f, r2[1], state, snap);
    }
    if state.show_disk {
        disk_panel(f, r3[0], state, snap);
        network_panel(f, r3[1], state, snap);
    } else {
        network_panel(f, r3[0], state, snap);
        process_panel(f, r3[1], state, snap);
    }
    if state.show_thermal {
        thermal_panel(f, r4[0], state, snap);
        process_panel(f, r4[1], state, snap);
    } else {
        process_panel(f, rows[3], state, snap);
    }
}

fn render_stacked(f: &mut Frame, area: Rect, state: &AppState, snap: &SystemSnapshot) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(28),
            Constraint::Percentage(20),
            Constraint::Percentage(14),
            Constraint::Percentage(38),
        ])
        .split(area);
    cpu_panel(f, rows[0], state, snap);
    if snap.gpu.is_some() {
        let split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rows[1]);
        memory_panel(f, split[0], state, snap);
        gpu_panel(f, split[1], state, snap);
    } else {
        memory_panel(f, rows[1], state, snap);
    }
    let row3 = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[2]);
    disk_panel(f, row3[0], state, snap);
    if snap.battery.is_some() {
        battery_panel(f, row3[1], state, snap);
    } else {
        network_panel(f, row3[1], state, snap);
    }
    process_panel(f, rows[3], state, snap);
}

fn cpu_panel(f: &mut Frame, area: Rect, state: &AppState, snap: &SystemSnapshot) {
    let outer = block("CPU", state);
    let inner = outer.inner(area);
    f.render_widget(outer, area);

    let chart_h = (inner.height / 3).max(3);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(chart_h), Constraint::Min(0)])
        .split(inner);

    let data: Vec<(f64, f64)> = state
        .cpu_history
        .iter_ordered()
        .enumerate()
        .map(|(i, v)| (i as f64, *v as f64))
        .collect();
    let max_x = (state.cpu_history.capacity() as f64).max(1.0);
    mini_line(f, chunks[0], &data, max_x, [0.0, 100.0], state.theme.primary);

    let core_count = snap.cpu.cores.len();
    let space = chunks[1].height as usize;
    let max_cores = space.saturating_sub(1);
    let mut lines: Vec<Line> = Vec::new();
    for (i, c) in snap.cpu.cores.iter().take(max_cores).enumerate() {
        lines.push(Line::from(vec![
            Span::styled(
                format!("C{:<2} ", i),
                Style::default().fg(state.theme.dim),
            ),
            Span::styled(
                bar(c.usage, 100.0, 12),
                Style::default().fg(usage_color(c.usage)),
            ),
            Span::raw(format!(" {:>5.1}%", c.usage)),
        ]));
    }
    if core_count > max_cores {
        lines.push(Line::from(Span::styled(
            format!("… {} more cores", core_count - max_cores),
            Style::default().fg(state.theme.dim),
        )));
    }
    let pkg = snap
        .cpu
        .package_temp
        .map(|t| format_temp(t, state.fahrenheit))
        .unwrap_or_else(|| "--".into());
    lines.push(Line::from(vec![
        Span::styled(
            "AVG ",
            Style::default()
                .fg(state.theme.header)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{:>5.1}%  ", snap.cpu.average_usage),
            Style::default().fg(usage_color(snap.cpu.average_usage)),
        ),
        Span::raw(format!("Pkg {}", pkg)),
    ]));
    f.render_widget(Paragraph::new(lines), chunks[1]);
}

fn gpu_panel(f: &mut Frame, area: Rect, state: &AppState, snap: &SystemSnapshot) {
    let g = match &snap.gpu {
        Some(g) => g,
        None => {
            let outer = block("GPU", state);
            let inner = outer.inner(area);
            f.render_widget(outer, area);
            f.render_widget(
                Paragraph::new("No NVIDIA GPU detected.")
                    .style(Style::default().fg(state.theme.dim)),
                inner,
            );
            return;
        }
    };
    let title = format!("GPU · {}", truncate(&g.name, 30));
    let outer = block(&title, state);
    let inner = outer.inner(area);
    f.render_widget(outer, area);

    let chart_h = (inner.height / 3).max(3);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(chart_h), Constraint::Min(0)])
        .split(inner);

    let data: Vec<(f64, f64)> = state
        .gpu_util_history
        .iter_ordered()
        .enumerate()
        .map(|(i, v)| (i as f64, *v as f64))
        .collect();
    let max_x = (state.gpu_util_history.capacity() as f64).max(1.0);
    mini_line(f, chunks[0], &data, max_x, [0.0, 100.0], state.theme.primary);

    let mem_pct = if g.memory_total > 0 {
        g.memory_used as f32 / g.memory_total as f32 * 100.0
    } else {
        0.0
    };
    let pwr_pct = if g.power_limit > 0.0 {
        (g.power_draw / g.power_limit) as f32 * 100.0
    } else {
        0.0
    };
    let lines = vec![
        Line::from(vec![
            Span::styled("Util ", Style::default().fg(state.theme.dim)),
            Span::styled(
                bar(g.utilization as f32, 100.0, 12),
                Style::default().fg(usage_color(g.utilization as f32)),
            ),
            Span::raw(format!(" {}%", g.utilization)),
        ]),
        Line::from(vec![
            Span::styled("VRAM ", Style::default().fg(state.theme.dim)),
            Span::styled(
                bar(mem_pct, 100.0, 12),
                Style::default().fg(usage_color(mem_pct)),
            ),
            Span::raw(format!(
                " {}/{}",
                format_bytes(g.memory_used),
                format_bytes(g.memory_total)
            )),
        ]),
        Line::from(vec![
            Span::styled("Pwr  ", Style::default().fg(state.theme.dim)),
            Span::styled(
                bar(pwr_pct, 100.0, 12),
                Style::default().fg(usage_color(pwr_pct)),
            ),
            Span::raw(format!(" {:.0}/{:.0} W", g.power_draw, g.power_limit)),
        ]),
        Line::from(vec![
            Span::styled(
                format!("Temp {}", format_temp(g.temperature as f32, state.fahrenheit)),
                Style::default().fg(temp_color(g.temperature as f32)),
            ),
            Span::raw(format!(
                "   Fan {}",
                g.fan_speed
                    .map(|s| format!("{}%", s))
                    .unwrap_or_else(|| "--".into())
            )),
        ]),
        Line::from(Span::styled(
            format!(
                "Driver {}  CUDA {}",
                g.driver_version,
                g.cuda_version.clone().unwrap_or_else(|| "?".into())
            ),
            Style::default().fg(state.theme.dim),
        )),
    ];
    f.render_widget(Paragraph::new(lines), chunks[1]);
}

fn memory_panel(f: &mut Frame, area: Rect, state: &AppState, snap: &SystemSnapshot) {
    let outer = block("Memory", state);
    let inner = outer.inner(area);
    f.render_widget(outer, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(0)])
        .split(inner);

    let mem_pct = if snap.memory.total > 0 {
        snap.memory.used as f32 / snap.memory.total as f32 * 100.0
    } else {
        0.0
    };
    let swap_pct = if snap.memory.swap_total > 0 {
        snap.memory.swap_used as f32 / snap.memory.swap_total as f32 * 100.0
    } else {
        0.0
    };
    let lines = vec![
        Line::from(vec![
            Span::styled("RAM  ", Style::default().fg(state.theme.dim)),
            Span::styled(
                bar(mem_pct, 100.0, 18),
                Style::default().fg(usage_color(mem_pct)),
            ),
            Span::raw(format!(
                " {}/{}",
                format_bytes(snap.memory.used),
                format_bytes(snap.memory.total)
            )),
        ]),
        Line::from(vec![
            Span::styled("Swap ", Style::default().fg(state.theme.dim)),
            Span::styled(
                bar(swap_pct, 100.0, 18),
                Style::default().fg(usage_color(swap_pct)),
            ),
            Span::raw(format!(
                " {}/{}",
                format_bytes(snap.memory.swap_used),
                format_bytes(snap.memory.swap_total)
            )),
        ]),
    ];
    f.render_widget(Paragraph::new(lines), chunks[0]);

    let data: Vec<(f64, f64)> = state
        .mem_history
        .iter_ordered()
        .enumerate()
        .map(|(i, v)| (i as f64, *v))
        .collect();
    let max_x = (state.mem_history.capacity() as f64).max(1.0);
    mini_line(f, chunks[1], &data, max_x, [0.0, 100.0], state.theme.primary);
}

fn battery_panel(f: &mut Frame, area: Rect, state: &AppState, snap: &SystemSnapshot) {
    let outer = block("Battery", state);
    let inner = outer.inner(area);
    f.render_widget(outer, area);

    let bat = match &snap.battery {
        Some(b) => b,
        None => {
            f.render_widget(
                Paragraph::new("No battery.").style(Style::default().fg(state.theme.dim)),
                inner,
            );
            return;
        }
    };

    let chart_h: u16 = if inner.height >= 6 { 2 } else { 0 };
    let chunks = if chart_h > 0 {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(chart_h), Constraint::Min(0)])
            .split(inner)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(0), Constraint::Min(0)])
            .split(inner)
    };

    if chart_h > 0 {
        let data: Vec<(f64, f64)> = state
            .battery_history
            .iter_ordered()
            .enumerate()
            .map(|(i, v)| (i as f64, *v as f64))
            .collect();
        let max_x = (state.battery_history.capacity() as f64).max(1.0);
        mini_line(
            f,
            chunks[0],
            &data,
            max_x,
            [0.0, 100.0],
            battery_color(bat.charge_percent),
        );
    }

    let icon = match bat.state {
        BatteryState::Charging => "⚡",
        BatteryState::Discharging => "▼",
        BatteryState::Full => "✓",
        BatteryState::Empty => "✗",
        BatteryState::Unknown => "?",
    };
    let eta = match bat.state {
        BatteryState::Charging => bat
            .time_to_full
            .map(format_duration)
            .map(|d| format!("ETA {}", d))
            .unwrap_or_else(|| "ETA --".into()),
        BatteryState::Discharging => bat
            .time_to_empty
            .map(format_duration)
            .map(|d| format!("ETA {}", d))
            .unwrap_or_else(|| "ETA --".into()),
        BatteryState::Full => "Full".into(),
        _ => "ETA --".into(),
    };
    let lines = vec![
        Line::from(vec![
            Span::styled(
                bar(bat.charge_percent, 100.0, 18),
                Style::default().fg(battery_color(bat.charge_percent)),
            ),
            Span::raw(format!(
                " {:.0}% {} {}",
                bat.charge_percent,
                icon,
                bat.state.label()
            )),
        ]),
        Line::from(format!(
            "Health {:.0}%  Rate {:.1}W  {}",
            bat.health_percent, bat.energy_rate, eta
        )),
        Line::from(format!(
            "Cycles {}  {:.2}V  {}",
            bat.cycle_count
                .map(|c| c.to_string())
                .unwrap_or_else(|| "--".into()),
            bat.voltage,
            if bat.ac_connected { "AC" } else { "BAT" }
        )),
    ];
    f.render_widget(Paragraph::new(lines), chunks[1]);
}

fn disk_panel(f: &mut Frame, area: Rect, state: &AppState, snap: &SystemSnapshot) {
    let outer = block("Disk", state);
    let inner = outer.inner(area);
    f.render_widget(outer, area);

    let mut total_r = 0.0f64;
    let mut total_w = 0.0f64;
    for d in &snap.disks {
        total_r += d.read_bytes_per_sec;
        total_w += d.write_bytes_per_sec;
    }

    let mut lines: Vec<Line> = Vec::new();
    let max_disks = (inner.height as usize).saturating_sub(1);
    for d in snap.disks.iter().take(max_disks) {
        let pct = if d.total > 0 {
            d.used as f32 / d.total as f32 * 100.0
        } else {
            0.0
        };
        lines.push(Line::from(vec![
            Span::styled(
                format!("{:<10} ", truncate(&d.mount_point, 10)),
                Style::default().fg(state.theme.text),
            ),
            Span::styled(
                bar(pct, 100.0, 12),
                Style::default().fg(usage_color(pct)),
            ),
            Span::raw(format!(
                " {}/{}",
                format_bytes(d.used),
                format_bytes(d.total)
            )),
        ]));
    }
    lines.push(Line::from(vec![
        Span::styled("R ", Style::default().fg(state.theme.dim)),
        Span::styled(
            format_rate(total_r),
            Style::default()
                .fg(state.theme.primary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("   W ", Style::default().fg(state.theme.dim)),
        Span::styled(
            format_rate(total_w),
            Style::default()
                .fg(state.theme.secondary)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    f.render_widget(Paragraph::new(lines), inner);
}

fn network_panel(f: &mut Frame, area: Rect, state: &AppState, snap: &SystemSnapshot) {
    let title = if snap.network.is_empty() {
        "Network".to_string()
    } else {
        let idx = state.selected_network_interface.min(snap.network.len() - 1);
        format!("Network · {}", snap.network[idx].interface)
    };
    let outer = block(&title, state);
    let inner = outer.inner(area);
    f.render_widget(outer, area);

    if snap.network.is_empty() {
        f.render_widget(
            Paragraph::new("No interfaces.").style(Style::default().fg(state.theme.dim)),
            inner,
        );
        return;
    }
    let idx = state.selected_network_interface.min(snap.network.len() - 1);
    let n = &snap.network[idx];

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(0)])
        .split(inner);

    let lines = vec![
        Line::from(vec![
            Span::styled("↓ ", Style::default().fg(state.theme.primary)),
            Span::styled(
                format_rate(n.rx_bytes_per_sec),
                Style::default()
                    .fg(state.theme.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("   total {}", format_bytes(n.rx_total))),
        ]),
        Line::from(vec![
            Span::styled("↑ ", Style::default().fg(state.theme.secondary)),
            Span::styled(
                format_rate(n.tx_bytes_per_sec),
                Style::default()
                    .fg(state.theme.secondary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("   total {}", format_bytes(n.tx_total))),
        ]),
    ];
    f.render_widget(Paragraph::new(lines), chunks[0]);

    let rx: Vec<(f64, f64)> = state
        .net_rx_history
        .iter_ordered()
        .enumerate()
        .map(|(i, v)| (i as f64, *v))
        .collect();
    let tx: Vec<(f64, f64)> = state
        .net_tx_history
        .iter_ordered()
        .enumerate()
        .map(|(i, v)| (i as f64, *v))
        .collect();
    let max_y = rx
        .iter()
        .chain(tx.iter())
        .map(|(_, v)| *v)
        .fold(1.0_f64, f64::max);
    let max_x = (state.net_rx_history.capacity() as f64).max(1.0);
    let datasets = vec![
        Dataset::default()
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(state.theme.primary))
            .data(&rx),
        Dataset::default()
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(state.theme.secondary))
            .data(&tx),
    ];
    let chart = Chart::new(datasets)
        .x_axis(Axis::default().bounds([0.0, max_x]))
        .y_axis(Axis::default().bounds([0.0, max_y * 1.1]))
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(chart, chunks[1]);
}

fn thermal_panel(f: &mut Frame, area: Rect, state: &AppState, snap: &SystemSnapshot) {
    let outer = block("Thermal", state);
    let inner = outer.inner(area);
    f.render_widget(outer, area);

    let mut lines: Vec<Line> = Vec::new();
    let max_lines = inner.height as usize;
    for z in snap.thermal.zones.iter().take(max_lines) {
        let max = z.critical.unwrap_or(100.0).max(50.0);
        lines.push(Line::from(vec![
            Span::styled(
                format!("{:<10} ", truncate(&z.name, 10)),
                Style::default().fg(state.theme.text),
            ),
            Span::styled(
                bar(z.temp, max, 14),
                Style::default().fg(temp_color(z.temp)),
            ),
            Span::raw(format!(" {}", format_temp(z.temp, state.fahrenheit))),
            Span::styled(
                z.critical
                    .map(|c| format!(" [{}]", format_temp(c, state.fahrenheit)))
                    .unwrap_or_default(),
                Style::default().fg(state.theme.dim),
            ),
        ]));
    }
    if !snap.thermal.fans.is_empty() && lines.len() < max_lines {
        let fan_summary = snap
            .thermal
            .fans
            .iter()
            .map(|fa| format!("{} {}rpm", truncate(&fa.name, 8), fa.rpm))
            .collect::<Vec<_>>()
            .join("  ");
        lines.push(Line::from(Span::styled(
            fan_summary,
            Style::default().fg(state.theme.dim),
        )));
    }
    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "No thermal data.",
            Style::default().fg(state.theme.dim),
        )));
    }
    f.render_widget(Paragraph::new(lines), inner);
}

fn process_panel(f: &mut Frame, area: Rect, state: &AppState, snap: &SystemSnapshot) {
    let outer = block("Top Processes (CPU)", state);
    let inner = outer.inner(area);
    f.render_widget(outer, area);

    let mut procs: Vec<&ProcessSnapshot> = snap.processes.iter().collect();
    sort_processes(&mut procs, ProcessSortColumn::Cpu, false);
    let max = (inner.height as usize).saturating_sub(1);

    let header = Row::new(vec!["PID", "NAME", "CPU%", "MEM%", "GPU"]).style(
        Style::default()
            .fg(state.theme.header)
            .add_modifier(Modifier::BOLD),
    );
    let rows: Vec<Row> = procs
        .iter()
        .take(max)
        .map(|p| {
            let gpu = p.gpu_memory.map(format_bytes).unwrap_or_else(|| "---".into());
            Row::new(vec![
                Span::raw(p.pid.to_string()),
                Span::raw(truncate(&p.name, 18)),
                Span::styled(
                    format!("{:.1}", p.cpu_usage),
                    Style::default().fg(usage_color(p.cpu_usage)),
                ),
                Span::styled(
                    format!("{:.1}", p.memory_usage),
                    Style::default().fg(usage_color(p.memory_usage)),
                ),
                Span::raw(gpu),
            ])
        })
        .collect();
    let table = Table::new(
        rows,
        [
            Constraint::Length(7),
            Constraint::Length(20),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Min(8),
        ],
    )
    .header(header);
    f.render_widget(table, inner);
}

fn mini_line(f: &mut Frame, area: Rect, data: &[(f64, f64)], max_x: f64, y: [f64; 2], color: Color) {
    let datasets = vec![Dataset::default()
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(color))
        .data(data)];
    let chart = Chart::new(datasets)
        .x_axis(Axis::default().bounds([0.0, max_x]))
        .y_axis(Axis::default().bounds(y))
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(chart, area);
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let t: String = s.chars().take(max - 1).collect();
        format!("{}…", t)
    }
}
