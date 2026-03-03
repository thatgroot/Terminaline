use crate::app::App;
use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn render_thermal(f: &mut Frame, app: &App, area: Rect) {
    let t = &app.thermal;
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(Span::styled(
        "━━━ Thermal Overview ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
        Style::default().fg(Color::Cyan).bold(),
    )));
    let pressure_color = match t.thermal_pressure.as_str() {
        "Normal" => Color::Green,
        "Warning" => Color::Yellow,
        "Urgent" => Color::Red,
        "Critical" => Color::LightRed,
        _ => Color::White,
    };
    lines.push(Line::from(vec![
        Span::styled(
            "  Thermal Pressure: ",
            Style::default().fg(Color::Yellow).bold(),
        ),
        Span::styled(
            &t.thermal_pressure,
            Style::default().fg(pressure_color).bold(),
        ),
    ]));
    lines.push(Line::from(""));
    if t.entries.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No thermal sensors available without elevated privileges",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(Span::styled(
            "  (powermetrics requires sudo for detailed sensor data)",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            format!(
                "━━━ Sensors ({}) ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
                t.entries.len()
            ),
            Style::default().fg(Color::Green).bold(),
        )));
        for entry in &t.entries {
            let temp_color = if entry.temperature > 80.0 {
                Color::Red
            } else if entry.temperature > 50.0 {
                Color::Yellow
            } else if entry.temperature > 0.0 {
                Color::Green
            } else {
                Color::Cyan
            };
            let bar_w = 20usize;
            let filled = if entry.temperature > 0.0 {
                ((entry.temperature / 100.0) * bar_w as f64).min(bar_w as f64) as usize
            } else {
                0
            };
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {:>20} ", entry.name),
                    Style::default().fg(Color::White).bold(),
                ),
                Span::styled("█".repeat(filled), Style::default().fg(temp_color)),
                Span::styled(
                    "░".repeat(bar_w.saturating_sub(filled)),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("  {:.1}°C", entry.temperature),
                    Style::default().fg(temp_color).bold(),
                ),
                Span::raw(format!("  [{}]", entry.category)),
            ]));
        }
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "━━━ Notes ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
        Style::default().fg(Color::DarkGray).bold(),
    )));
    lines.push(Line::from(Span::styled(
        "  • Thermal levels (0=Normal, higher=throttling)",
        Style::default().fg(Color::DarkGray),
    )));
    lines.push(Line::from(Span::styled(
        "  • Battery temp from AppleSmartBattery (÷100 = °C)",
        Style::default().fg(Color::DarkGray),
    )));
    lines.push(Line::from(Span::styled(
        "  • For detailed CPU/GPU temps, run: sudo powermetrics",
        Style::default().fg(Color::DarkGray),
    )));
    f.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" 🌡️ Thermal / Sensors "),
            )
            .wrap(Wrap { trim: false }),
        area,
    );
}
