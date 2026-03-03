use crate::app::App;
use crate::types::DiskMode;
use crate::utils::hs;
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

pub fn render_disk(f: &mut Frame, app: &App, area: Rect) {
    match app.disk_mode {
        DiskMode::Partitions => render_disk_partitions(f, app, area),
        DiskMode::Files => render_disk_files(f, app, area),
    }
}

fn render_disk_partitions(f: &mut Frame, app: &App, area: Rect) {
    let ch = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Min(6),
            Constraint::Length(5),
        ])
        .split(area);
    let hw = &app.disk_hw;
    let hw_lines = vec![
        Line::from(Span::styled(
            "━━━ Primary Disk ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
            Style::default().fg(Color::Cyan).bold(),
        )),
        Line::from(vec![
            Span::styled("  Device: ", Style::default().fg(Color::Yellow)),
            Span::raw(&hw.media_name),
            Span::raw("  │  "),
            Span::styled("Protocol: ", Style::default().fg(Color::Green)),
            Span::raw(&hw.protocol),
            Span::raw("  │  "),
            Span::styled("Size: ", Style::default().fg(Color::Magenta)),
            Span::raw(&hw.disk_size),
        ]),
        Line::from(vec![
            Span::styled("  Block: ", Style::default().fg(Color::Cyan)),
            Span::raw(&hw.block_size),
            Span::raw("  │  "),
            Span::styled("Scheme: ", Style::default().fg(Color::Blue)),
            Span::raw(&hw.content),
            Span::raw("  │  "),
            Span::styled(
                "SMART: ",
                Style::default().fg(if hw.smart_status.contains("Verified") {
                    Color::Green
                } else {
                    Color::Red
                }),
            ),
            Span::raw(&hw.smart_status),
        ]),
        Line::from(vec![
            Span::styled("  I/O: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!(
                "{:.1} KB/t, {:.0} tps, {:.2} MB/s",
                app.iostat.kb_per_transfer, app.iostat.transfers_per_sec, app.iostat.mb_per_sec
            )),
            Span::raw("  │  "),
            Span::styled("Lifetime: ", Style::default().fg(Color::Magenta)),
            Span::raw(format!(
                "{} R ({}), {} W ({})",
                app.top_stats.disk_reads,
                app.top_stats.disk_read_bytes,
                app.top_stats.disk_writes,
                app.top_stats.disk_write_bytes
            )),
        ]),
    ];
    f.render_widget(
        Paragraph::new(hw_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" 💾 Disk Hardware & I/O "),
        ),
        ch[0],
    );
    let hdr = Row::new(
        ["Mount", "Used", "Total", "Type", "Usage"]
            .map(|h| Cell::from(h).style(Style::default().bold().fg(Color::Cyan))),
    )
    .height(1);
    let vh = ch[1].height.saturating_sub(3) as usize;
    let cursor = app.disk_cursor.min(app.disk_list.len().saturating_sub(1));
    let scroll_top = if cursor >= vh { cursor - vh + 1 } else { 0 };
    let rows: Vec<Row> = app
        .disk_list
        .iter()
        .enumerate()
        .skip(scroll_top)
        .take(vh)
        .map(|(i, d)| {
            let p = if d.total > 0 {
                d.used as f64 / d.total as f64 * 100.0
            } else {
                0.0
            };
            let c = if p < 70.0 {
                Color::Green
            } else if p < 90.0 {
                Color::Yellow
            } else {
                Color::Red
            };
            let bw = 15usize;
            let fl = (p as usize * bw) / 100;
            let bar = format!("{}{}", "█".repeat(fl), "░".repeat(bw.saturating_sub(fl)));
            let row = Row::new(vec![
                Cell::from(format!(
                    "{}{}",
                    d.mount_point,
                    if d.is_removable { " ⏏" } else { "" }
                )),
                Cell::from(hs(d.used)),
                Cell::from(hs(d.total)),
                Cell::from(d.fs_type.clone()),
                Cell::from(format!("{} {:.1}%", bar, p)).style(Style::default().fg(c)),
            ]);
            if i == cursor {
                row.style(
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                row.style(Style::default().fg(Color::White))
            }
        })
        .collect();
    f.render_widget(
        Table::new(
            rows,
            [
                Constraint::Min(15),
                Constraint::Length(10),
                Constraint::Length(10),
                Constraint::Length(6),
                Constraint::Min(22),
            ],
        )
        .header(hdr)
        .block(Block::default().borders(Borders::ALL).title(format!(
            " Partitions [{}/{}] ↑↓ navigate, Enter → browse ",
            cursor + 1,
            app.disk_list.len()
        ))),
        ch[1],
    );
    let mut sb = ScrollbarState::new(app.disk_list.len()).position(cursor);
    f.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight),
        ch[1],
        &mut sb,
    );
    f.render_widget(
        Paragraph::new(vec![Line::from(vec![
            Span::styled(" ↑↓ ", Style::default().fg(Color::Black).bg(Color::Cyan)),
            Span::raw(" Navigate  "),
            Span::styled(
                " Enter ",
                Style::default().fg(Color::Black).bg(Color::Green),
            ),
            Span::raw(" Browse Files  "),
        ])])
        .block(Block::default().borders(Borders::ALL).title(" Keys ")),
        ch[2],
    );
}

fn render_disk_files(f: &mut Frame, app: &App, area: Rect) {
    let ch = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(6),
            Constraint::Length(5),
        ])
        .split(area);
    let sort_label = match app.disk_sort {
        crate::types::SortMode::SizeDsc => "Size ↓",
        crate::types::SortMode::SizeAsc => "Size ↑",
        crate::types::SortMode::NameAsc => "Name A→Z",
        crate::types::SortMode::NameDsc => "Name Z→A",
    };
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" 📂 ", Style::default().fg(Color::Yellow)),
            Span::styled(&app.disk_path, Style::default().fg(Color::White).bold()),
            Span::raw("  │  "),
            Span::styled("Sort: ", Style::default().fg(Color::Cyan)),
            Span::raw(sort_label),
            Span::raw("  │  "),
            Span::styled("Filter: ", Style::default().fg(Color::Green)),
            Span::raw(if app.disk_filter_system {
                "Hide System ✓"
            } else {
                "Show All"
            }),
            Span::raw(format!("  │  {} items", app.disk_files.len())),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" File Browser "),
        ),
        ch[0],
    );
    let hdr = Row::new(
        ["Name", "Size", "Type"]
            .map(|h| Cell::from(h).style(Style::default().bold().fg(Color::Cyan))),
    )
    .height(1);
    let vh = ch[1].height.saturating_sub(3) as usize;
    let cursor = app
        .disk_file_cursor
        .min(app.disk_files.len().saturating_sub(1));
    let scroll_top = if cursor >= vh { cursor - vh + 1 } else { 0 };
    let rows: Vec<Row> = app
        .disk_files
        .iter()
        .enumerate()
        .skip(scroll_top)
        .take(vh)
        .map(|(i, fe)| {
            let icon = if fe.is_dir { "📁 " } else { "📄 " };
            let sc = if fe.is_system {
                Color::DarkGray
            } else if fe.is_dir {
                Color::Cyan
            } else {
                Color::White
            };
            let type_label = if fe.is_dir {
                "DIR"
            } else {
                fe.name.rsplit('.').next().unwrap_or("???")
            };
            let sys_badge = if fe.is_system {
                Cell::from(format!("{} [SYS]", type_label))
                    .style(Style::default().fg(Color::DarkGray))
            } else {
                Cell::from(type_label.to_string()).style(Style::default().fg(Color::Green))
            };
            let row = Row::new(vec![
                Cell::from(format!("{}{}", icon, fe.name)).style(Style::default().fg(sc)),
                Cell::from(hs(fe.size)),
                sys_badge,
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
    let title = if app.disk_files.is_empty() {
        " No files (or access denied) ".to_string()
    } else {
        format!(
            " [{}/{}] ↑↓ navigate, Enter → open, Esc → back ",
            cursor + 1,
            app.disk_files.len()
        )
    };
    f.render_widget(
        Table::new(
            rows,
            [
                Constraint::Min(40),
                Constraint::Length(12),
                Constraint::Length(12),
            ],
        )
        .header(hdr)
        .block(Block::default().borders(Borders::ALL).title(title)),
        ch[1],
    );
    if !app.disk_files.is_empty() {
        let mut sb = ScrollbarState::new(app.disk_files.len()).position(cursor);
        f.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight),
            ch[1],
            &mut sb,
        );
    }
    f.render_widget(
        Paragraph::new(vec![Line::from(vec![
            Span::styled(" ↑↓ ", Style::default().fg(Color::Black).bg(Color::Cyan)),
            Span::raw(" Navigate  "),
            Span::styled(
                " Enter ",
                Style::default().fg(Color::Black).bg(Color::Green),
            ),
            Span::raw(" Open Dir  "),
            Span::styled(
                " Esc/← ",
                Style::default().fg(Color::Black).bg(Color::Yellow),
            ),
            Span::raw(" Back  "),
            Span::styled(" s ", Style::default().fg(Color::Black).bg(Color::Magenta)),
            Span::raw(" Sort  "),
            Span::styled(" f ", Style::default().fg(Color::Black).bg(Color::Red)),
            Span::raw(" Filter  "),
        ])])
        .block(Block::default().borders(Borders::ALL).title(" Keys ")),
        ch[2],
    );
}
