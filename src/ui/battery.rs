use crate::app::App;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

pub fn render_battery(f: &mut Frame, app: &App, area: Rect) {
    let b = &app.battery;
    if !b.present {
        f.render_widget(
            Paragraph::new("No battery detected (desktop Mac?)")
                .block(Block::default().borders(Borders::ALL).title(" 🔋 Battery "))
                .alignment(Alignment::Center),
            area,
        );
        return;
    }
    let pct_num: u16 = b.level.trim_end_matches('%').parse().unwrap_or(0);
    let bc = if pct_num > 50 {
        Color::Green
    } else if pct_num > 20 {
        Color::Yellow
    } else {
        Color::Red
    };
    let ch = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(10)])
        .split(area);
    f.render_widget(
        Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" 🔋 Battery — {} ", b.state)),
            )
            .gauge_style(Style::default().fg(bc).bg(Color::DarkGray))
            .percent(pct_num)
            .label(format!("{} — {} — {}", b.level, b.state, b.remaining)),
        ch[0],
    );
    let health_pct = if !b.max_capacity.is_empty() && !b.design_capacity.is_empty() {
        let max: f64 = b.max_capacity.parse().unwrap_or(0.0);
        let design: f64 = b.design_capacity.parse().unwrap_or(1.0);
        if design > 0.0 {
            format!("{:.1}%", max / design * 100.0)
        } else {
            "N/A".into()
        }
    } else {
        "N/A".into()
    };
    let temp_c = if !b.temperature.is_empty() {
        let raw: f64 = b.temperature.parse().unwrap_or(0.0);
        format!("{:.1}°C", raw / 100.0)
    } else {
        "N/A".into()
    };
    let voltage_v = if !b.voltage.is_empty() {
        let mv: f64 = b.voltage.parse().unwrap_or(0.0);
        format!("{:.3} V", mv / 1000.0)
    } else {
        "N/A".into()
    };
    let amp_s = if !b.amperage.is_empty() {
        let ma: f64 = b.amperage.parse().unwrap_or(0.0);
        format!("{:.0} mA", ma)
    } else {
        "N/A".into()
    };
    let lines = vec![
        Line::from(Span::styled(
            "━━━ Battery Details ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
            Style::default().fg(Color::Cyan).bold(),
        )),
        Line::from(vec![
            Span::styled("  Charge: ", Style::default().fg(Color::Green).bold()),
            Span::raw(&b.level),
            Span::raw("  │  "),
            Span::styled("State: ", Style::default().fg(Color::Yellow)),
            Span::raw(&b.state),
        ]),
        Line::from(vec![
            Span::styled("  Remaining: ", Style::default().fg(Color::Magenta)),
            Span::raw(&b.remaining),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "━━━ Health & Cycles ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
            Style::default().fg(Color::Yellow).bold(),
        )),
        Line::from(vec![
            Span::styled("  Cycle Count: ", Style::default().fg(Color::Cyan)),
            Span::raw(&b.cycle_count),
        ]),
        Line::from(vec![
            Span::styled("  Condition: ", Style::default().fg(Color::Green)),
            Span::raw(&b.condition),
        ]),
        Line::from(vec![
            Span::styled("  Health: ", Style::default().fg(Color::Magenta)),
            Span::raw(&health_pct),
            Span::raw(format!(
                "  (max: {} / design: {})",
                b.max_capacity, b.design_capacity
            )),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "━━━ Electrical ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
            Style::default().fg(Color::Green).bold(),
        )),
        Line::from(vec![
            Span::styled("  Voltage: ", Style::default().fg(Color::Cyan)),
            Span::raw(&voltage_v),
            Span::raw("  │  "),
            Span::styled("Current: ", Style::default().fg(Color::Yellow)),
            Span::raw(&amp_s),
        ]),
        Line::from(vec![
            Span::styled("  Temperature: ", Style::default().fg(Color::Red)),
            Span::raw(&temp_c),
        ]),
    ];
    f.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Battery Details "),
        ),
        ch[1],
    );
}
