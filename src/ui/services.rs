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

pub fn render_services(f: &mut Frame, app: &App, area: Rect) {
    let ch = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(8)])
        .split(area);
    let total = app.services.len();
    let running = app.services.iter().filter(|s| s.pid > 0).count();
    let stopped = total - running;
    let error_count = app
        .services
        .iter()
        .filter(|s| s.last_exit != 0 && s.pid <= 0)
        .count();
    f.render_widget(
        Paragraph::new(vec![
            Line::from(vec![
                Span::styled(" Total: ", Style::default().fg(Color::Cyan).bold()),
                Span::raw(format!("{}", total)),
                Span::raw("  │  "),
                Span::styled("Running: ", Style::default().fg(Color::Green).bold()),
                Span::raw(format!("{}", running)),
                Span::raw("  │  "),
                Span::styled("Stopped: ", Style::default().fg(Color::DarkGray).bold()),
                Span::raw(format!("{}", stopped)),
                Span::raw("  │  "),
                Span::styled("Errors: ", Style::default().fg(Color::Red).bold()),
                Span::raw(format!("{}", error_count)),
            ]),
            Line::from(Span::styled(
                "  Source: launchctl list",
                Style::default().fg(Color::DarkGray),
            )),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" ⚙️ LaunchDaemons/Agents "),
        ),
        ch[0],
    );
    let hdr = Row::new(
        ["PID", "Exit", "Label"]
            .map(|h| Cell::from(h).style(Style::default().bold().fg(Color::Cyan))),
    )
    .height(1);
    let vh = ch[1].height.saturating_sub(3) as usize;
    let cursor = app.service_scroll.min(total.saturating_sub(1));
    let scroll_top = if cursor >= vh { cursor - vh + 1 } else { 0 };
    let rows: Vec<Row> = app
        .services
        .iter()
        .enumerate()
        .skip(scroll_top)
        .take(vh)
        .map(|(i, s)| {
            let pid_str = if s.pid > 0 {
                format!("{}", s.pid)
            } else {
                "-".into()
            };
            let pid_color = if s.pid > 0 {
                Color::Green
            } else {
                Color::DarkGray
            };
            let exit_color = if s.last_exit == 0 {
                Color::White
            } else {
                Color::Red
            };
            let row = Row::new(vec![
                Cell::from(pid_str).style(Style::default().fg(pid_color)),
                Cell::from(format!("{}", s.last_exit)).style(Style::default().fg(exit_color)),
                Cell::from(s.label.clone()),
            ]);
            if i == cursor {
                row.style(
                    Style::default()
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
                Constraint::Length(8),
                Constraint::Length(6),
                Constraint::Min(50),
            ],
        )
        .header(hdr)
        .block(Block::default().borders(Borders::ALL).title(format!(
            " [{}/{}] ↑↓ navigate ",
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
}
