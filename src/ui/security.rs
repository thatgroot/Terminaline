use crate::app::App;
use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn render_security(f: &mut Frame, app: &App, area: Rect) {
    let s = &app.security;
    let on_off = |b: bool| -> (String, Color) {
        if b {
            ("✓ Enabled".into(), Color::Green)
        } else {
            ("✗ Disabled".into(), Color::Red)
        }
    };
    let (sip_label, sip_color) = on_off(s.sip_enabled);
    let (gk_label, gk_color) = on_off(s.gatekeeper_enabled);
    let (fv_label, fv_color) = on_off(s.filevault_enabled);
    let (fw_label, fw_color) = on_off(s.firewall_enabled);
    let lines = vec![
        Line::from(Span::styled(
            "━━━ System Integrity Protection (SIP) ━━━━━━━━━━━━━━━━━━━━━━",
            Style::default().fg(Color::Cyan).bold(),
        )),
        Line::from(vec![
            Span::styled("  Status: ", Style::default().fg(Color::Yellow).bold()),
            Span::styled(&sip_label, Style::default().fg(sip_color).bold()),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(&s.sip_status, Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "━━━ Gatekeeper ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
            Style::default().fg(Color::Green).bold(),
        )),
        Line::from(vec![
            Span::styled("  Status: ", Style::default().fg(Color::Yellow).bold()),
            Span::styled(&gk_label, Style::default().fg(gk_color).bold()),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(&s.gatekeeper_status, Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "━━━ FileVault (Disk Encryption) ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
            Style::default().fg(Color::Magenta).bold(),
        )),
        Line::from(vec![
            Span::styled("  Status: ", Style::default().fg(Color::Yellow).bold()),
            Span::styled(&fv_label, Style::default().fg(fv_color).bold()),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(&s.filevault_status, Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "━━━ Firewall ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
            Style::default().fg(Color::Red).bold(),
        )),
        Line::from(vec![
            Span::styled("  Status: ", Style::default().fg(Color::Yellow).bold()),
            Span::styled(&fw_label, Style::default().fg(fw_color).bold()),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(&s.firewall_status, Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled("  Stealth Mode: ", Style::default().fg(Color::Cyan)),
            Span::styled(
                if s.firewall_stealth {
                    "✓ On"
                } else {
                    "✗ Off"
                },
                Style::default().fg(if s.firewall_stealth {
                    Color::Green
                } else {
                    Color::DarkGray
                }),
            ),
            Span::raw("  │  "),
            Span::styled("Block All: ", Style::default().fg(Color::Red)),
            Span::styled(
                if s.firewall_block_all {
                    "✓ On"
                } else {
                    "✗ Off"
                },
                Style::default().fg(if s.firewall_block_all {
                    Color::Red
                } else {
                    Color::DarkGray
                }),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "━━━ Security Summary ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
            Style::default().fg(Color::Yellow).bold(),
        )),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("SIP", Style::default().fg(sip_color).bold()),
            Span::raw("  "),
            Span::styled("GK", Style::default().fg(gk_color).bold()),
            Span::raw("  "),
            Span::styled("FV", Style::default().fg(fv_color).bold()),
            Span::raw("  "),
            Span::styled("FW", Style::default().fg(fw_color).bold()),
            Span::raw("  │  Overall: "),
            if s.sip_enabled && s.gatekeeper_enabled && s.filevault_enabled && s.firewall_enabled {
                Span::styled(
                    "🛡️  Excellent — All protections active",
                    Style::default().fg(Color::Green).bold(),
                )
            } else if s.sip_enabled && s.gatekeeper_enabled {
                Span::styled(
                    "⚠️  Good — Some protections disabled",
                    Style::default().fg(Color::Yellow).bold(),
                )
            } else {
                Span::styled(
                    "🔓 Warning — Critical protections disabled",
                    Style::default().fg(Color::Red).bold(),
                )
            },
        ]),
    ];
    f.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" 🔐 Security Status "),
            )
            .wrap(Wrap { trim: false }),
        area,
    );
}
