use std::time::Duration;

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Row, Table, TableState};
use ratatui::Frame;

use super::block;
use crate::app::sort_processes;
use crate::collector::{ProcessSnapshot, ProcessState};
use crate::state::AppState;
use crate::util::{format_bytes, format_duration, usage_color};

pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
    let snap = match &state.current_snapshot {
        Some(s) => s,
        None => return,
    };

    let total = snap.processes.len();
    let running = snap
        .processes
        .iter()
        .filter(|p| p.state == ProcessState::Run)
        .count();
    let sleeping = snap
        .processes
        .iter()
        .filter(|p| matches!(p.state, ProcessState::Sleep | ProcessState::Idle))
        .count();
    let title = format!(
        "Processes · Total {} · Running {} · Sleeping {}",
        total, running, sleeping
    );
    let outer = block(&title, state);
    let inner = outer.inner(area);
    f.render_widget(outer, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(3)])
        .split(inner);

    let arrow = if state.process_sort_ascending { "▲" } else { "▼" };
    let filter_style = if state.process_filter_editing {
        Style::default()
            .fg(state.theme.selected_fg)
            .bg(state.theme.selected_bg)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(state.theme.text)
    };
    let filter_display = if state.process_filter_editing {
        format!("{}_", state.process_filter)
    } else if state.process_filter.is_empty() {
        "(press / to filter)".to_string()
    } else {
        state.process_filter.clone()
    };
    let header_line = Line::from(vec![
        Span::styled("Filter ", Style::default().fg(state.theme.dim)),
        Span::styled(format!("[{}]", filter_display), filter_style),
        Span::raw("    "),
        Span::styled("Sort ", Style::default().fg(state.theme.dim)),
        Span::styled(
            format!("{} {}", state.process_sort.label(), arrow),
            Style::default()
                .fg(state.theme.primary)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    f.render_widget(Paragraph::new(header_line), chunks[0]);

    let filter_lower = state.process_filter.to_lowercase();
    let mut procs: Vec<&ProcessSnapshot> = snap
        .processes
        .iter()
        .filter(|p| filter_lower.is_empty() || p.name.to_lowercase().contains(&filter_lower))
        .collect();
    sort_processes(&mut procs, state.process_sort, state.process_sort_ascending);

    let mut header_cols = vec!["PID", "USER", "NAME", "CPU%", "MEM%"];
    if state.show_gpu_column {
        header_cols.push("VRAM");
    }
    header_cols.push("STATE");
    header_cols.push("TIME");
    let header = Row::new(header_cols).style(
        Style::default()
            .fg(state.theme.header)
            .add_modifier(Modifier::BOLD),
    );
    let rows: Vec<Row> = procs
        .iter()
        .map(|p| {
            let cpu_color = usage_color(p.cpu_usage);
            let mem_color = usage_color(p.memory_usage);
            let time = format_duration(Duration::from_secs(p.cumulative_cpu_time));
            let mut cells = vec![
                Span::raw(p.pid.to_string()),
                Span::raw(truncate(&p.user, 10)),
                Span::raw(truncate(&p.name, 26)),
                Span::styled(format!("{:.1}", p.cpu_usage), Style::default().fg(cpu_color)),
                Span::styled(
                    format!("{:.1}", p.memory_usage),
                    Style::default().fg(mem_color),
                ),
            ];
            if state.show_gpu_column {
                cells.push(Span::raw(
                    p.gpu_memory.map(format_bytes).unwrap_or_else(|| "---".into()),
                ));
            }
            cells.push(Span::raw(p.state.label().to_string()));
            cells.push(Span::raw(time));
            Row::new(cells)
        })
        .collect();

    let mut widths = vec![
        Constraint::Length(7),
        Constraint::Length(11),
        Constraint::Length(28),
        Constraint::Length(7),
        Constraint::Length(7),
    ];
    if state.show_gpu_column {
        widths.push(Constraint::Length(10));
    }
    widths.push(Constraint::Length(7));
    widths.push(Constraint::Min(8));
    let table = Table::new(rows, widths)
    .header(header)
    .row_highlight_style(
        Style::default()
            .fg(state.theme.selected_fg)
            .bg(state.theme.selected_bg)
            .add_modifier(Modifier::BOLD),
    );
    let mut ts = TableState::default();
    if !procs.is_empty() {
        ts.select(Some(state.process_selected_index.min(procs.len() - 1)));
    }
    f.render_stateful_widget(table, chunks[1], &mut ts);
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let t: String = s.chars().take(max - 1).collect();
        format!("{}…", t)
    }
}
