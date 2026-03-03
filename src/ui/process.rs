use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table},
    Frame,
};

pub fn render_process(f: &mut Frame, app: &App, area: Rect) {
    let ch = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(8), Constraint::Length(7)])
        .split(area);
    let hdr = Row::new(
        [
            "PID", "User", "CPU%", "MEM%", "RSS", "VSZ", "State", "Thr", "Command",
        ]
        .map(|h| Cell::from(h).style(Style::default().bold().fg(Color::Cyan))),
    )
    .height(1);
    let vh = ch[0].height.saturating_sub(3) as usize;
    let total = app.processes.len();
    let cursor = app.process_scroll.min(total.saturating_sub(1));
    let scroll_top = if cursor >= vh { cursor - vh + 1 } else { 0 };
    let rows: Vec<Row> = app
        .processes
        .iter()
        .enumerate()
        .skip(scroll_top)
        .take(vh)
        .map(|(i, p)| {
            let cpu_color = if p.cpu > 50.0 {
                Color::Red
            } else if p.cpu > 10.0 {
                Color::Yellow
            } else {
                Color::Green
            };
            let mem_color = if p.mem > 10.0 {
                Color::Red
            } else if p.mem > 3.0 {
                Color::Yellow
            } else {
                Color::White
            };
            let state_color = match p.state.chars().next().unwrap_or(' ') {
                'R' => Color::Green,
                'S' => Color::DarkGray,
                'Z' => Color::Red,
                _ => Color::White,
            };
            let row = Row::new(vec![
                Cell::from(format!("{}", p.pid)),
                Cell::from(p.user.clone()).style(Style::default().fg(Color::Cyan)),
                Cell::from(format!("{:.1}", p.cpu)).style(Style::default().fg(cpu_color)),
                Cell::from(format!("{:.1}", p.mem)).style(Style::default().fg(mem_color)),
                Cell::from(p.rss.clone()),
                Cell::from(p.vsize.clone()),
                Cell::from(p.state.clone()).style(Style::default().fg(state_color)),
                Cell::from(format!("{}", p.threads)),
                Cell::from(p.command.clone()).style(Style::default().fg(Color::White).bold()),
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
                Constraint::Length(7),
                Constraint::Length(10),
                Constraint::Length(6),
                Constraint::Length(6),
                Constraint::Length(9),
                Constraint::Length(9),
                Constraint::Length(5),
                Constraint::Length(4),
                Constraint::Min(20),
            ],
        )
        .header(hdr)
        .block(Block::default().borders(Borders::ALL).title(format!(
            " 📊 Process Explorer [{}/{}] — sorted by CPU% ",
            cursor + 1,
            total
        ))),
        ch[0],
    );
    if total > 0 {
        let mut sb = ScrollbarState::new(total).position(cursor);
        f.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight),
            ch[0],
            &mut sb,
        );
    }
    if let Some(p) = app.processes.get(cursor) {
        let detail = vec![
            Line::from(vec![
                Span::styled(" Command: ", Style::default().fg(Color::Yellow).bold()),
                Span::styled(&p.command, Style::default().fg(Color::White).bold()),
                Span::raw(format!("  │  PID: {} / PPID: {}", p.pid, p.ppid)),
            ]),
            Line::from(vec![
                Span::styled(" CPU: ", Style::default().fg(Color::Green)),
                Span::raw(format!("{:.1}%", p.cpu)),
                Span::raw("  │  "),
                Span::styled("Memory: ", Style::default().fg(Color::Magenta)),
                Span::raw(format!("{:.1}%", p.mem)),
                Span::raw("  │  "),
                Span::styled("RSS: ", Style::default().fg(Color::Cyan)),
                Span::raw(&p.rss),
                Span::raw("  │  "),
                Span::styled("VSZ: ", Style::default().fg(Color::Blue)),
                Span::raw(&p.vsize),
            ]),
            Line::from(vec![
                Span::styled(" State: ", Style::default().fg(Color::Yellow)),
                Span::raw(&p.state),
                Span::raw("  │  "),
                Span::styled("Threads: ", Style::default().fg(Color::Green)),
                Span::raw(format!("{}", p.threads)),
                Span::raw("  │  "),
                Span::styled("User: ", Style::default().fg(Color::Cyan)),
                Span::raw(&p.user),
                Span::raw("  │  "),
                Span::styled("Started: ", Style::default().fg(Color::DarkGray)),
                Span::raw(&p.started),
            ]),
        ];
        f.render_widget(
            ratatui::widgets::Paragraph::new(detail).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Process Detail "),
            ),
            ch[1],
        );
    }
}
