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
        Span::styled(
            if state.process_tree_view {
                "    Tree on"
            } else {
                ""
            }
            .to_string(),
            Style::default().fg(state.theme.dim),
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

    let header = Row::new(vec![
        "PID", "USER", "NAME", "CPU%", "MEM%", "VRAM", "STATE", "TIME",
    ])
    .style(
        Style::default()
            .fg(state.theme.header)
            .add_modifier(Modifier::BOLD),
    );
    let rows: Vec<Row> = procs
        .iter()
        .map(|p| {
            let cpu_color = usage_color(p.cpu_usage);
            let mem_color = usage_color(p.memory_usage);
            let vram = p.gpu_memory.map(format_bytes).unwrap_or_else(|| "---".into());
            let time = format_duration(Duration::from_secs(p.cumulative_cpu_time));
            Row::new(vec![
                Span::raw(p.pid.to_string()),
                Span::raw(truncate(&p.user, 10)),
                Span::raw(truncate(&p.name, 26)),
                Span::styled(format!("{:.1}", p.cpu_usage), Style::default().fg(cpu_color)),
                Span::styled(
                    format!("{:.1}", p.memory_usage),
                    Style::default().fg(mem_color),
                ),
                Span::raw(vram),
                Span::raw(p.state.label().to_string()),
                Span::raw(time),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(7),
            Constraint::Length(11),
            Constraint::Length(28),
            Constraint::Length(7),
            Constraint::Length(7),
            Constraint::Length(10),
            Constraint::Length(7),
            Constraint::Min(8),
        ],
    )
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
