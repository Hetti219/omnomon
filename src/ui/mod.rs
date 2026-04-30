pub mod battery;
pub mod cpu;
pub mod dashboard;
pub mod disk;
pub mod gpu;
pub mod header;
pub mod help;
pub mod memory;
pub mod network;
pub mod process;
pub mod theme;
pub mod thermal;

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::state::{AppState, Tab};

pub fn render(f: &mut Frame, state: &AppState) {
    let area = f.area();
    if area.width < 80 {
        let warn = Paragraph::new("Terminal too narrow. Minimum 80 columns required.")
            .style(Style::default().fg(state.theme.secondary));
        f.render_widget(warn, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(area);

    header::render(f, chunks[0], state);
    render_tabs(f, chunks[1], state);

    match state.current_tab {
        Tab::Dashboard => dashboard::render(f, chunks[2], state),
        Tab::Cpu => cpu::render(f, chunks[2], state),
        Tab::Gpu => gpu::render(f, chunks[2], state),
        Tab::Memory => memory::render(f, chunks[2], state),
        Tab::Disk => disk::render(f, chunks[2], state),
        Tab::Network => network::render(f, chunks[2], state),
        Tab::Processes => process::render(f, chunks[2], state),
        Tab::Battery => battery::render(f, chunks[2], state),
        Tab::Thermal => thermal::render(f, chunks[2], state),
    }

    render_status_bar(f, chunks[3], state);

    if state.show_help {
        help::render(f, area, state);
    }
}

fn render_tabs(f: &mut Frame, area: Rect, state: &AppState) {
    let mut spans = Vec::new();
    for (i, tab) in Tab::ALL.iter().enumerate() {
        let label = format!(" [{}] {} ", i + 1, tab.label());
        let style = if *tab == state.current_tab {
            Style::default()
                .fg(state.theme.selected_fg)
                .bg(state.theme.selected_bg)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(state.theme.dim)
        };
        spans.push(Span::styled(label, style));
    }
    let line = Line::from(spans);
    f.render_widget(Paragraph::new(line), area);
}

fn render_status_bar(f: &mut Frame, area: Rect, state: &AppState) {
    let rate = state.refresh_rate.as_millis();
    let win = state.graph_time_window.as_secs();
    let text = format!(
        " rate {}ms · graph {}s · theme {} · [?] help · [q] quit",
        rate, win, state.theme.name
    );
    let p = Paragraph::new(text).style(Style::default().fg(state.theme.dim));
    f.render_widget(p, area);
}

pub fn block<'a>(title: &'a str, state: &AppState) -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(state.theme.border))
        .title(Span::styled(
            format!(" {} ", title),
            Style::default()
                .fg(state.theme.header)
                .add_modifier(Modifier::BOLD),
        ))
}
