use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn render_usb(f: &mut Frame, app: &App, area: Rect) {
    let ch = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);
    // USB
    let mut usb_lines: Vec<Line> = Vec::new();
    usb_lines.push(Line::from(Span::styled(
        format!(
            "━━━ USB Devices ({}) ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
            app.usb_devices.len()
        ),
        Style::default().fg(Color::Cyan).bold(),
    )));
    if app.usb_devices.is_empty() {
        usb_lines.push(Line::from(Span::styled(
            "  No USB devices found",
            Style::default().fg(Color::DarkGray),
        )));
    }
    for dev in &app.usb_devices {
        usb_lines.push(Line::from(vec![
            Span::styled("  🔌 ", Style::default().fg(Color::Yellow)),
            Span::styled(&dev.name, Style::default().fg(Color::White).bold()),
        ]));
        usb_lines.push(Line::from(vec![
            Span::raw("    "),
            Span::styled("Vendor: ", Style::default().fg(Color::Cyan)),
            Span::raw(if dev.vendor.is_empty() {
                &dev.vendor_id
            } else {
                &dev.vendor
            }),
            Span::raw("  │  "),
            Span::styled("PID: ", Style::default().fg(Color::Green)),
            Span::raw(&dev.product_id),
        ]));
        usb_lines.push(Line::from(vec![
            Span::raw("    "),
            Span::styled("Speed: ", Style::default().fg(Color::Magenta)),
            Span::raw(&dev.speed),
            Span::raw("  │  "),
            Span::styled("Power: ", Style::default().fg(Color::Yellow)),
            Span::raw(&dev.bus_power),
        ]));
        if !dev.serial.is_empty() {
            usb_lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled("Serial: ", Style::default().fg(Color::DarkGray)),
                Span::raw(&dev.serial),
            ]));
        }
        usb_lines.push(Line::from(""));
    }
    f.render_widget(
        Paragraph::new(usb_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" 🔌 USB Devices "),
            )
            .wrap(Wrap { trim: false }),
        ch[0],
    );
    // Thunderbolt
    let mut tb_lines: Vec<Line> = Vec::new();
    tb_lines.push(Line::from(Span::styled(
        format!(
            "━━━ Thunderbolt ({}) ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
            app.thunderbolt.len()
        ),
        Style::default().fg(Color::Yellow).bold(),
    )));
    if app.thunderbolt.is_empty() {
        tb_lines.push(Line::from(Span::styled(
            "  No Thunderbolt devices found",
            Style::default().fg(Color::DarkGray),
        )));
    }
    for dev in &app.thunderbolt {
        tb_lines.push(Line::from(vec![
            Span::styled("  ⚡ ", Style::default().fg(Color::Yellow)),
            Span::styled(&dev.device_name, Style::default().fg(Color::White).bold()),
        ]));
        tb_lines.push(Line::from(vec![
            Span::raw("    "),
            Span::styled("Speed: ", Style::default().fg(Color::Cyan)),
            Span::raw(&dev.speed),
            Span::raw("  │  "),
            Span::styled("Link: ", Style::default().fg(Color::Green)),
            Span::raw(&dev.link_status),
        ]));
        if !dev.uuid.is_empty() {
            tb_lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled("UUID: ", Style::default().fg(Color::DarkGray)),
                Span::raw(&dev.uuid),
            ]));
        }
        tb_lines.push(Line::from(""));
    }
    f.render_widget(
        Paragraph::new(tb_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" ⚡ Thunderbolt "),
            )
            .wrap(Wrap { trim: false }),
        ch[1],
    );
}
