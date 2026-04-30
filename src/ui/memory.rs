use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::symbols;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Axis, Chart, Dataset, GraphType, Paragraph};
use ratatui::Frame;

use super::block;
use crate::state::AppState;
use crate::util::{bar, format_bytes, usage_color};

pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
    let snap = match &state.current_snapshot {
        Some(s) => s,
        None => return,
    };
    let outer = block("Memory", state);
    let inner = outer.inner(area);
    f.render_widget(outer, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Length(7), Constraint::Min(0)])
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

    render_chart(f, chunks[0], state);

    let lines = vec![
        Line::from(vec![
            Span::styled("RAM   ", Style::default().fg(state.theme.dim)),
            Span::styled(bar(mem_pct, 100.0, 30), Style::default().fg(usage_color(mem_pct))),
            Span::raw(format!(
                "  {} / {}  ({:.1}%)",
                format_bytes(snap.memory.used),
                format_bytes(snap.memory.total),
                mem_pct
            )),
        ]),
        Line::from(vec![
            Span::styled("Swap  ", Style::default().fg(state.theme.dim)),
            Span::styled(bar(swap_pct, 100.0, 30), Style::default().fg(usage_color(swap_pct))),
            Span::raw(format!(
                "  {} / {}  ({:.1}%)",
                format_bytes(snap.memory.swap_used),
                format_bytes(snap.memory.swap_total),
                swap_pct
            )),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                format!("Available: {}  ", format_bytes(snap.memory.available)),
                Style::default().fg(state.theme.text),
            ),
            Span::styled(
                format!("Cached: {}  ", format_bytes(snap.memory.cached)),
                Style::default().fg(state.theme.text),
            ),
            Span::styled(
                format!("Buffers: {}", format_bytes(snap.memory.buffers)),
                Style::default().fg(state.theme.text),
            ),
        ]),
    ];
    f.render_widget(Paragraph::new(lines), chunks[1]);
}

fn render_chart(f: &mut Frame, area: Rect, state: &AppState) {
    let mem: Vec<(f64, f64)> = state
        .mem_history
        .iter_ordered()
        .enumerate()
        .map(|(i, v)| (i as f64, *v))
        .collect();
    let swap: Vec<(f64, f64)> = state
        .swap_history
        .iter_ordered()
        .enumerate()
        .map(|(i, v)| (i as f64, *v))
        .collect();
    let max_x = (state.mem_history.capacity() as f64).max(1.0);
    let datasets = vec![
        Dataset::default()
            .name("RAM")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(state.theme.primary))
            .data(&mem),
        Dataset::default()
            .name("Swap")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(state.theme.secondary))
            .data(&swap),
    ];
    let chart = Chart::new(datasets)
        .x_axis(Axis::default().bounds([0.0, max_x]))
        .y_axis(Axis::default().bounds([0.0, 100.0]))
        .block(
            ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::NONE)
                .title(Span::styled(
                    format!("RAM/Swap history ({}s)", state.graph_time_window.as_secs()),
                    Style::default()
                        .fg(state.theme.dim)
                        .add_modifier(Modifier::BOLD),
                )),
        );
    f.render_widget(chart, area);
}
