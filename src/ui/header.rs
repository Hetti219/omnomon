use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::state::AppState;
use crate::util::format_uptime;

pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
    let info = state
        .current_snapshot
        .as_ref()
        .map(|s| s.system_info.clone())
        .unwrap_or_default();
    let load = info.load_avg;
    let line = Line::from(vec![
        Span::styled(
            "OmniMon ",
            Style::default()
                .fg(state.theme.header)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("· {} ", info.hostname), Style::default().fg(state.theme.text)),
        Span::styled(format!("· {} ", info.os_name), Style::default().fg(state.theme.dim)),
        Span::styled(
            format!("· kernel {} ", info.kernel_version),
            Style::default().fg(state.theme.dim),
        ),
        Span::styled(
            format!("· up {} ", format_uptime(info.uptime)),
            Style::default().fg(state.theme.text),
        ),
        Span::styled(
            format!("· load {:.2} {:.2} {:.2}", load.0, load.1, load.2),
            Style::default().fg(state.theme.primary),
        ),
    ]);
    f.render_widget(Paragraph::new(line), area);
}
