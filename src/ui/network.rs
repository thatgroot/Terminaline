use crate::app::App;
use crate::utils::hs;
use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

pub fn render_net(f: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(Span::styled(
        "━━━ Network Totals (lifetime) ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
        Style::default().fg(Color::Cyan).bold(),
    )));
    lines.push(Line::from(vec![
        Span::styled("  Packets In: ", Style::default().fg(Color::Green)),
        Span::raw(&app.top_stats.net_packets_in),
        Span::raw(" ("),
        Span::raw(&app.top_stats.net_bytes_in),
        Span::raw(")"),
        Span::raw("  │  "),
        Span::styled("Packets Out: ", Style::default().fg(Color::Yellow)),
        Span::raw(&app.top_stats.net_packets_out),
        Span::raw(" ("),
        Span::raw(&app.top_stats.net_bytes_out),
        Span::raw(")"),
    ]));

    // Bandwidth sparkline
    if !app.net_in_history.is_empty() {
        let sparkchars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
        let max_in = app.net_in_history.iter().max().copied().unwrap_or(1).max(1);
        let spark_in: String = app
            .net_in_history
            .iter()
            .map(|v| {
                let idx = (*v as f64 / max_in as f64 * 7.0).min(7.0) as usize;
                sparkchars[idx]
            })
            .collect();
        let max_out = app
            .net_out_history
            .iter()
            .max()
            .copied()
            .unwrap_or(1)
            .max(1);
        let spark_out: String = app
            .net_out_history
            .iter()
            .map(|v| {
                let idx = (*v as f64 / max_out as f64 * 7.0).min(7.0) as usize;
                sparkchars[idx]
            })
            .collect();
        let last_in = app.net_in_history.last().copied().unwrap_or(0);
        let last_out = app.net_out_history.last().copied().unwrap_or(0);
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("  ↓ In/s:  ", Style::default().fg(Color::Green).bold()),
            Span::styled(spark_in, Style::default().fg(Color::Green)),
            Span::raw(format!(" {}/s", hs(last_in))),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  ↑ Out/s: ", Style::default().fg(Color::Yellow).bold()),
            Span::styled(spark_out, Style::default().fg(Color::Yellow)),
            Span::raw(format!(" {}/s", hs(last_out))),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "━━━ Network Interfaces ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
        Style::default().fg(Color::Green).bold(),
    )));
    for iface in &app.net_interfaces {
        let sc = if iface.status == "active" {
            Color::Green
        } else {
            Color::DarkGray
        };
        let active = iface.status == "active" || iface.pkts_in > 0 || iface.pkts_out > 0;
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {:>10} ", iface.name),
                Style::default().fg(Color::White).bold(),
            ),
            Span::styled(format!("{:>10}", iface.status), Style::default().fg(sc)),
            Span::raw(format!("  MTU:{}", iface.mtu)),
            if !iface.ip.is_empty() {
                Span::styled(
                    format!("  IP:{}", iface.ip),
                    Style::default().fg(Color::Cyan),
                )
            } else {
                Span::raw("")
            },
        ]));
        if active {
            lines.push(Line::from(vec![
                Span::raw("             "),
                Span::styled("↓ ", Style::default().fg(Color::Green)),
                Span::raw(format!("{} pkts ({})  ", iface.pkts_in, hs(iface.bytes_in))),
                Span::styled("↑ ", Style::default().fg(Color::Yellow)),
                Span::raw(format!(
                    "{} pkts ({})  ",
                    iface.pkts_out,
                    hs(iface.bytes_out)
                )),
                if iface.errs_in > 0 || iface.errs_out > 0 {
                    Span::styled(
                        format!("Err:{}/{}", iface.errs_in, iface.errs_out),
                        Style::default().fg(Color::Red),
                    )
                } else {
                    Span::raw("")
                },
            ]));
        }
    }
    let ih = area.height.saturating_sub(2) as usize;
    let total = lines.len();
    let scroll = app.net_scroll.min(total.saturating_sub(ih));
    let vis: Vec<Line> = lines.into_iter().skip(scroll).take(ih).collect();
    f.render_widget(
        Paragraph::new(vis).block(Block::default().borders(Borders::ALL).title(format!(
            " 🌐 Network [{}-{}/{}] ",
            scroll + 1,
            (scroll + ih).min(total),
            total
        ))),
        area,
    );
    let mut sb = ScrollbarState::new(total).position(scroll);
    f.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight),
        area,
        &mut sb,
    );
}
