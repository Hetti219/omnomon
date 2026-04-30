use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::symbols;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Axis, Chart, Dataset, GraphType, Paragraph};
use ratatui::Frame;

use super::block;
use crate::state::AppState;
use crate::util::{bar, format_bytes, format_rate, usage_color};

pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
    let snap = match &state.current_snapshot {
        Some(s) => s,
        None => return,
    };
    let outer = block("Disk", state);
    let inner = outer.inner(area);
    f.render_widget(outer, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(3)])
        .split(inner);

    render_chart(f, chunks[0], state);

    let mut lines: Vec<Line> = Vec::new();
    let mut total_read = 0.0;
    let mut total_write = 0.0;
    for d in &snap.disks {
        let pct = if d.total > 0 {
            d.used as f32 / d.total as f32 * 100.0
        } else {
            0.0
        };
        total_read += d.read_bytes_per_sec;
        total_write += d.write_bytes_per_sec;
        lines.push(Line::from(vec![
            Span::styled(
                format!("{:<14} ", truncate(&d.mount_point, 14)),
                Style::default().fg(state.theme.text),
            ),
            Span::styled(bar(pct, 100.0, 18), Style::default().fg(usage_color(pct))),
            Span::raw(format!(
                "  {} / {}  ({:.0}%)  fs:{}  R:{}  W:{}",
                format_bytes(d.used),
                format_bytes(d.total),
                pct,
                d.fs_type,
                format_rate(d.read_bytes_per_sec),
                format_rate(d.write_bytes_per_sec)
            )),
        ]));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(
            "Totals  ",
            Style::default()
                .fg(state.theme.header)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!(
            "Read {}   Write {}",
            format_rate(total_read),
            format_rate(total_write)
        )),
    ]));
    f.render_widget(Paragraph::new(lines), chunks[1]);
}

fn render_chart(f: &mut Frame, area: Rect, state: &AppState) {
    let read: Vec<(f64, f64)> = state
        .disk_read_history
        .iter_ordered()
        .enumerate()
        .map(|(i, v)| (i as f64, *v))
        .collect();
    let write: Vec<(f64, f64)> = state
        .disk_write_history
        .iter_ordered()
        .enumerate()
        .map(|(i, v)| (i as f64, *v))
        .collect();
    let max_y = read
        .iter()
        .chain(write.iter())
        .map(|(_, v)| *v)
        .fold(1.0_f64, f64::max);
    let max_x = (state.disk_read_history.capacity() as f64).max(1.0);
    let datasets = vec![
        Dataset::default()
            .name("Read")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(state.theme.primary))
            .data(&read),
        Dataset::default()
            .name("Write")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(state.theme.secondary))
            .data(&write),
    ];
    let chart = Chart::new(datasets)
        .x_axis(Axis::default().bounds([0.0, max_x]))
        .y_axis(Axis::default().bounds([0.0, max_y * 1.1]))
        .block(
            ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::NONE)
                .title(Span::styled(
                    format!("Disk I/O ({}s)", state.graph_time_window.as_secs()),
                    Style::default().fg(state.theme.dim),
                )),
        );
    f.render_widget(chart, area);
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("…{}", &s[s.len() - (max - 1)..])
    }
}
