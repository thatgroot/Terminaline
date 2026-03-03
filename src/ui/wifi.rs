use crate::app::App;
use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn render_wifi(f: &mut Frame, app: &App, area: Rect) {
    let w = &app.wifi;
    let connected = !w.ssid.is_empty();
    let rssi_val: i32 = w.rssi.trim_end_matches(" dBm").parse().unwrap_or(0);
    let signal_strength = if rssi_val >= -50 {
        ("Excellent", Color::Green)
    } else if rssi_val >= -60 {
        ("Good", Color::Cyan)
    } else if rssi_val >= -70 {
        ("Fair", Color::Yellow)
    } else {
        ("Weak", Color::Red)
    };
    let lines = vec![
        Line::from(Span::styled(
            "━━━ Wi-Fi Connection ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
            Style::default().fg(Color::Cyan).bold(),
        )),
        Line::from(vec![
            Span::styled("  Status: ", Style::default().fg(Color::Yellow).bold()),
            if connected {
                Span::styled("● Connected", Style::default().fg(Color::Green).bold())
            } else {
                Span::styled("○ Disconnected", Style::default().fg(Color::Red).bold())
            },
        ]),
        Line::from(vec![
            Span::styled("  Interface: ", Style::default().fg(Color::Cyan)),
            Span::raw(&w.interface),
            Span::raw("  │  "),
            Span::styled("Hardware: ", Style::default().fg(Color::Magenta)),
            Span::raw(if w.hardware.is_empty() {
                "N/A"
            } else {
                &w.hardware
            }),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "━━━ Network ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
            Style::default().fg(Color::Green).bold(),
        )),
        Line::from(vec![
            Span::styled("  SSID: ", Style::default().fg(Color::Yellow).bold()),
            Span::styled(
                if w.ssid.is_empty() {
                    "Not connected"
                } else {
                    &w.ssid
                },
                Style::default().fg(Color::White).bold(),
            ),
        ]),
        Line::from(vec![
            Span::styled("  BSSID: ", Style::default().fg(Color::Cyan)),
            Span::raw(if w.bssid.is_empty() { "N/A" } else { &w.bssid }),
        ]),
        Line::from(vec![
            Span::styled("  Security: ", Style::default().fg(Color::Green)),
            Span::raw(if w.security_type.is_empty() {
                "N/A"
            } else {
                &w.security_type
            }),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "━━━ Signal Quality ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
            Style::default().fg(Color::Magenta).bold(),
        )),
        Line::from(vec![
            Span::styled("  RSSI: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                if w.rssi.is_empty() {
                    "N/A".into()
                } else {
                    w.rssi.clone()
                },
                Style::default().fg(signal_strength.1),
            ),
            Span::raw("  │  "),
            Span::styled("Noise: ", Style::default().fg(Color::Red)),
            Span::raw(if w.noise.is_empty() { "N/A" } else { &w.noise }),
            Span::raw("  │  "),
            Span::styled("Quality: ", Style::default().fg(Color::Cyan)),
            Span::styled(
                signal_strength.0,
                Style::default().fg(signal_strength.1).bold(),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Channel: ", Style::default().fg(Color::Green)),
            Span::raw(if w.channel.is_empty() {
                "N/A"
            } else {
                &w.channel
            }),
            Span::raw("  │  "),
            Span::styled("PHY Mode: ", Style::default().fg(Color::Magenta)),
            Span::raw(if w.phy_mode.is_empty() {
                "N/A"
            } else {
                &w.phy_mode
            }),
        ]),
        Line::from(vec![
            Span::styled("  TX Rate: ", Style::default().fg(Color::Yellow)),
            Span::raw(if w.tx_rate.is_empty() {
                "N/A"
            } else {
                &w.tx_rate
            }),
            Span::raw("  │  "),
            Span::styled("Country: ", Style::default().fg(Color::Cyan)),
            Span::raw(if w.country_code.is_empty() {
                "N/A"
            } else {
                &w.country_code
            }),
        ]),
    ];
    f.render_widget(
        Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title(" 📶 Wi-Fi "))
            .wrap(Wrap { trim: false }),
        area,
    );
}
