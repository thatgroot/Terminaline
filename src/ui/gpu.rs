use crate::app::App;
use crate::utils::hs;
use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn render_gpu(f: &mut Frame, app: &App, area: Rect) {
    let g = &app.gpu;
    let lines = vec![
        Line::from(Span::styled(
            "━━━ GPU Information ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
            Style::default().fg(Color::Cyan).bold(),
        )),
        Line::from(vec![
            Span::styled("  Chipset: ", Style::default().fg(Color::Yellow).bold()),
            Span::styled(&g.chipset, Style::default().fg(Color::White).bold()),
        ]),
        Line::from(vec![
            Span::styled("  Type: ", Style::default().fg(Color::Green)),
            Span::raw(&g.gpu_type),
            Span::raw("  │  "),
            Span::styled("Bus: ", Style::default().fg(Color::Cyan)),
            Span::raw(&g.bus),
        ]),
        Line::from(vec![
            Span::styled("  GPU Cores: ", Style::default().fg(Color::Magenta)),
            Span::raw(&g.cores),
            Span::raw("  │  "),
            Span::styled("Vendor: ", Style::default().fg(Color::Blue)),
            Span::raw(&g.vendor),
        ]),
        Line::from(vec![
            Span::styled("  Metal: ", Style::default().fg(Color::LightRed).bold()),
            Span::styled(&g.metal, Style::default().fg(Color::Green).bold()),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "━━━ Display ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
            Style::default().fg(Color::Yellow).bold(),
        )),
        Line::from(vec![
            Span::styled("  Display: ", Style::default().fg(Color::Cyan)),
            Span::raw(&g.display_name),
        ]),
        Line::from(vec![
            Span::styled("  Type: ", Style::default().fg(Color::Green)),
            Span::raw(&g.display_type),
        ]),
        Line::from(vec![
            Span::styled("  Resolution: ", Style::default().fg(Color::Magenta).bold()),
            Span::styled(&g.resolution, Style::default().fg(Color::White).bold()),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "━━━ Unified Memory Architecture ━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
            Style::default().fg(Color::Green).bold(),
        )),
        Line::from(vec![
            Span::styled("  Physical Memory: ", Style::default().fg(Color::Cyan)),
            Span::raw(hs(app.cpu_details.phys_mem)),
        ]),
        Line::from(vec![
            Span::styled("  Note: ", Style::default().fg(Color::DarkGray)),
            Span::raw("Apple Silicon uses UMA. GPU and CPU share physical memory."),
        ]),
    ];
    f.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" 🎮 GPU / Display "),
            )
            .wrap(Wrap { trim: false }),
        area,
    );
}
