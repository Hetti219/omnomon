use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::symbols;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Axis, Chart, Dataset, GraphType, Paragraph};
use ratatui::Frame;

use super::block;
use crate::history::RingBuffer;
use crate::state::AppState;
use crate::util::{bar, format_frequency_mhz, format_temp, temp_color, usage_color};

pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
    let snap = match &state.current_snapshot {
        Some(s) => s,
        None => {
            f.render_widget(Paragraph::new("Collecting…"), area);
            return;
        }
    };

    let title = format!(
        "CPU · {} · {}C/{}T · {}",
        snap.system_info.cpu_model,
        snap.system_info.cpu_cores_physical,
        snap.system_info.cpu_cores_logical,
        snap.system_info.cpu_arch
    );
    let outer = block(&title, state);
    let inner = outer.inner(area);
    f.render_widget(outer, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(3)])
        .split(inner);

    render_chart(f, chunks[0], &state.cpu_history, state);

    let mut lines: Vec<Line> = Vec::new();
    for (i, core) in snap.cpu.cores.iter().enumerate() {
        let bar = bar(core.usage, 100.0, 20);
        let temp_str = match core.temperature {
            Some(t) => format_temp(t, state.fahrenheit),
            None => "  --".into(),
        };
        let line = Line::from(vec![
            Span::styled(
                format!("Core {:<2} ", i),
                Style::default().fg(state.theme.dim),
            ),
            Span::styled(bar, Style::default().fg(usage_color(core.usage))),
            Span::raw(format!(
                " {:>5.1}% {:>10}  ",
                core.usage,
                format_frequency_mhz(core.frequency_mhz)
            )),
            Span::styled(
                temp_str,
                Style::default().fg(core
                    .temperature
                    .map(temp_color)
                    .unwrap_or(state.theme.dim)),
            ),
        ]);
        lines.push(line);
    }
    let avg = snap.cpu.average_usage;
    let pkg_temp = snap
        .cpu
        .package_temp
        .map(|t| format_temp(t, state.fahrenheit))
        .unwrap_or_else(|| "--".into());
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(
            "Average ",
            Style::default()
                .fg(state.theme.header)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{:.1}%  ", avg),
            Style::default().fg(usage_color(avg)),
        ),
        Span::styled(format!("Pkg temp {}", pkg_temp), Style::default().fg(state.theme.text)),
    ]));
    f.render_widget(Paragraph::new(lines), chunks[1]);
}

pub fn render_chart(f: &mut Frame, area: Rect, hist: &RingBuffer<f32>, state: &AppState) {
    let data: Vec<(f64, f64)> = hist
        .iter_ordered()
        .enumerate()
        .map(|(i, v)| (i as f64, *v as f64))
        .collect();
    let max_x = (hist.capacity() as f64).max(1.0);
    let datasets = vec![Dataset::default()
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(state.theme.primary))
        .data(&data)];
    let chart = Chart::new(datasets)
        .x_axis(Axis::default().bounds([0.0, max_x]))
        .y_axis(
            Axis::default()
                .bounds([0.0, 100.0])
                .style(Style::default().fg(state.theme.dim)),
        )
        .block(
            ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::NONE)
                .title(Span::styled(
                    format!("Usage history ({}s)", state.graph_time_window.as_secs()),
                    Style::default().fg(state.theme.dim),
                )),
        );
    f.render_widget(chart, area);
}
