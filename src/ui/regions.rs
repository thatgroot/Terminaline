use crate::app::App;
use crate::types::RegionType;
use crate::utils::*;
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

pub fn render_regions(f: &mut Frame, app: &App, area: Rect) {
    let ch = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Min(8),
            Constraint::Length(7),
        ])
        .split(area);
    let tv: usize = app.regions.iter().map(|r| r.size).sum();
    let mut tc: HashMap<RegionType, (usize, usize)> = HashMap::new();
    for r in &app.regions {
        let e = tc.entry(r.region_type).or_insert((0, 0));
        e.0 += 1;
        e.1 += r.size;
    }
    let sp: Vec<Span> = [
        RegionType::Stack,
        RegionType::Heap,
        RegionType::Code,
        RegionType::Dylib,
        RegionType::Anonymous,
        RegionType::MappedFile,
    ]
    .iter()
    .flat_map(|t| {
        let (c, s) = tc.get(t).unwrap_or(&(0, 0));
        vec![
            Span::styled(
                format!(" {} ", t.label()),
                Style::default().fg(Color::Black).bg(t.color()),
            ),
            Span::raw(format!(":{} ({})  ", c, hs(*s as u64))),
        ]
    })
    .collect();
    f.render_widget(
        Paragraph::new(vec![
            Line::from(vec![
                Span::styled("Regions: ", Style::default().fg(Color::Cyan).bold()),
                Span::raw(format!("{}  ", app.regions.len())),
                Span::styled("Virtual: ", Style::default().fg(Color::Yellow).bold()),
                Span::raw(hsu(tv)),
                Span::raw("  │  "),
                Span::styled("PID: ", Style::default().fg(Color::Magenta).bold()),
                Span::raw(format!("{}", app.pid)),
            ]),
            Line::from(sp),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Process Memory Map "),
        ),
        ch[0],
    );
    let hdr = Row::new(
        ["Start", "End", "Size", "Perm", "Type", "Name"]
            .map(|h| Cell::from(h).style(Style::default().bold().fg(Color::Cyan))),
    )
    .height(1)
    .bottom_margin(0);
    let vh = ch[1].height.saturating_sub(3) as usize;
    let cursor = app.region_scroll.min(app.regions.len().saturating_sub(1));
    let scroll_top = if cursor >= vh { cursor - vh + 1 } else { 0 };
    let rows: Vec<Row> = app
        .regions
        .iter()
        .enumerate()
        .skip(scroll_top)
        .take(vh)
        .map(|(i, r)| {
            let c = r.region_type.color();
            let row = Row::new(vec![
                Cell::from(format!("0x{:012x}", r.start)),
                Cell::from(format!("0x{:012x}", r.end)),
                Cell::from(hsu(r.size)),
                Cell::from(r.perms.clone()),
                Cell::from(r.region_type.label()).style(Style::default().fg(Color::Black).bg(c)),
                Cell::from(trunc(&r.name, 40)),
            ]);
            if i == cursor {
                row.style(
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                row.style(Style::default().fg(c))
            }
        })
        .collect();
    f.render_widget(
        Table::new(
            rows,
            [
                Constraint::Min(16),
                Constraint::Min(16),
                Constraint::Length(10),
                Constraint::Length(5),
                Constraint::Length(6),
                Constraint::Min(30),
            ],
        )
        .header(hdr)
        .block(Block::default().borders(Borders::ALL).title(format!(
            " Regions [{}/{}] ↑↓ navigate ",
            cursor + 1,
            app.regions.len()
        ))),
        ch[1],
    );
    let mut sb = ScrollbarState::new(app.regions.len()).position(cursor);
    f.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight),
        ch[1],
        &mut sb,
    );
    if let Some(r) = app.regions.get(cursor) {
        let size_pct = if tv > 0 {
            r.size as f64 / tv as f64 * 100.0
        } else {
            0.0
        };
        let detail = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("▸ ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!("{} ", r.region_type.label()),
                    Style::default()
                        .fg(Color::Black)
                        .bg(r.region_type.color())
                        .bold(),
                ),
                Span::raw("  "),
                Span::styled(&r.name, Style::default().fg(Color::White).bold()),
            ]),
            Line::from(vec![
                Span::styled("  Address: ", Style::default().fg(Color::Yellow)),
                Span::raw(format!("0x{:016x} → 0x{:016x}", r.start, r.end)),
                Span::raw("  │  "),
                Span::styled("Size: ", Style::default().fg(Color::Green)),
                Span::raw(format!("{} ({:.2}% of virtual)", hsu(r.size), size_pct)),
            ]),
            Line::from(vec![
                Span::styled("  Permissions: ", Style::default().fg(Color::Magenta)),
                Span::styled(
                    if r.perms.contains('R') { "R" } else { "-" },
                    Style::default().fg(if r.perms.contains('R') {
                        Color::Green
                    } else {
                        Color::DarkGray
                    }),
                ),
                Span::styled(
                    if r.perms.contains('W') { "W" } else { "-" },
                    Style::default().fg(if r.perms.contains('W') {
                        Color::Yellow
                    } else {
                        Color::DarkGray
                    }),
                ),
                Span::styled(
                    if r.perms.contains('X') { "X" } else { "-" },
                    Style::default().fg(if r.perms.contains('X') {
                        Color::Red
                    } else {
                        Color::DarkGray
                    }),
                ),
                Span::raw(format!(
                    "  │  Pages: ~{}",
                    r.size / app.vm_stat.page_size.max(1) as usize
                )),
            ]),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" ▸ Selected Region Detail "),
        );
        f.render_widget(detail, ch[2]);
    }
}
