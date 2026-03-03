pub mod activity;
pub mod audio;
pub mod battery;
pub mod bluetooth;
pub mod camera;
pub mod cpu;
pub mod disk;
pub mod gpu;
pub mod network;
pub mod process;
pub mod ram;
pub mod regions;
pub mod security;
pub mod services;
pub mod thermal;
pub mod usb;
pub mod visual;
pub mod wifi;

use crate::app::App;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

pub fn ui(f: &mut Frame, app: &App) {
    let ch = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(f.area());
    let titles: Vec<Line> = [
        "1:RAM", "2:Map", "3:Vis", "4:CPU", "5:Disk", "6:Net", "7:GPU", "8:Bat", "9:Cam", "0:Act",
        "P:Proc", "B:BT", "U:USB", "A:Aud", "X:Sec", "L:Svc", "W:WiFi", "T:Therm",
    ]
    .iter()
    .map(|t| Line::from(format!(" {} ", t)))
    .collect();
    f.render_widget(
        Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL).title(format!(
                " ⚡ {} │ {} │ up {} ",
                app.sys_info.hostname, app.sys_info.os_type, app.sys_info.uptime
            )))
            .highlight_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .select(app.tab),
        ch[0],
    );
    match app.tab {
        0 => ram::render_ram(f, app, ch[1]),
        1 => regions::render_regions(f, app, ch[1]),
        2 => visual::render_visual(f, app, ch[1]),
        3 => cpu::render_cpu(f, app, ch[1]),
        4 => disk::render_disk(f, app, ch[1]),
        5 => network::render_net(f, app, ch[1]),
        6 => gpu::render_gpu(f, app, ch[1]),
        7 => battery::render_battery(f, app, ch[1]),
        8 => camera::render_camera(f, app, ch[1]),
        9 => activity::render_activity(f, app, ch[1]),
        10 => process::render_process(f, app, ch[1]),
        11 => bluetooth::render_bluetooth(f, app, ch[1]),
        12 => usb::render_usb(f, app, ch[1]),
        13 => audio::render_audio(f, app, ch[1]),
        14 => security::render_security(f, app, ch[1]),
        15 => services::render_services(f, app, ch[1]),
        16 => wifi::render_wifi(f, app, ch[1]),
        17 => thermal::render_thermal(f, app, ch[1]),
        _ => {}
    }
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" q", Style::default().fg(Color::Red).bold()),
            Span::raw(":Quit "),
            Span::styled("1-0", Style::default().fg(Color::Cyan).bold()),
            Span::raw(":Tab "),
            Span::styled("PBUAXLWT", Style::default().fg(Color::Magenta).bold()),
            Span::raw(":More "),
            Span::styled("↑↓/jk", Style::default().fg(Color::Yellow).bold()),
            Span::raw(":Scroll "),
            Span::raw(format!(
                "│ PID:{} │ Tick:{} │ {}/{}/{}",
                app.pid,
                app.tick_count,
                app.sys_info.os_type,
                app.sys_info.os_release,
                app.sys_info.os_build
            )),
        ]))
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center),
        ch[2],
    );
}
