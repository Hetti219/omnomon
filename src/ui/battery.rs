use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
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

    let icon = match bat.state {
        BatteryState::Charging => "⚡",
        BatteryState::Discharging => "▼",
        BatteryState::Full => "✓",
        BatteryState::Empty => "✗",
        BatteryState::NotCharging => "·",
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
            Span::raw(if bat.ac_connected {
                "connected".to_string()
            } else {
                "unplugged".to_string()
            }),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            eta,
            Style::default()
                .fg(state.theme.primary)
                .add_modifier(Modifier::BOLD),
        )),
    ];
    f.render_widget(Paragraph::new(lines), inner);
}
