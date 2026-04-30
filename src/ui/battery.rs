use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::symbols;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph};
use ratatui::Frame;

use super::block;
use crate::collector::BatteryState;
use crate::state::AppState;
use crate::util::{bar, battery_color, format_duration, format_temp};

pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
    let snap = match &state.current_snapshot {
        Some(s) => s,
        None => return,
    };
    let outer = block("Battery", state);
    let inner = outer.inner(area);
    f.render_widget(outer, area);

    let bat = match &snap.battery {
        Some(b) => b,
        None => {
            f.render_widget(
                Paragraph::new("No battery detected (desktop or unsupported).")
                    .style(Style::default().fg(state.theme.dim)),
                inner,
            );
            return;
        }
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Min(3),
        ])
        .split(inner);

    render_charge_chart(f, chunks[0], state);
    render_rate_chart(f, chunks[1], state);
    render_stats(f, chunks[2], state, bat);
}

fn render_charge_chart(f: &mut Frame, area: Rect, state: &AppState) {
    let data: Vec<(f64, f64)> = state
        .battery_history
        .iter_ordered()
        .enumerate()
        .map(|(i, v)| (i as f64, *v as f64))
        .collect();
    let max_x = (state.battery_history.capacity() as f64).max(1.0);
    let datasets = vec![Dataset::default()
        .name("Charge%")
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
            Block::default()
                .borders(Borders::NONE)
                .title(Span::styled(
                    format!(
                        "Charge history ({}s)",
                        state.graph_time_window.as_secs()
                    ),
                    Style::default().fg(state.theme.dim),
                )),
        );
    f.render_widget(chart, area);
}

fn render_rate_chart(f: &mut Frame, area: Rect, state: &AppState) {
    let data: Vec<(f64, f64)> = state
        .battery_rate_history
        .iter_ordered()
        .enumerate()
        .map(|(i, v)| (i as f64, *v))
        .collect();
    let max_x = (state.battery_rate_history.capacity() as f64).max(1.0);
    let max_y = data.iter().map(|(_, v)| *v).fold(1.0_f64, f64::max);
    let datasets = vec![Dataset::default()
        .name("Watts")
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(state.theme.secondary))
        .data(&data)];
    let chart = Chart::new(datasets)
        .x_axis(Axis::default().bounds([0.0, max_x]))
        .y_axis(
            Axis::default()
                .bounds([0.0, max_y * 1.1])
                .style(Style::default().fg(state.theme.dim)),
        )
        .block(
            Block::default()
                .borders(Borders::NONE)
                .title(Span::styled(
                    format!(
                        "Power draw history ({}s)",
                        state.graph_time_window.as_secs()
                    ),
                    Style::default().fg(state.theme.dim),
                )),
        );
    f.render_widget(chart, area);
}

fn render_stats(
    f: &mut Frame,
    area: Rect,
    state: &AppState,
    bat: &crate::collector::BatterySnapshot,
) {
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
            .map(|d| format!("ETA full: {}", format_duration(d)))
            .unwrap_or_else(|| "ETA full: --".into()),
        BatteryState::Discharging => bat
            .time_to_empty
            .map(|d| format!("ETA empty: {}", format_duration(d)))
            .unwrap_or_else(|| "ETA empty: --".into()),
        BatteryState::Full => "Fully charged".into(),
        _ => "ETA: --".into(),
    };

    let temp = bat
        .temperature
        .map(|t| format_temp(t, state.fahrenheit))
        .unwrap_or_else(|| "--".into());

    let lines = vec![
        Line::from(vec![
            Span::styled(
                bar(bat.charge_percent, 100.0, 32),
                Style::default().fg(battery_color(bat.charge_percent)),
            ),
            Span::raw(format!(
                "  {:.0}% {} {}",
                bat.charge_percent,
                icon,
                bat.state.label()
            )),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Health   ", Style::default().fg(state.theme.dim)),
            Span::raw(format!("{:.0}%", bat.health_percent)),
            Span::styled("    Cycles   ", Style::default().fg(state.theme.dim)),
            Span::raw(
                bat.cycle_count
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "--".into()),
            ),
        ]),
        Line::from(vec![
            Span::styled("Power    ", Style::default().fg(state.theme.dim)),
            Span::raw(format!("{:.1} W", bat.energy_rate)),
            Span::styled("    Voltage  ", Style::default().fg(state.theme.dim)),
            Span::raw(format!("{:.2} V", bat.voltage)),
        ]),
        Line::from(vec![
            Span::styled("Temp     ", Style::default().fg(state.theme.dim)),
            Span::raw(temp),
            Span::styled("    AC       ", Style::default().fg(state.theme.dim)),
            Span::styled(
                if bat.ac_connected {
                    "connected".to_string()
                } else {
                    "unplugged".to_string()
                },
                Style::default().fg(if bat.ac_connected {
                    state.theme.primary
                } else {
                    state.theme.secondary
                }),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            eta,
            Style::default()
                .fg(state.theme.primary)
                .add_modifier(Modifier::BOLD),
        )),
    ];
    f.render_widget(Paragraph::new(lines), area);
}
