use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::state::AppState;

pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
    let popup = centered_rect(70, 80, area);
    f.render_widget(Clear, popup);

    let mut lines: Vec<Line> = Vec::new();
    lines.push(section("Global", state));
    lines.push(kv("q / Ctrl+C", "Quit"));
    lines.push(kv("?", "Toggle this help"));
    lines.push(kv("Esc", "Close help / clear filter"));
    lines.push(kv("1 – 9", "Switch tab"));
    lines.push(kv("Tab / Shift+Tab", "Next / previous tab"));
    lines.push(kv("r", "Force refresh"));
    lines.push(kv("+ / =", "Zoom in graph (shorter window)"));
    lines.push(kv("-", "Zoom out graph (longer window)"));
    lines.push(Line::from(""));
    lines.push(section("Processes", state));
    lines.push(kv("↑ ↓ / k j", "Move selection"));
    lines.push(kv("Home / g", "Jump to top"));
    lines.push(kv("End / G", "Jump to bottom"));
    lines.push(kv("/", "Edit filter (Enter to apply, Esc to clear)"));
    lines.push(kv("s", "Cycle sort column"));
    lines.push(kv("S", "Toggle sort direction"));
    lines.push(kv("K", "Kill selected (SIGTERM)"));
    lines.push(kv("D", "Kill selected (SIGKILL)"));
    lines.push(kv("t", "Toggle tree view"));
    lines.push(Line::from(""));
    lines.push(section("Network", state));
    lines.push(kv("n", "Cycle interface"));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  Press ? or Esc to close.",
        Style::default().fg(state.theme.dim),
    )));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(state.theme.border))
        .title(Span::styled(
            " Help ",
            Style::default()
                .fg(state.theme.header)
                .add_modifier(Modifier::BOLD),
        ));
    let p = Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Left);
    f.render_widget(p, popup);
}

fn section(title: &str, state: &AppState) -> Line<'static> {
    Line::from(Span::styled(
        format!(" {}", title),
        Style::default()
            .fg(state.theme.primary)
            .add_modifier(Modifier::BOLD),
    ))
}

fn kv(key: &str, desc: &str) -> Line<'static> {
    Line::from(vec![
        Span::raw("   "),
        Span::styled(
            format!("{:<18}", key),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw(desc.to_string()),
    ])
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(v[1])[1]
}
