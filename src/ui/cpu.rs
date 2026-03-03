use crate::app::App;
use crate::utils::hs;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

pub fn render_cpu(f: &mut Frame, app: &App, area: Rect) {
    let d = &app.cpu_details;
    let ch = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),
            Constraint::Length(3),
            Constraint::Min(10),
        ])
        .split(area);
    let mut id_lines = vec![
        Line::from(vec![
            Span::styled("Brand: ", Style::default().fg(Color::Cyan).bold()),
            Span::styled(&d.brand, Style::default().fg(Color::White).bold()),
        ]),
        Line::from(vec![
            Span::styled("Arch: ", Style::default().fg(Color::Yellow).bold()),
            Span::raw(&d.arch),
            Span::raw("  │  "),
            Span::styled("Cores: ", Style::default().fg(Color::Green).bold()),
            Span::raw(format!("{}", d.core_count)),
            Span::raw("  │  "),
            Span::styled("Threads: ", Style::default().fg(Color::Magenta).bold()),
            Span::raw(format!("{}", d.thread_count)),
            Span::raw("  │  "),
            Span::styled("Pkg: ", Style::default().fg(Color::LightBlue).bold()),
            Span::raw(format!("{}", d.cores_per_package)),
        ]),
        Line::from(vec![
            Span::styled("Load: ", Style::default().fg(Color::Red).bold()),
            Span::raw(format!(
                "{:.2}/{:.2}/{:.2} (1/5/15m)",
                app.load_avg[0], app.load_avg[1], app.load_avg[2]
            )),
            Span::raw("  │  "),
            Span::styled("CPU: ", Style::default().fg(Color::Cyan)),
            Span::styled(
                format!("{:.1}%u ", app.top_stats.cpu_user),
                Style::default().fg(Color::Green),
            ),
            Span::styled(
                format!("{:.1}%s ", app.top_stats.cpu_sys),
                Style::default().fg(Color::Red),
            ),
            Span::styled(
                format!("{:.1}%i", app.top_stats.cpu_idle),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
    ];
    if d.num_perf_levels >= 2 {
        id_lines.push(Line::from(vec![
            Span::styled("Topology: ", Style::default().fg(Color::Cyan).bold()),
            Span::styled(
                format!("{} P-cores", d.perf_cores),
                Style::default().fg(Color::Green).bold(),
            ),
            Span::raw(" + "),
            Span::styled(
                format!("{} E-cores", d.efficiency_cores),
                Style::default().fg(Color::Yellow).bold(),
            ),
        ]));
    }
    f.render_widget(
        Paragraph::new(id_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" 🔬 Processor "),
        ),
        ch[0],
    );
    let avg = if !app.cpu_cores.is_empty() {
        app.cpu_cores.iter().map(|c| c.usage).sum::<f32>() / app.cpu_cores.len() as f32
    } else {
        0.0
    };
    let ap = avg.min(100.0) as u16;
    let ac = if ap < 50 {
        Color::Green
    } else if ap < 80 {
        Color::Yellow
    } else {
        Color::Red
    };

    // Add sparkline to gauge label
    let spark_str = if !app.cpu_history.is_empty() {
        let sparkchars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
        let s: String = app
            .cpu_history
            .iter()
            .map(|v| {
                let idx = (v / 100.0 * 7.0).min(7.0) as usize;
                sparkchars[idx]
            })
            .collect();
        format!(" {:.1}% │ {}", avg, s)
    } else {
        format!("{:.1}%", avg)
    };

    f.render_widget(
        Gauge::default()
            .block(Block::default().borders(Borders::ALL).title(" Overall "))
            .gauge_style(Style::default().fg(ac).bg(Color::DarkGray))
            .percent(ap)
            .label(spark_str),
        ch[1],
    );
    let mut cl: Vec<Line> = Vec::new();
    cl.push(Line::from(Span::styled(
        "━━━ Cache Hierarchy ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
        Style::default().fg(Color::Cyan).bold(),
    )));
    cl.push(Line::from(vec![
        Span::styled("  Line Size: ", Style::default().fg(Color::Yellow)),
        Span::raw(format!("{} B", d.cache_line_size)),
    ]));
    if d.num_perf_levels >= 2 {
        cl.push(Line::from(Span::styled(
            "  ┌─ P-Cores ─────────────────────",
            Style::default().fg(Color::Green),
        )));
        cl.push(Line::from(vec![
            Span::styled("  │ L1i: ", Style::default().fg(Color::Green)),
            Span::raw(hs(d.perf_l1i)),
            Span::styled("  L1d: ", Style::default().fg(Color::Green)),
            Span::raw(hs(d.perf_l1d)),
        ]));
        cl.push(Line::from(vec![
            Span::styled("  │ L2:  ", Style::default().fg(Color::Green)),
            Span::raw(hs(d.perf_l2)),
        ]));
        cl.push(Line::from(Span::styled(
            "  ├─ E-Cores ─────────────────────",
            Style::default().fg(Color::Yellow),
        )));
        cl.push(Line::from(vec![
            Span::styled("  │ L1i: ", Style::default().fg(Color::Yellow)),
            Span::raw(hs(d.eff_l1i)),
            Span::styled("  L1d: ", Style::default().fg(Color::Yellow)),
            Span::raw(hs(d.eff_l1d)),
        ]));
        cl.push(Line::from(vec![
            Span::styled("  │ L2:  ", Style::default().fg(Color::Yellow)),
            Span::raw(hs(d.eff_l2)),
        ]));
        cl.push(Line::from(Span::styled(
            "  └───────────────────────────────",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        cl.push(Line::from(vec![
            Span::styled("  L1i: ", Style::default().fg(Color::Green)),
            Span::raw(hs(d.l1i_cache)),
            Span::styled("  L1d: ", Style::default().fg(Color::Green)),
            Span::raw(hs(d.l1d_cache)),
        ]));
        cl.push(Line::from(vec![
            Span::styled("  L2: ", Style::default().fg(Color::Yellow)),
            Span::raw(hs(d.l2_cache)),
        ]));
        if d.l3_cache > 0 {
            cl.push(Line::from(vec![
                Span::styled("  L3: ", Style::default().fg(Color::Red)),
                Span::raw(hs(d.l3_cache)),
            ]));
        }
    }
    cl.push(Line::from(""));
    cl.push(Line::from(Span::styled(
        "━━━ ISA Extensions ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
        Style::default().fg(Color::Magenta).bold(),
    )));
    let fpl = 6;
    let mut fi = 0;
    while fi < d.features.len() {
        let end = (fi + fpl).min(d.features.len());
        let spans: Vec<Span> = d.features[fi..end]
            .iter()
            .flat_map(|feat| {
                let c = if feat.contains("AES") || feat.contains("SHA") || feat.contains("PMULL") {
                    Color::Red
                } else if feat.contains("SIMD")
                    || feat.contains("FP16")
                    || feat.contains("BF16")
                    || feat.contains("I8MM")
                    || feat.contains("DotProd")
                {
                    Color::Cyan
                } else if feat.contains("LSE")
                    || feat.contains("CRC")
                    || feat.contains("PAuth")
                    || feat.contains("BTI")
                {
                    Color::Yellow
                } else {
                    Color::White
                };
                vec![
                    Span::styled(
                        format!(" {} ", feat),
                        Style::default().fg(Color::Black).bg(c),
                    ),
                    Span::raw(" "),
                ]
            })
            .collect();
        cl.push(Line::from([vec![Span::raw("  ")], spans].concat()));
        fi = end;
    }
    cl.push(Line::from(""));
    cl.push(Line::from(Span::styled(
        format!(
            "━━━ Per-Core ({} CPUs) ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
            app.cpu_cores.len()
        ),
        Style::default().fg(Color::Green).bold(),
    )));
    let bw = area.width.saturating_sub(32) as usize;
    for core in &app.cpu_cores {
        let p = core.usage.min(100.0);
        let fl = (p as usize * bw) / 100;
        let em = bw.saturating_sub(fl);
        let c = if p < 50.0 {
            Color::Green
        } else if p < 80.0 {
            Color::Yellow
        } else {
            Color::Red
        };
        cl.push(Line::from(vec![
            Span::styled(
                format!("  {:>5} ", core.name),
                Style::default().fg(Color::Cyan),
            ),
            Span::styled(format!("{:5.1}% ", p), Style::default().fg(c).bold()),
            Span::styled("█".repeat(fl), Style::default().fg(c)),
            Span::styled("░".repeat(em), Style::default().fg(Color::DarkGray)),
            Span::raw(format!(" {}MHz", core.frequency)),
        ]));
    }
    let ih = ch[2].height.saturating_sub(2) as usize;
    let total = cl.len();
    let scroll = app.cpu_scroll.min(total.saturating_sub(ih));
    let vis: Vec<Line> = cl.into_iter().skip(scroll).take(ih).collect();
    f.render_widget(
        Paragraph::new(vis).block(Block::default().borders(Borders::ALL).title(format!(
            " 🔍 CPU [{}-{}/{}] ",
            scroll + 1,
            (scroll + ih).min(total),
            total
        ))),
        ch[2],
    );
    let mut sb = ScrollbarState::new(total).position(scroll);
    f.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight),
        ch[2],
        &mut sb,
    );
}
