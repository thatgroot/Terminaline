use crate::app::App;
use crate::types::RegionType;
use crate::utils::*;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::collections::HashMap;

pub fn render_visual(f: &mut Frame, app: &App, area: Rect) {
    let ch = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(12),
        ])
        .split(area);
    let ls: Vec<Span> = [
        RegionType::Stack,
        RegionType::Heap,
        RegionType::Code,
        RegionType::Dylib,
        RegionType::Anonymous,
        RegionType::MappedFile,
    ]
    .iter()
    .flat_map(|t| {
        vec![
            Span::styled(format!(" █ {} ", t.label()), Style::default().fg(t.color())),
            Span::raw(" "),
        ]
    })
    .collect();
    f.render_widget(
        Paragraph::new(Line::from(ls))
            .block(Block::default().borders(Borders::ALL).title(" Legend "))
            .alignment(Alignment::Center),
        ch[0],
    );
    if app.regions.is_empty() {
        f.render_widget(
            Paragraph::new("No regions.")
                .block(Block::default().borders(Borders::ALL))
                .alignment(Alignment::Center),
            ch[1],
        );
    } else {
        let iw = ch[1].width.saturating_sub(2) as usize;
        let ih = ch[1].height.saturating_sub(2) as usize;
        if iw > 0 && ih > 0 {
            let mn = app.regions.iter().map(|r| r.start).min().unwrap_or(0);
            let mx = app.regions.iter().map(|r| r.end).max().unwrap_or(1);
            let ar = if mx > mn { mx - mn } else { 1 };
            let total_cells = iw * ih;
            let bpc = (ar / total_cells.max(1)).max(1);
            let mut cells = vec![('·', Color::DarkGray); total_cells];
            let cursor = app.region_scroll.min(app.regions.len().saturating_sub(1));
            for (idx, r) in app.regions.iter().enumerate() {
                let so = r.start.saturating_sub(mn);
                let eo = r.end.saturating_sub(mn).min(ar);
                let (cs, ce) = (so / bpc, (eo / bpc).min(total_cells));
                let glyph = match r.region_type {
                    RegionType::Stack => '█',
                    RegionType::Heap => '▓',
                    RegionType::Code => '▒',
                    RegionType::Dylib => '░',
                    RegionType::Anonymous => '·',
                    RegionType::MappedFile => '▪',
                };
                let color = if idx == cursor {
                    Color::White
                } else {
                    r.region_type.color()
                };
                for i in cs..ce {
                    if i < total_cells {
                        cells[i] = (glyph, color);
                    }
                }
            }
            let lines: Vec<Line> = (0..ih)
                .map(|row| {
                    let s = row * iw;
                    let e = (s + iw).min(total_cells);
                    Line::from(
                        cells[s..e]
                            .iter()
                            .map(|(c, col)| Span::styled(c.to_string(), Style::default().fg(*col)))
                            .collect::<Vec<_>>(),
                    )
                })
                .collect();
            f.render_widget(
                Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(format!(
                    " Address Space (0x{:x}..0x{:x}, {}/cell) ",
                    mn,
                    mx,
                    hsu(bpc)
                ))),
                ch[1],
            );
        }
    }
    let tv: usize = app.regions.iter().map(|r| r.size).sum();
    let mut tc: HashMap<RegionType, (usize, usize)> = HashMap::new();
    for r in &app.regions {
        let e = tc.entry(r.region_type).or_insert((0, 0));
        e.0 += 1;
        e.1 += r.size;
    }
    let mut summary: Vec<Line> = Vec::new();
    summary.push(Line::from(vec![
        Span::styled("Total Virtual: ", Style::default().fg(Color::Cyan).bold()),
        Span::raw(hsu(tv)),
        Span::raw(format!("  ({} regions)", app.regions.len())),
        Span::raw("  │  "),
        Span::styled("PhysMem: ", Style::default().fg(Color::Yellow)),
        Span::raw(format!(
            "{} used, {} free",
            app.top_stats.phys_used, app.top_stats.phys_unused
        )),
    ]));
    summary.push(Line::from(""));
    let types = [
        RegionType::Stack,
        RegionType::Heap,
        RegionType::Code,
        RegionType::Dylib,
        RegionType::MappedFile,
        RegionType::Anonymous,
    ];
    for t in &types {
        let (count, size) = tc.get(t).copied().unwrap_or((0, 0));
        let pct = if tv > 0 {
            size as f64 / tv as f64 * 100.0
        } else {
            0.0
        };
        let bw = 20usize;
        let filled = (pct as usize * bw / 100).min(bw);
        summary.push(Line::from(vec![
            Span::styled(
                format!("  {:>8} ", t.label()),
                Style::default().fg(t.color()).bold(),
            ),
            Span::styled("█".repeat(filled), Style::default().fg(t.color())),
            Span::styled(
                "░".repeat(bw.saturating_sub(filled)),
                Style::default().fg(Color::DarkGray),
            ),
            Span::raw(format!(
                " {:5.1}%  {:>3} regions  {:>10}",
                pct,
                count,
                hsu(size)
            )),
        ]));
    }
    if let Some(r) = app
        .regions
        .get(app.region_scroll.min(app.regions.len().saturating_sub(1)))
    {
        summary.push(Line::from(""));
        summary.push(Line::from(vec![
            Span::styled("  ▸ Selected: ", Style::default().fg(Color::White).bold()),
            Span::styled(
                format!("{} ", r.region_type.label()),
                Style::default().fg(Color::Black).bg(r.region_type.color()),
            ),
            Span::raw(format!(
                " {} — {} ({})",
                trunc(&r.name, 30),
                hsu(r.size),
                r.perms
            )),
        ]));
    }
    f.render_widget(
        Paragraph::new(summary).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Memory Breakdown "),
        ),
        ch[2],
    );
}
