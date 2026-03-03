use crate::app::App;
use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn render_audio(f: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();
    let inputs: Vec<_> = app.audio_devices.iter().filter(|d| d.is_input).collect();
    let outputs: Vec<_> = app.audio_devices.iter().filter(|d| !d.is_input).collect();
    lines.push(Line::from(Span::styled(
        format!(
            "━━━ Output Devices ({}) ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
            outputs.len()
        ),
        Style::default().fg(Color::Green).bold(),
    )));
    for dev in &outputs {
        let icon = if dev.is_default { "🔊" } else { "🔈" };
        lines.push(Line::from(vec![
            Span::raw(format!("  {} ", icon)),
            Span::styled(&dev.name, Style::default().fg(Color::White).bold()),
            if dev.is_default {
                Span::styled(" [DEFAULT]", Style::default().fg(Color::Green).bold())
            } else {
                Span::raw("")
            },
        ]));
        lines.push(Line::from(vec![
            Span::raw("    "),
            Span::styled("Manufacturer: ", Style::default().fg(Color::Cyan)),
            Span::raw(if dev.manufacturer.is_empty() {
                "N/A"
            } else {
                &dev.manufacturer
            }),
            Span::raw("  │  "),
            Span::styled("Transport: ", Style::default().fg(Color::Yellow)),
            Span::raw(if dev.transport.is_empty() {
                "Built-in"
            } else {
                &dev.transport
            }),
        ]));
        lines.push(Line::from(vec![
            Span::raw("    "),
            Span::styled("Sample Rate: ", Style::default().fg(Color::Magenta)),
            Span::raw(if dev.sample_rate > 0 {
                format!("{} Hz", dev.sample_rate)
            } else {
                "N/A".into()
            }),
        ]));
        lines.push(Line::from(""));
    }
    lines.push(Line::from(Span::styled(
        format!(
            "━━━ Input Devices ({}) ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
            inputs.len()
        ),
        Style::default().fg(Color::Cyan).bold(),
    )));
    for dev in &inputs {
        let icon = if dev.is_default { "🎙️" } else { "🎤" };
        lines.push(Line::from(vec![
            Span::raw(format!("  {} ", icon)),
            Span::styled(&dev.name, Style::default().fg(Color::White).bold()),
            if dev.is_default {
                Span::styled(" [DEFAULT]", Style::default().fg(Color::Green).bold())
            } else {
                Span::raw("")
            },
        ]));
        lines.push(Line::from(vec![
            Span::raw("    "),
            Span::styled("Manufacturer: ", Style::default().fg(Color::Cyan)),
            Span::raw(if dev.manufacturer.is_empty() {
                "N/A"
            } else {
                &dev.manufacturer
            }),
            Span::raw("  │  "),
            Span::styled("Transport: ", Style::default().fg(Color::Yellow)),
            Span::raw(if dev.transport.is_empty() {
                "Built-in"
            } else {
                &dev.transport
            }),
        ]));
        lines.push(Line::from(""));
    }
    f.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" 🎧 Audio Devices ({}) ", app.audio_devices.len())),
            )
            .wrap(Wrap { trim: false }),
        area,
    );
}
