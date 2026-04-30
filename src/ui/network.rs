use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::symbols;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph, Row, Table};
use ratatui::Frame;

use super::block;
use crate::state::AppState;
use crate::util::{format_bytes, format_rate};

pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
    let snap = match &state.current_snapshot {
        Some(s) => s,
        None => return,
    };

    let title = if snap.network.is_empty() {
        "Network".to_string()
    } else {
        let idx = state.selected_network_interface.min(snap.network.len() - 1);
        format!("Network · {} · [n] cycle interface", snap.network[idx].interface)
    };
    let outer = block(&title, state);
    let inner = outer.inner(area);
    f.render_widget(outer, area);

    if snap.network.is_empty() {
        f.render_widget(
            Paragraph::new("No network interfaces detected.")
                .style(Style::default().fg(state.theme.dim)),
            inner,
        );
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Length(4),
            Constraint::Min(3),
        ])
        .split(inner);

    render_chart(f, chunks[0], state);

    let idx = state.selected_network_interface.min(snap.network.len() - 1);
    let net = &snap.network[idx];
    let lines = vec![
        Line::from(vec![
            Span::styled("RX  ", Style::default().fg(state.theme.dim)),
            Span::styled(
                format_rate(net.rx_bytes_per_sec),
                Style::default()
                    .fg(state.theme.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("   total {}", format_bytes(net.rx_total))),
        ]),
        Line::from(vec![
            Span::styled("TX  ", Style::default().fg(state.theme.dim)),
            Span::styled(
                format_rate(net.tx_bytes_per_sec),
                Style::default()
                    .fg(state.theme.secondary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("   total {}", format_bytes(net.tx_total))),
        ]),
        Line::from(vec![
            Span::styled("Addr", Style::default().fg(state.theme.dim)),
            Span::raw(format!(
                "  IPv4 {}   IPv6 {}",
                net.ipv4.clone().unwrap_or_else(|| "--".into()),
                net.ipv6.clone().unwrap_or_else(|| "--".into())
            )),
        ]),
    ];
    f.render_widget(Paragraph::new(lines), chunks[1]);

    let header = Row::new(vec!["INTERFACE", "RX/s", "TX/s", "RX TOTAL", "TX TOTAL"])
        .style(
            Style::default()
                .fg(state.theme.header)
                .add_modifier(Modifier::BOLD),
        );
    let rows: Vec<Row> = snap
        .network
        .iter()
        .enumerate()
        .map(|(i, n)| {
            let style = if i == idx {
                Style::default()
                    .fg(state.theme.selected_fg)
                    .bg(state.theme.selected_bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(state.theme.text)
            };
            Row::new(vec![
                n.interface.clone(),
                format_rate(n.rx_bytes_per_sec),
                format_rate(n.tx_bytes_per_sec),
                format_bytes(n.rx_total),
                format_bytes(n.tx_total),
            ])
            .style(style)
        })
        .collect();
    let table = Table::new(
        rows,
        [
            Constraint::Length(16),
            Constraint::Length(14),
            Constraint::Length(14),
            Constraint::Length(14),
            Constraint::Min(10),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(state.theme.dim))
            .title(Span::styled(
                "Interfaces",
                Style::default().fg(state.theme.dim),
            )),
    );
    f.render_widget(table, chunks[2]);
}

fn render_chart(f: &mut Frame, area: Rect, state: &AppState) {
    let rx: Vec<(f64, f64)> = state
        .net_rx_history
        .iter_ordered()
        .enumerate()
        .map(|(i, v)| (i as f64, *v))
        .collect();
    let tx: Vec<(f64, f64)> = state
        .net_tx_history
        .iter_ordered()
        .enumerate()
        .map(|(i, v)| (i as f64, *v))
        .collect();
    let max_y = rx
        .iter()
        .chain(tx.iter())
        .map(|(_, v)| *v)
        .fold(1.0_f64, f64::max);
    let max_x = (state.net_rx_history.capacity() as f64).max(1.0);
    let datasets = vec![
        Dataset::default()
            .name("RX")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(state.theme.primary))
            .data(&rx),
        Dataset::default()
            .name("TX")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(state.theme.secondary))
            .data(&tx),
    ];
    let chart = Chart::new(datasets)
        .x_axis(Axis::default().bounds([0.0, max_x]))
        .y_axis(Axis::default().bounds([0.0, max_y * 1.1]))
        .block(
            Block::default()
                .borders(Borders::NONE)
                .title(Span::styled(
                    format!(
                        "Throughput ({}s)",
                        state.graph_time_window.as_secs()
                    ),
                    Style::default().fg(state.theme.dim),
                )),
        );
    f.render_widget(chart, area);
}
