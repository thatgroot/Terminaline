use crate::app::App;
use crate::utils::hs;
use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

pub fn render_ram(f: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();
    let total = app.sys.total_memory();
    let used = app.sys.used_memory();
    let pct = if total > 0 {
        used as f64 / total as f64 * 100.0
    } else {
        0.0
    };
    let pc = if pct < 50.0 {
        Color::Green
    } else if pct < 80.0 {
        Color::Yellow
    } else {
        Color::Red
    };
    lines.push(Line::from(vec![Span::styled(
        "━━━ Physical Memory ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
        Style::default().fg(Color::Cyan).bold(),
    )]));
    lines.push(Line::from(vec![
        Span::styled("  Total: ", Style::default().fg(Color::White).bold()),
        Span::raw(hs(total)),
        Span::raw("  │  "),
        Span::styled("Used: ", Style::default().fg(pc).bold()),
        Span::raw(format!("{} ({:.1}%)", hs(used), pct)),
        Span::raw("  │  "),
        Span::styled("Free: ", Style::default().fg(Color::Green)),
        Span::raw(hs(total.saturating_sub(used))),
    ]));

    // Sparkline history
    if !app.ram_history.is_empty() {
        let sparkchars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
        let spark: String = app
            .ram_history
            .iter()
            .map(|v| {
                let idx = (v / 100.0 * 7.0).min(7.0) as usize;
                sparkchars[idx]
            })
            .collect();
        lines.push(Line::from(vec![
            Span::styled("  History: ", Style::default().fg(Color::Magenta)),
            Span::styled(spark, Style::default().fg(pc)),
        ]));
    }

    lines.push(Line::from(vec![
        Span::styled("  PhysMem: ", Style::default().fg(Color::Yellow)),
        Span::raw(&app.top_stats.phys_used),
        Span::raw(" used"),
        Span::raw("  ("),
        Span::styled("wired ", Style::default().fg(Color::Red)),
        Span::raw(&app.top_stats.phys_wired),
        Span::raw(", "),
        Span::styled("compressor ", Style::default().fg(Color::Magenta)),
        Span::raw(&app.top_stats.phys_compressor),
        Span::raw(")  "),
        Span::styled("unused: ", Style::default().fg(Color::Green)),
        Span::raw(&app.top_stats.phys_unused),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Regions: ", Style::default().fg(Color::Cyan)),
        Span::raw(format!("{} total", app.top_stats.mem_regions_total)),
        Span::raw(format!(
            "  (resident {}, private {}, shared {})",
            app.top_stats.mem_regions_resident,
            app.top_stats.mem_regions_private,
            app.top_stats.mem_regions_shared
        )),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  SharedLibs: ", Style::default().fg(Color::Blue)),
        Span::raw(format!(
            "{} resident, {} data",
            app.top_stats.sharedlibs_resident, app.top_stats.sharedlibs_data
        )),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "━━━ Swap ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
        Style::default().fg(Color::Magenta).bold(),
    )));
    lines.push(Line::from(vec![
        Span::styled("  Total: ", Style::default().fg(Color::White)),
        Span::raw(&app.swap_info.total),
        Span::raw("  │  "),
        Span::styled("Used: ", Style::default().fg(Color::Yellow)),
        Span::raw(&app.swap_info.used),
        Span::raw("  │  "),
        Span::styled("Free: ", Style::default().fg(Color::Green)),
        Span::raw(&app.swap_info.free),
        Span::raw(if app.swap_info.encrypted {
            "  │  🔒 Encrypted"
        } else {
            ""
        }),
    ]));
    lines.push(Line::from(""));
    let ps = app.vm_stat.page_size;
    let vm = &app.vm_stat;
    lines.push(Line::from(Span::styled(
        format!(
            "━━━ VM Page Categories (page size: {} bytes) ━━━━━━━━━━━━━━",
            ps
        ),
        Style::default().fg(Color::Green).bold(),
    )));
    let cats = [
        ("Active", vm.active, Color::Green),
        ("Inactive", vm.inactive, Color::Yellow),
        ("Speculative", vm.speculative, Color::Blue),
        ("Wired", vm.wired, Color::Red),
        ("Compressed", vm.compressor, Color::Magenta),
        ("Purgeable", vm.purgeable, Color::Cyan),
        ("Free", vm.free, Color::White),
        ("Throttled", vm.throttled, Color::DarkGray),
        ("Reactivated", vm.reactivated, Color::LightYellow),
    ];
    for (label, pages, color) in &cats {
        let size = pages * ps;
        let bar_w = 20usize;
        let total_pages =
            vm.active + vm.inactive + vm.speculative + vm.wired + vm.compressor + vm.free;
        let frac = if total_pages > 0 {
            (*pages as f64 / total_pages as f64 * bar_w as f64) as usize
        } else {
            0
        };
        lines.push(Line::from(vec![
            Span::styled(format!("  {:>12} ", label), Style::default().fg(*color)),
            Span::styled("█".repeat(frac), Style::default().fg(*color)),
            Span::styled(
                "░".repeat(bar_w.saturating_sub(frac)),
                Style::default().fg(Color::DarkGray),
            ),
            Span::raw(format!(" {:>8} pages  {:>10}", pages, hs(size))),
        ]));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "━━━ Page Fault & Compression Stats ━━━━━━━━━━━━━━━━━━━━━━━━━",
        Style::default().fg(Color::Red).bold(),
    )));
    lines.push(Line::from(vec![
        Span::styled("  Faults: ", Style::default().fg(Color::Cyan)),
        Span::raw(format!("{}", vm.faults)),
        Span::raw("  │  "),
        Span::styled("Pageins: ", Style::default().fg(Color::Green)),
        Span::raw(format!("{}", vm.pageins)),
        Span::raw("  │  "),
        Span::styled("Pageouts: ", Style::default().fg(Color::Red)),
        Span::raw(format!("{}", vm.pageouts)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  CoW: ", Style::default().fg(Color::Yellow)),
        Span::raw(format!("{}", vm.copy_on_write)),
        Span::raw("  │  "),
        Span::styled("Zero Fill: ", Style::default().fg(Color::Magenta)),
        Span::raw(format!("{}", vm.zero_fill)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Compressions: ", Style::default().fg(Color::Cyan)),
        Span::raw(format!("{}", vm.compressions)),
        Span::raw("  │  "),
        Span::styled("Decompressions: ", Style::default().fg(Color::Green)),
        Span::raw(format!("{}", vm.decompressions)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Swap In: ", Style::default().fg(Color::Yellow)),
        Span::raw(format!("{}", vm.swapins)),
        Span::raw("  │  "),
        Span::styled("Swap Out: ", Style::default().fg(Color::Red)),
        Span::raw(format!("{}", vm.swapouts)),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "━━━ Process Stats ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
        Style::default().fg(Color::Yellow).bold(),
    )));
    lines.push(Line::from(vec![
        Span::styled("  Processes: ", Style::default().fg(Color::Cyan)),
        Span::raw(format!("{}", app.top_stats.processes)),
        Span::raw("  │  "),
        Span::styled("Running: ", Style::default().fg(Color::Green)),
        Span::raw(format!("{}", app.top_stats.running)),
        Span::raw("  │  "),
        Span::styled("Sleeping: ", Style::default().fg(Color::DarkGray)),
        Span::raw(format!("{}", app.top_stats.sleeping)),
        Span::raw("  │  "),
        Span::styled("Threads: ", Style::default().fg(Color::Magenta)),
        Span::raw(format!("{}", app.top_stats.threads)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  CPU: ", Style::default().fg(Color::Cyan)),
        Span::styled(
            format!("{:.1}% user", app.top_stats.cpu_user),
            Style::default().fg(Color::Green),
        ),
        Span::raw("  "),
        Span::styled(
            format!("{:.1}% sys", app.top_stats.cpu_sys),
            Style::default().fg(Color::Red),
        ),
        Span::raw("  "),
        Span::styled(
            format!("{:.1}% idle", app.top_stats.cpu_idle),
            Style::default().fg(Color::DarkGray),
        ),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "━━━ Top Memory Consumers ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
        Style::default().fg(Color::LightBlue).bold(),
    )));
    for (i, (name, mem, cpu_pct)) in app.top_stats.top_procs.iter().enumerate() {
        lines.push(Line::from(vec![
            Span::styled(
                format!("  #{:<2} ", i + 1),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                format!("{:<25}", name),
                Style::default().fg(Color::White).bold(),
            ),
            Span::styled(format!("{:>8}", mem), Style::default().fg(Color::Yellow)),
            Span::raw("  "),
            Span::styled(format!("{:>6}", cpu_pct), Style::default().fg(Color::Cyan)),
        ]));
    }
    let ih = area.height.saturating_sub(2) as usize;
    let total_lines = lines.len();
    let scroll = app.ram_scroll.min(total_lines.saturating_sub(ih));
    let vis: Vec<Line> = lines.into_iter().skip(scroll).take(ih).collect();
    f.render_widget(
        Paragraph::new(vis).block(Block::default().borders(Borders::ALL).title(format!(
            " 🧠 RAM [{}-{}/{}] ",
            scroll + 1,
            (scroll + ih).min(total_lines),
            total_lines
        ))),
        area,
    );
    let mut sb = ScrollbarState::new(total_lines).position(scroll);
    f.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight),
        area,
        &mut sb,
    );
}
