use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::symbols;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Axis, Chart, Dataset, GraphType, Paragraph, Row, Table};
use ratatui::Frame;

use super::block;
use crate::collector::GpuProcessType;
use crate::state::AppState;
use crate::util::{bar, format_bytes, format_temp, temp_color, usage_color};

pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
    let snap = match &state.current_snapshot {
        Some(s) => s,
        None => return,
    };
    let gpu = match &snap.gpu {
        Some(g) => g,
        None => {
            let outer = block("GPU", state);
            let inner = outer.inner(area);
            f.render_widget(outer, area);
            f.render_widget(
                Paragraph::new("No NVIDIA GPU detected (or NVML unavailable).")
                    .style(Style::default().fg(state.theme.dim)),
                inner,
            );
            return;
        }
    };

    let title = format!(
        "GPU · {} · Driver {} · CUDA {}",
        gpu.name,
        gpu.driver_version,
        gpu.cuda_version.clone().unwrap_or_else(|| "?".into())
    );
    let outer = block(&title, state);
    let inner = outer.inner(area);
    f.render_widget(outer, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),
            Constraint::Length(6),
            Constraint::Min(3),
        ])
        .split(inner);

    render_chart(f, rows[0], state);

    let mem_pct = if gpu.memory_total > 0 {
        (gpu.memory_used as f32 / gpu.memory_total as f32) * 100.0
    } else {
        0.0
    };
    let pwr_pct = if gpu.power_limit > 0.0 {
        (gpu.power_draw / gpu.power_limit) as f32 * 100.0
    } else {
        0.0
    };
    let lines = vec![
        Line::from(vec![
            Span::styled("Util  ", Style::default().fg(state.theme.dim)),
            Span::styled(
                bar(gpu.utilization as f32, 100.0, 24),
                Style::default().fg(usage_color(gpu.utilization as f32)),
            ),
            Span::raw(format!(
                "  {}%   Enc {}%   Dec {}%",
                gpu.utilization, gpu.encoder_util, gpu.decoder_util
            )),
        ]),
        Line::from(vec![
            Span::styled("VRAM  ", Style::default().fg(state.theme.dim)),
            Span::styled(
                bar(mem_pct, 100.0, 24),
                Style::default().fg(usage_color(mem_pct)),
            ),
            Span::raw(format!(
                "  {} / {}",
                format_bytes(gpu.memory_used),
                format_bytes(gpu.memory_total)
            )),
        ]),
        Line::from(vec![
            Span::styled("Power ", Style::default().fg(state.theme.dim)),
            Span::styled(
                bar(pwr_pct, 100.0, 24),
                Style::default().fg(usage_color(pwr_pct)),
            ),
            Span::raw(format!("  {:.1} W / {:.1} W", gpu.power_draw, gpu.power_limit)),
        ]),
        Line::from(vec![
            Span::raw(format!(
                "Clocks  graphics {} MHz · memory {} MHz · sm {} MHz",
                gpu.clock_graphics, gpu.clock_memory, gpu.clock_sm
            )),
        ]),
        Line::from(vec![
            Span::styled(
                format!("Temp {}", format_temp(gpu.temperature as f32, state.fahrenheit)),
                Style::default().fg(temp_color(gpu.temperature as f32)),
            ),
            Span::raw(format!(
                "   Fan {}   PCIe Gen{}x{}",
                gpu.fan_speed.map(|f| format!("{}%", f)).unwrap_or("--".into()),
                gpu.pcie_gen,
                gpu.pcie_width
            )),
        ]),
    ];
    f.render_widget(Paragraph::new(lines), rows[1]);

    let header_row = Row::new(vec!["PID", "NAME", "VRAM", "TYPE"]).style(
        Style::default()
            .fg(state.theme.header)
            .add_modifier(Modifier::BOLD),
    );
    let mut data_rows: Vec<Row> = gpu
        .processes
        .iter()
        .map(|p| {
            let t = match p.process_type {
                GpuProcessType::Graphics => "Graphics",
                GpuProcessType::Compute => "Compute",
                GpuProcessType::Both => "Both",
            };
            Row::new(vec![
                p.pid.to_string(),
                p.name.clone(),
                format_bytes(p.memory_used),
                t.into(),
            ])
        })
        .collect();
    if data_rows.is_empty() {
        data_rows.push(Row::new(vec!["—", "(no GPU processes)", "—", "—"]));
    }
    let table = Table::new(
        data_rows,
        [
            Constraint::Length(8),
            Constraint::Min(20),
            Constraint::Length(12),
            Constraint::Length(10),
        ],
    )
    .header(header_row)
    .block(
        ratatui::widgets::Block::default()
            .borders(ratatui::widgets::Borders::TOP)
            .border_style(Style::default().fg(state.theme.dim))
            .title(Span::styled(
                "GPU Processes",
                Style::default().fg(state.theme.dim),
            )),
    );
    f.render_widget(table, rows[2]);
}

fn render_chart(f: &mut Frame, area: Rect, state: &AppState) {
    let util: Vec<(f64, f64)> = state
        .gpu_util_history
        .iter_ordered()
        .enumerate()
        .map(|(i, v)| (i as f64, *v as f64))
        .collect();
    let max_x = (state.gpu_util_history.capacity() as f64).max(1.0);
    let datasets = vec![Dataset::default()
        .name("Util%")
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(state.theme.primary))
        .data(&util)];
    let chart = Chart::new(datasets)
        .x_axis(Axis::default().bounds([0.0, max_x]))
        .y_axis(Axis::default().bounds([0.0, 100.0]))
        .block(
            ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::NONE)
                .title(Span::styled(
                    format!("GPU utilization history ({}s)", state.graph_time_window.as_secs()),
                    Style::default().fg(state.theme.dim),
                )),
        );
    f.render_widget(chart, area);
}
