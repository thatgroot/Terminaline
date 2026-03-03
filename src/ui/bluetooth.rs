use crate::app::App;
use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn render_bluetooth(f: &mut Frame, app: &App, area: Rect) {
    let bt = &app.bluetooth;
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(Span::styled(
        "━━━ Bluetooth Controller ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
        Style::default().fg(Color::Cyan).bold(),
    )));
    lines.push(Line::from(vec![
        Span::styled("  State: ", Style::default().fg(Color::Yellow).bold()),
        Span::styled(
            &bt.state,
            Style::default()
                .fg(if bt.state.contains("On") {
                    Color::Green
                } else {
                    Color::Red
                })
                .bold(),
        ),
        Span::raw("  │  "),
        Span::styled("Address: ", Style::default().fg(Color::Cyan)),
        Span::raw(&bt.address),
        Span::raw("  │  "),
        Span::styled("Discoverable: ", Style::default().fg(Color::Magenta)),
        Span::raw(if bt.discoverable { "Yes" } else { "No" }),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Chipset: ", Style::default().fg(Color::Green)),
        Span::raw(&bt.chipset),
        Span::raw("  │  "),
        Span::styled("FW: ", Style::default().fg(Color::Blue)),
        Span::raw(&bt.firmware),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Transport: ", Style::default().fg(Color::Yellow)),
        Span::raw(&bt.transport),
        Span::raw("  │  "),
        Span::styled("Vendor: ", Style::default().fg(Color::Magenta)),
        Span::raw(&bt.vendor),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Services: ", Style::default().fg(Color::Green)),
        Span::raw(if bt.services.is_empty() { "N/A" } else { &bt.services }),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!(
            "━━━ Paired Devices ({}) ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
            bt.devices.len()
        ),
        Style::default().fg(Color::Green).bold(),
    )));
    if bt.devices.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No paired devices found",
            Style::default().fg(Color::DarkGray),
        )));
    }
    for dev in &bt.devices {
        let conn_icon = if dev.connected { "●" } else { "○" };
        let conn_color = if dev.connected {
            Color::Green
        } else {
            Color::DarkGray
        };
        lines.push(Line::from(vec![
            Span::styled(format!("  {} ", conn_icon), Style::default().fg(conn_color)),
            Span::styled(&dev.name, Style::default().fg(Color::White).bold()),
            Span::raw("  │  "),
            Span::styled("Addr: ", Style::default().fg(Color::Cyan)),
            Span::raw(&dev.address),
        ]));
        lines.push(Line::from(vec![
            Span::raw("    "),
            Span::styled("Type: ", Style::default().fg(Color::Yellow)),
            Span::raw(if dev.device_type.is_empty() {
                "Unknown"
            } else {
                &dev.device_type
            }),
            Span::raw("  │  "),
            Span::styled("FW: ", Style::default().fg(Color::Magenta)),
            Span::raw(if dev.firmware.is_empty() {
                "N/A"
            } else {
                &dev.firmware
            }),
            Span::raw("  │  "),
            Span::styled("Status: ", Style::default().fg(conn_color)),
            Span::raw(if dev.connected {
                "Connected"
            } else {
                "Disconnected"
            }),
        ]));
        lines.push(Line::from(""));
    }
    f.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" 🔵 Bluetooth "),
            )
            .wrap(Wrap { trim: false }),
        area,
    );
}
