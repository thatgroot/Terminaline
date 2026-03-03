use crate::app::App;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_camera(f: &mut Frame, app: &App, area: Rect) {
    if app.cameras.is_empty() {
        f.render_widget(
            Paragraph::new(vec![
                Line::from(""),
                Line::from(Span::styled(
                    "No cameras detected",
                    Style::default().fg(Color::Yellow).bold(),
                )),
                Line::from(""),
                Line::from("Reads from: system_profiler SPCameraDataType"),
            ])
            .block(Block::default().borders(Borders::ALL).title(" 📷 Cameras "))
            .alignment(Alignment::Center),
            area,
        );
        return;
    }
    let mut lines: Vec<Line> = Vec::new();
    for (i, cam) in app.cameras.iter().enumerate() {
        lines.push(Line::from(Span::styled(
            format!(
                "━━━ Camera {} ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
                i + 1
            ),
            Style::default().fg(Color::Cyan).bold(),
        )));
        lines.push(Line::from(vec![
            Span::styled("  Name: ", Style::default().fg(Color::Yellow)),
            Span::styled(&cam.name, Style::default().fg(Color::White).bold()),
            Span::raw("  "),
            Span::styled("● Connected", Style::default().fg(Color::Green)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Model: ", Style::default().fg(Color::Magenta)),
            Span::raw(if cam.model_id.is_empty() {
                "N/A"
            } else {
                &cam.model_id
            }),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  UID: ", Style::default().fg(Color::Blue)),
            Span::raw(if cam.unique_id.is_empty() {
                "N/A"
            } else {
                &cam.unique_id
            }),
        ]));
        lines.push(Line::from(""));
    }
    f.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" 📷 Cameras ({}) ", app.cameras.len())),
        ),
        area,
    );
}
