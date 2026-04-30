use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use super::block;
use crate::state::AppState;
use crate::util::{bar, format_temp, temp_color};

pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
    let snap = match &state.current_snapshot {
        Some(s) => s,
        None => return,
    };
    let outer = block("Thermal", state);
    let inner = outer.inner(area);
    f.render_widget(outer, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(inner);

    let mut zone_lines: Vec<Line> = Vec::new();
    if snap.thermal.zones.is_empty() {
        zone_lines.push(Line::from(Span::styled(
            "No thermal zones available.",
            Style::default().fg(state.theme.dim),
        )));
    } else {
        for z in &snap.thermal.zones {
            let max = z.critical.unwrap_or(100.0).max(50.0);
            zone_lines.push(Line::from(vec![
                Span::styled(
                    format!("{:<14} ", truncate(&z.name, 14)),
                    Style::default().fg(state.theme.text),
                ),
                Span::styled(bar(z.temp, max, 24), Style::default().fg(temp_color(z.temp))),
                Span::raw(format!(
                    "  {}",
                    format_temp(z.temp, state.fahrenheit)
                )),
                Span::styled(
                    z.critical
                        .map(|c| format!("  [{}]", format_temp(c, state.fahrenheit)))
                        .unwrap_or_default(),
                    Style::default().fg(state.theme.dim),
                ),
            ]));
        }
    }
    f.render_widget(
        Paragraph::new(zone_lines).block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(state.theme.dim))
                .title(Span::styled(
                    "Zones",
                    Style::default().fg(state.theme.dim),
                )),
        ),
        chunks[0],
    );

    let mut fan_lines: Vec<Line> = Vec::new();
    if snap.thermal.fans.is_empty() {
        fan_lines.push(Line::from(Span::styled(
            "No fans detected.",
            Style::default().fg(state.theme.dim),
        )));
    } else {
        for fan in &snap.thermal.fans {
            let max = fan.max_rpm.unwrap_or(5000).max(1) as f32;
            fan_lines.push(Line::from(vec![
                Span::styled(
                    format!("{:<18} ", truncate(&fan.name, 18)),
                    Style::default().fg(state.theme.text),
                ),
                Span::styled(
                    bar(fan.rpm as f32, max, 18),
                    Style::default().fg(state.theme.primary),
                ),
                Span::raw(format!("  {} RPM", fan.rpm)),
                Span::styled(
                    fan.max_rpm
                        .map(|m| format!("   max {}", m))
                        .unwrap_or_default(),
                    Style::default().fg(state.theme.dim),
                ),
            ]));
        }
    }
    f.render_widget(
        Paragraph::new(fan_lines).block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(state.theme.dim))
                .title(Span::styled(
                    "Fans",
                    Style::default().fg(state.theme.dim),
                )),
        ),
        chunks[1],
    );
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let t: String = s.chars().take(max - 1).collect();
        format!("{}…", t)
    }
}
