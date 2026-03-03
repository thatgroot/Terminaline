use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, Borders, Cell, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState,
        Table,
    },
    Frame,
};
use std::collections::HashMap;

pub fn render_activity(f: &mut Frame, app: &App, area: Rect) {
    let ch = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Min(6),
            Constraint::Length(7),
        ])
        .split(area);
    let total = app.activity_connections.len();
    let tcp_count = app
        .activity_connections
        .iter()
        .filter(|c| c.proto == "TCP")
        .count();
    let udp_count = app
        .activity_connections
        .iter()
        .filter(|c| c.proto == "UDP")
        .count();
    let established = app
        .activity_connections
        .iter()
        .filter(|c| c.state == "ESTABLISHED")
        .count();
    let listening = app
        .activity_connections
        .iter()
        .filter(|c| c.state == "LISTEN")
        .count();
    let mut procs: HashMap<String, usize> = HashMap::new();
    for c in &app.activity_connections {
        *procs.entry(c.process.clone()).or_insert(0) += 1;
    }
    let mut proc_counts: Vec<(String, usize)> = procs.into_iter().collect();
    proc_counts.sort_by(|a, b| b.1.cmp(&a.1));
    let top3: String = proc_counts
        .iter()
        .take(3)
        .map(|(n, c)| format!("{} ({})", n, c))
        .collect::<Vec<_>>()
        .join(", ");
    f.render_widget(
        Paragraph::new(vec![
            Line::from(vec![
                Span::styled(" Connections: ", Style::default().fg(Color::Cyan).bold()),
                Span::raw(format!("{}", total)),
                Span::raw("  │  "),
                Span::styled("TCP: ", Style::default().fg(Color::Green)),
                Span::raw(format!("{}", tcp_count)),
                Span::raw("  │  "),
                Span::styled("UDP: ", Style::default().fg(Color::Yellow)),
                Span::raw(format!("{}", udp_count)),
                Span::raw("  │  "),
                Span::styled("EST: ", Style::default().fg(Color::Magenta)),
                Span::raw(format!("{}", established)),
                Span::raw("  │  "),
                Span::styled("LISTEN: ", Style::default().fg(Color::Blue)),
                Span::raw(format!("{}", listening)),
            ]),
            Line::from(vec![
                Span::styled(" Top: ", Style::default().fg(Color::Yellow).bold()),
                Span::raw(top3),
            ]),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" 🔍 System Activity — Live Network Connections "),
        ),
        ch[0],
    );
    let hdr = Row::new(
        ["PID", "Process", "Proto", "Local", "Remote", "State"]
            .map(|h| Cell::from(h).style(Style::default().bold().fg(Color::Cyan))),
    )
    .height(1);
    let vh = ch[1].height.saturating_sub(3) as usize;
    let cursor = app.activity_scroll.min(total.saturating_sub(1));
    let scroll_top = if cursor >= vh { cursor - vh + 1 } else { 0 };
    let rows: Vec<Row> = app
        .activity_connections
        .iter()
        .enumerate()
        .skip(scroll_top)
        .take(vh)
        .map(|(i, c)| {
            let state_color = match c.state.as_str() {
                "ESTABLISHED" => Color::Green,
                "LISTEN" => Color::Blue,
                "CLOSE_WAIT" | "TIME_WAIT" => Color::Yellow,
                "SYN_SENT" | "SYN_RECV" => Color::Magenta,
                _ => Color::White,
            };
            let row = Row::new(vec![
                Cell::from(format!("{}", c.pid)),
                Cell::from(c.process.clone()),
                Cell::from(c.proto.clone()).style(Style::default().fg(if c.proto == "TCP" {
                    Color::Green
                } else {
                    Color::Yellow
                })),
                Cell::from(c.local_addr.clone()),
                Cell::from(if c.remote_addr.is_empty() {
                    "—".to_string()
                } else {
                    c.remote_addr.clone()
                }),
                Cell::from(c.state.clone()).style(Style::default().fg(state_color)),
            ]);
            if i == cursor {
                row.style(
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                row
            }
        })
        .collect();
    f.render_widget(
        Table::new(
            rows,
            [
                Constraint::Length(7),
                Constraint::Length(16),
                Constraint::Length(5),
                Constraint::Min(22),
                Constraint::Min(26),
                Constraint::Length(14),
            ],
        )
        .header(hdr)
        .block(Block::default().borders(Borders::ALL).title(format!(
            " [{}/{}] ↑↓ select ",
            cursor + 1,
            total
        ))),
        ch[1],
    );
    if total > 0 {
        let mut sb = ScrollbarState::new(total).position(cursor);
        f.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight),
            ch[1],
            &mut sb,
        );
    }
    let detail = if let Some(c) = app.activity_connections.get(cursor) {
        vec![
            Line::from(vec![
                Span::styled(" Process: ", Style::default().fg(Color::Yellow).bold()),
                Span::raw(format!("{} (PID {})", c.process, c.pid)),
                Span::raw("  │  "),
                Span::styled("FD: ", Style::default().fg(Color::Cyan)),
                Span::raw(&c.fd),
                Span::raw("  │  "),
                Span::styled("Protocol: ", Style::default().fg(Color::Green)),
                Span::raw(&c.proto),
            ]),
            Line::from(vec![
                Span::styled(" Local:  ", Style::default().fg(Color::Magenta).bold()),
                Span::raw(&c.local_addr),
            ]),
            Line::from(vec![
                Span::styled(" Remote: ", Style::default().fg(Color::Red).bold()),
                Span::raw(if c.remote_addr.is_empty() {
                    "— (listening/bound)"
                } else {
                    &c.remote_addr
                }),
                Span::raw("  │  "),
                Span::styled("State: ", Style::default().fg(Color::Blue)),
                Span::raw(&c.state),
            ]),
        ]
    } else {
        vec![Line::from("No connections")]
    };
    f.render_widget(
        Paragraph::new(detail).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Connection Details "),
        ),
        ch[2],
    );
}
