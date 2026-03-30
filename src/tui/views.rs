use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Cell, Clear, Paragraph, Row, Sparkline, Table, Wrap,
};
use ratatui::Frame;

use crate::firewall::events::Verdict;
use crate::tui::firewall_app::{App, FeedFilter, View};
use crate::tui::theme;
use crate::utils::{truncate_str, format_number, format_bytes_short};

// ── Main render dispatcher ──────────────────────────────────────────────────

pub fn render(f: &mut Frame, app: &App) {
    let area = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // info bar
            Constraint::Length(1), // tab strip
            Constraint::Length(1), // separator
            Constraint::Min(0),    // content
            Constraint::Length(1), // status bar
        ])
        .split(area);

    render_info_bar(f, app, chunks[0]);
    render_tab_strip(f, app, chunks[1]);
    render_separator(f, chunks[2]);

    match app.current_view {
        View::Dashboard   => render_dashboard(f, app, chunks[3]),
        View::LiveFeed    => render_live_feed(f, app, chunks[3]),
        View::Blocked     => render_blocked(f, app, chunks[3]),
        View::IpInspector => render_ip_inspector(f, app, chunks[3]),
        View::Rules       => render_rules(f, app, chunks[3]),
        View::Stats       => render_stats(f, app, chunks[3]),
        View::Config      => render_config(f, app, chunks[3]),
    }

    render_status_line(f, app, chunks[4]);

    if app.show_help {
        render_help_overlay(f, area);
    }
}

// ── Header: info bar ────────────────────────────────────────────────────────

fn render_info_bar(f: &mut Frame, app: &App, area: Rect) {
    let snap = app.stats_snapshot();

    let mut spans = vec![
        Span::styled(" ◆ INFYNON FIREWALL", theme::title_style()),
        Span::styled("  │  ", Style::default().fg(theme::DIMMER)),
    ];

    if app.is_maintenance() {
        spans.push(Span::styled("⚠ MAINTENANCE  │  ", Style::default().fg(theme::ORANGE).add_modifier(Modifier::BOLD)));
    }

    spans.push(Span::styled(
        format!("↑ {:.0}/s  ", snap.requests_per_second),
        Style::default().fg(theme::CYAN),
    ));
    spans.push(Span::styled(
        format!("✘ {:.0}/s blocked  ", snap.blocks_per_second),
        Style::default().fg(theme::RED),
    ));
    spans.push(Span::styled("│  ", Style::default().fg(theme::DIMMER)));
    spans.push(Span::styled(
        format!("{} conn", snap.active_connections),
        Style::default().fg(theme::DIM),
    ));

    if app.paused {
        spans.push(Span::styled("  │  ⏸ PAUSED", Style::default().fg(theme::YELLOW).add_modifier(Modifier::BOLD)));
    }

    // Shortcuts
    spans.push(Span::styled("     ", Style::default()));
    for (key, label) in &[("m", "maint"), ("r", "reload"), ("/", "search"), ("?", "help"), ("q", "quit")] {
        spans.push(Span::styled(key.to_string(), Style::default().fg(theme::YELLOW).add_modifier(Modifier::BOLD)));
        spans.push(Span::styled(format!(" {}  ", label), Style::default().fg(theme::DIMMER)));
    }

    let p = Paragraph::new(Line::from(spans))
        .style(Style::default().bg(theme::BG_HIGHLIGHT));
    f.render_widget(p, area);
}

// ── Header: tab strip ────────────────────────────────────────────────────────

fn render_tab_strip(f: &mut Frame, app: &App, area: Rect) {
    let mut spans: Vec<Span> = vec![Span::raw(" ")];

    for view in View::all() {
        let is_active = app.current_view == *view;
        let num = view.key().to_string();
        let name = view.label();

        if is_active {
            spans.push(Span::styled("▌", Style::default().fg(theme::CYAN).bg(theme::BG_HIGHLIGHT)));
            spans.push(Span::styled(
                format!(" {} · {} ", num, name),
                Style::default().fg(theme::BG).bg(theme::CYAN).add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled("▐ ", Style::default().fg(theme::CYAN).bg(theme::BG_HIGHLIGHT)));
        } else {
            spans.push(Span::styled(format!(" {} ", num), Style::default().fg(theme::DIMMER)));
            spans.push(Span::styled(format!("{}  ", name), Style::default().fg(theme::DIM)));
        }
    }

    let p = Paragraph::new(Line::from(spans))
        .style(Style::default().bg(theme::BG_HIGHLIGHT));
    f.render_widget(p, area);
}

// ── Header: separator ────────────────────────────────────────────────────────

fn render_separator(f: &mut Frame, area: Rect) {
    let line = "─".repeat(area.width as usize);
    let p = Paragraph::new(Line::from(vec![
        Span::styled(line, Style::default().fg(theme::BORDER)),
    ]));
    f.render_widget(p, area);
}

// ── Status line ─────────────────────────────────────────────────────────────

fn render_status_line(f: &mut Frame, app: &App, area: Rect) {
    let snap = app.stats_snapshot();

    // Notification toast takes over
    if let Some(notif) = app.active_notification() {
        let p = Paragraph::new(Line::from(vec![
            Span::styled(" ✦ ", Style::default().fg(theme::YELLOW).add_modifier(Modifier::BOLD)),
            Span::styled(notif, Style::default().fg(theme::YELLOW)),
        ]))
        .style(Style::default().bg(theme::BG_HIGHLIGHT));
        f.render_widget(p, area);
        return;
    }

    let mut spans = vec![
        Span::styled(" ● RUNNING  │  ", Style::default().fg(theme::GREEN)),
        Span::styled(
            format!("{} req  {} blocked  {} flagged", format_number(snap.total_requests), format_number(snap.total_blocked), snap.total_flagged),
            Style::default().fg(theme::TEXT_DIM),
        ),
        Span::styled("  │  ", Style::default().fg(theme::DIMMER)),
    ];

    if !app.search_input.is_empty() {
        spans.push(Span::styled("/ ", Style::default().fg(theme::YELLOW)));
        spans.push(Span::styled(&app.search_input, Style::default().fg(theme::WHITE).add_modifier(Modifier::BOLD)));
        spans.push(Span::styled("  │  ", Style::default().fg(theme::DIMMER)));
    }

    spans.push(Span::styled("p pause  f filter  b/u block/unblock", Style::default().fg(theme::DIMMER)));

    let p = Paragraph::new(Line::from(spans))
        .style(Style::default().bg(theme::BG_HIGHLIGHT));
    f.render_widget(p, area);
}

// ── Help Overlay ────────────────────────────────────────────────────────────

fn render_help_overlay(f: &mut Frame, area: Rect) {
    let help_area = centered_rect(60, 70, area);
    f.render_widget(Clear, help_area);

    let help_text = vec![
        Line::from(Span::styled(" KEYBOARD SHORTCUTS ", Style::default().fg(theme::CYAN).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(vec![Span::styled(" Global Keys", Style::default().fg(theme::YELLOW).add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::styled("  1-7    ", theme::stat_value()), Span::styled("Switch between views", theme::dim_style())]),
        Line::from(vec![Span::styled("  q      ", theme::stat_value()), Span::styled("Quit TUI (firewall keeps running)", theme::dim_style())]),
        Line::from(vec![Span::styled("  /      ", theme::stat_value()), Span::styled("Search/filter", theme::dim_style())]),
        Line::from(vec![Span::styled("  ?      ", theme::stat_value()), Span::styled("Toggle this help", theme::dim_style())]),
        Line::from(vec![Span::styled("  r      ", theme::stat_value()), Span::styled("Reload config from file", theme::dim_style())]),
        Line::from(vec![Span::styled("  m      ", theme::stat_value()), Span::styled("Toggle maintenance mode", theme::dim_style())]),
        Line::from(""),
        Line::from(vec![Span::styled(" Live Feed / Blocked", Style::default().fg(theme::YELLOW).add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::styled("  p      ", theme::stat_value()), Span::styled("Pause/resume auto-scroll", theme::dim_style())]),
        Line::from(vec![Span::styled("  f      ", theme::stat_value()), Span::styled("Cycle filter (All/Blocked/Allowed/Flagged)", theme::dim_style())]),
        Line::from(vec![Span::styled("  Up/Dn  ", theme::stat_value()), Span::styled("Scroll", theme::dim_style())]),
        Line::from(""),
        Line::from(vec![Span::styled(" IP Inspector", Style::default().fg(theme::YELLOW).add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::styled("  b      ", theme::stat_value()), Span::styled("Block selected IP", theme::dim_style())]),
        Line::from(vec![Span::styled("  u      ", theme::stat_value()), Span::styled("Unblock selected IP", theme::dim_style())]),
        Line::from(""),
        Line::from(vec![Span::styled(" Config", Style::default().fg(theme::YELLOW).add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::styled("  Enter  ", theme::stat_value()), Span::styled("Edit selected field", theme::dim_style())]),
        Line::from(vec![Span::styled("  s      ", theme::stat_value()), Span::styled("Save config to file", theme::dim_style())]),
        Line::from(""),
        Line::from(Span::styled(" Press any key to close ", theme::dim_style())),
    ];

    let help_p = Paragraph::new(help_text)
        .block(Block::default()
            .title(Span::styled(" Help ", theme::title_style()))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::CYAN)))
        .wrap(Wrap { trim: false });
    f.render_widget(help_p, help_area);
}

// ── Dashboard View ──────────────────────────────────────────────────────────

fn render_dashboard(f: &mut Frame, app: &App, area: Rect) {
    let snap = app.stats_snapshot();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Status bar
            Constraint::Length(5),  // Sparklines
            Constraint::Min(0),    // Content split
        ])
        .split(area);

    // Status bar
    let uptime = snap.format_uptime();
    let maint_span = if app.is_maintenance() {
        Span::styled("  MAINTENANCE", Style::default().fg(theme::ORANGE).add_modifier(Modifier::BOLD))
    } else {
        Span::raw("")
    };

    let status_text = vec![
        Span::styled("  Status: ", theme::stat_label()),
        Span::styled("RUNNING", theme::status_running()),
        maint_span,
        Span::styled("    Uptime: ", theme::stat_label()),
        Span::styled(uptime, theme::stat_value()),
        Span::styled("    Requests: ", theme::stat_label()),
        Span::styled(format_number(snap.total_requests), theme::stat_value()),
        Span::styled("    Blocked: ", theme::stat_label()),
        Span::styled(format_number(snap.total_blocked), Style::default().fg(theme::RED).add_modifier(Modifier::BOLD)),
    ];
    let status_p = Paragraph::new(Line::from(status_text))
        .block(Block::default().borders(Borders::BOTTOM).border_style(theme::border_style()));
    f.render_widget(status_p, chunks[0]);

    // Sparklines
    let spark_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    let traffic_data: Vec<u64> = snap.traffic_last_60s.iter().map(|v| *v as u64).collect();
    let traffic_spark = Sparkline::default()
        .block(Block::default()
            .title(Span::styled(
                format!(" Traffic: {:.0} req/s ", snap.requests_per_second),
                theme::title_style(),
            ))
            .borders(Borders::ALL)
            .border_style(theme::border_style()))
        .data(&traffic_data)
        .style(Style::default().fg(theme::CYAN));
    f.render_widget(traffic_spark, spark_chunks[0]);

    let blocks_data: Vec<u64> = snap.blocks_last_60s.iter().map(|v| *v as u64).collect();
    let blocks_spark = Sparkline::default()
        .block(Block::default()
            .title(Span::styled(
                format!(" Blocks: {:.0} block/s ", snap.blocks_per_second),
                Style::default().fg(theme::RED).add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_style(theme::border_style()))
        .data(&blocks_data)
        .style(Style::default().fg(theme::RED));
    f.render_widget(blocks_spark, spark_chunks[1]);

    // Bottom: Top IPs + Top Rules + Recent Events
    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(33), Constraint::Percentage(33), Constraint::Percentage(34)])
        .split(chunks[2]);

    // Top blocked IPs
    let ip_rows: Vec<Row> = snap.top_blocked_ips.iter().take(8)
        .enumerate()
        .map(|(i, (ip, count))| {
            Row::new(vec![
                Cell::from(format!("{}.", i + 1)).style(theme::dim_style()),
                Cell::from(ip.as_str()).style(Style::default().fg(theme::TEXT)),
                Cell::from(format_number(*count)).style(Style::default().fg(theme::RED)),
            ])
        })
        .collect();
    let ip_table = Table::new(ip_rows, [Constraint::Length(3), Constraint::Min(15), Constraint::Length(10)])
        .block(Block::default()
            .title(Span::styled(" Top Blocked IPs ", Style::default().fg(theme::RED).add_modifier(Modifier::BOLD)))
            .borders(Borders::ALL)
            .border_style(theme::border_style()));
    f.render_widget(ip_table, bottom_chunks[0]);

    // Top triggered rules
    let rule_rows: Vec<Row> = snap.top_rules.iter().take(8)
        .enumerate()
        .map(|(i, (name, count))| {
            Row::new(vec![
                Cell::from(format!("{}.", i + 1)).style(theme::dim_style()),
                Cell::from(truncate_str(name, 18)).style(Style::default().fg(theme::YELLOW)),
                Cell::from(format_number(*count)).style(theme::stat_value()),
            ])
        })
        .collect();
    let rule_table = Table::new(rule_rows, [Constraint::Length(3), Constraint::Min(15), Constraint::Length(10)])
        .block(Block::default()
            .title(Span::styled(" Top Rules ", Style::default().fg(theme::YELLOW).add_modifier(Modifier::BOLD)))
            .borders(Borders::ALL)
            .border_style(theme::border_style()));
    f.render_widget(rule_table, bottom_chunks[1]);

    // Recent events (only fetch last 10, not entire buffer)
    let events = app.state.recent_events_tail(10);
    let event_rows: Vec<Row> = events.iter().rev()
        .map(|e| {
            let time = e.timestamp.format("%H:%M:%S").to_string();
            let verdict_str = e.verdict.to_string();
            Row::new(vec![
                Cell::from(time).style(theme::dim_style()),
                Cell::from(truncate_str(&e.source_ip, 15)).style(Style::default().fg(theme::TEXT)),
                Cell::from(e.method.as_str()).style(Style::default().fg(theme::PURPLE)),
                Cell::from(truncate_str(&e.path, 15)).style(Style::default().fg(theme::TEXT_DIM)),
                Cell::from(verdict_str.clone()).style(theme::verdict_style(&verdict_str)),
            ])
        })
        .collect();
    let event_table = Table::new(event_rows, [
        Constraint::Length(8), Constraint::Length(15), Constraint::Length(6),
        Constraint::Min(10), Constraint::Length(12),
    ])
        .block(Block::default()
            .title(Span::styled(" Recent Events ", theme::title_style()))
            .borders(Borders::ALL)
            .border_style(theme::border_style()));
    f.render_widget(event_table, bottom_chunks[2]);
}

// ── Live Feed View ──────────────────────────────────────────────────────────

fn render_live_feed(f: &mut Frame, app: &App, area: Rect) {
    let events = app.filtered_events();
    let header = Row::new(vec![
        Cell::from("TIME").style(theme::header_style()),
        Cell::from("IP").style(theme::header_style()),
        Cell::from("METHOD").style(theme::header_style()),
        Cell::from("PATH").style(theme::header_style()),
        Cell::from("VERDICT").style(theme::header_style()),
        Cell::from("RULE").style(theme::header_style()),
        Cell::from("MS").style(theme::header_style()),
    ]).height(1);

    let visible_height = area.height.saturating_sub(3) as usize;
    let rows: Vec<Row> = events.iter().rev().skip(app.scroll_offset).take(visible_height)
        .map(|e| {
            let time = e.timestamp.format("%H:%M:%S").to_string();
            let verdict_str = e.verdict.to_string();
            Row::new(vec![
                Cell::from(time).style(theme::dim_style()),
                Cell::from(e.source_ip.as_str()).style(Style::default().fg(theme::TEXT)),
                Cell::from(e.method.as_str()).style(Style::default().fg(theme::PURPLE)),
                Cell::from(truncate_str(&e.path, 30)).style(Style::default().fg(theme::TEXT_DIM)),
                Cell::from(verdict_str.clone()).style(theme::verdict_style(&verdict_str)),
                Cell::from(truncate_str(e.blocked_by_rule.as_deref().unwrap_or("-"), 18)).style(Style::default().fg(theme::YELLOW)),
                Cell::from(format!("{:.1}", e.total_latency_ms)).style(theme::dim_style()),
            ])
        })
        .collect();

    let search_hint = if !app.search_input.is_empty() {
        format!(" search:'{}' ", app.search_input)
    } else {
        String::new()
    };

    let title = format!(
        " Live Feed -- {} -- {} events{} -- [p]ause [f]ilter [/]search ",
        app.feed_filter.label(),
        events.len(),
        search_hint,
    );

    let table = Table::new(rows, [
        Constraint::Length(8), Constraint::Length(16), Constraint::Length(7),
        Constraint::Min(20), Constraint::Length(12), Constraint::Length(20), Constraint::Length(8),
    ])
        .header(header)
        .block(Block::default()
            .title(Span::styled(title, theme::title_style()))
            .borders(Borders::ALL)
            .border_style(theme::border_style()));

    f.render_widget(table, area);

    // Search input overlay
    if app.search_active {
        render_search_bar(f, &app.search_input, area);
    }
}

// ── Blocked View ────────────────────────────────────────────────────────────

fn render_blocked(f: &mut Frame, app: &App, area: Rect) {
    let events: Vec<_> = app.recent_events().into_iter()
        .filter(|e| matches!(e.verdict, Verdict::Block | Verdict::RateLimited))
        .collect();

    let header = Row::new(vec![
        Cell::from("TIME").style(theme::header_style()),
        Cell::from("IP").style(theme::header_style()),
        Cell::from("METHOD").style(theme::header_style()),
        Cell::from("PATH").style(theme::header_style()),
        Cell::from("VERDICT").style(theme::header_style()),
        Cell::from("STAGE").style(theme::header_style()),
        Cell::from("RULE").style(theme::header_style()),
        Cell::from("REASON").style(theme::header_style()),
    ]).height(1);

    let visible_height = area.height.saturating_sub(3) as usize;
    let rows: Vec<Row> = events.iter().rev().skip(app.scroll_offset).take(visible_height)
        .map(|e| {
            let time = e.timestamp.format("%H:%M:%S").to_string();
            let verdict_str = e.verdict.to_string();
            Row::new(vec![
                Cell::from(time).style(theme::dim_style()),
                Cell::from(e.source_ip.as_str()).style(Style::default().fg(theme::TEXT)),
                Cell::from(e.method.as_str()).style(Style::default().fg(theme::PURPLE)),
                Cell::from(truncate_str(&e.path, 20)).style(Style::default().fg(theme::TEXT_DIM)),
                Cell::from(verdict_str.clone()).style(theme::verdict_style(&verdict_str)),
                Cell::from(e.blocked_by_stage.as_deref().unwrap_or("-")).style(Style::default().fg(theme::ORANGE)),
                Cell::from(truncate_str(e.blocked_by_rule.as_deref().unwrap_or("-"), 16)).style(Style::default().fg(theme::YELLOW)),
                Cell::from(truncate_str(e.blocked_reason.as_deref().unwrap_or("-"), 30)).style(Style::default().fg(theme::TEXT_DIM)),
            ])
        })
        .collect();

    let table = Table::new(rows, [
        Constraint::Length(8), Constraint::Length(16), Constraint::Length(7),
        Constraint::Length(20), Constraint::Length(12), Constraint::Length(12),
        Constraint::Length(18), Constraint::Min(10),
    ])
        .header(header)
        .block(Block::default()
            .title(Span::styled(
                format!(" Blocked Requests ({}) ", events.len()),
                Style::default().fg(theme::RED).add_modifier(Modifier::BOLD)))
            .borders(Borders::ALL)
            .border_style(theme::border_style()));

    f.render_widget(table, area);
}

// ── IP Inspector View ───────────────────────────────────────────────────────

fn render_ip_inspector(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Search bar
            Constraint::Min(0),    // Content
        ])
        .split(area);

    // Search bar
    let search_text = if app.ip_search_active {
        format!("  Search IP: {}|", app.ip_search)
    } else if app.ip_search.is_empty() {
        "  Press / to search an IP address".to_string()
    } else {
        format!("  IP: {}  [b]lock [u]nblock [/]new search", app.ip_search)
    };
    let search_p = Paragraph::new(search_text)
        .style(if app.ip_search_active {
            Style::default().fg(theme::CYAN)
        } else {
            Style::default().fg(theme::TEXT_DIM)
        })
        .block(Block::default().borders(Borders::ALL).border_style(theme::border_style())
            .title(Span::styled(" IP Inspector ", theme::title_style())));
    f.render_widget(search_p, chunks[0]);

    if app.ip_search.is_empty() {
        // Show top IPs
        let snap = app.stats_snapshot();
        let rows: Vec<Row> = snap.top_ips.iter().take(20)
            .map(|(ip, count)| {
                let blocked = snap.top_blocked_ips.iter().find(|(i, _)| i == ip).map(|(_, c)| *c).unwrap_or(0);
                let block_pct = if *count > 0 { blocked as f64 / *count as f64 * 100.0 } else { 0.0 };
                let pct_color = if block_pct > 50.0 { theme::RED } else if block_pct > 20.0 { theme::YELLOW } else { theme::GREEN };
                Row::new(vec![
                    Cell::from(ip.as_str()).style(Style::default().fg(theme::TEXT)),
                    Cell::from(format_number(*count)).style(theme::stat_value()),
                    Cell::from(format_number(blocked)).style(Style::default().fg(theme::RED)),
                    Cell::from(format!("{:.1}%", block_pct)).style(Style::default().fg(pct_color)),
                ])
            })
            .collect();

        let header = Row::new(vec![
            Cell::from("IP").style(theme::header_style()),
            Cell::from("REQUESTS").style(theme::header_style()),
            Cell::from("BLOCKED").style(theme::header_style()),
            Cell::from("BLOCK %").style(theme::header_style()),
        ]);

        let table = Table::new(rows, [
            Constraint::Min(20), Constraint::Length(12), Constraint::Length(12), Constraint::Length(10),
        ])
            .header(header)
            .block(Block::default()
                .title(Span::styled(" All IPs (by request count) ", theme::title_style()))
                .borders(Borders::ALL)
                .border_style(theme::border_style()));
        f.render_widget(table, chunks[1]);
    } else {
        // Show details for specific IP
        let events = app.events_for_ip(&app.ip_search);
        let total = events.len();
        let blocked = events.iter().filter(|e| matches!(e.verdict, Verdict::Block | Verdict::RateLimited)).count();
        let first_seen = events.first().map(|e| e.timestamp.format("%Y-%m-%d %H:%M:%S").to_string()).unwrap_or_else(|| "N/A".to_string());
        let last_seen = events.last().map(|e| e.timestamp.format("%Y-%m-%d %H:%M:%S").to_string()).unwrap_or_else(|| "N/A".to_string());

        let info_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(8), Constraint::Min(0)])
            .split(chunks[1]);

        // Collect top paths for this IP
        let mut path_counts: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
        let mut rule_counts: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
        for e in &events {
            *path_counts.entry(e.path.clone()).or_insert(0) += 1;
            if let Some(ref r) = e.blocked_by_rule {
                *rule_counts.entry(r.clone()).or_insert(0) += 1;
            }
        }

        let info = vec![
            Line::from(vec![
                Span::styled("  IP: ", theme::stat_label()),
                Span::styled(&app.ip_search, theme::stat_value()),
                Span::styled("    First Seen: ", theme::stat_label()),
                Span::styled(&first_seen, theme::dim_style()),
            ]),
            Line::from(vec![
                Span::styled("  Total: ", theme::stat_label()),
                Span::styled(format_number(total as u64), theme::stat_value()),
                Span::styled("    Last Seen: ", theme::stat_label()),
                Span::styled(&last_seen, theme::dim_style()),
            ]),
            Line::from(vec![
                Span::styled("  Blocked: ", theme::stat_label()),
                Span::styled(format_number(blocked as u64), Style::default().fg(theme::RED).add_modifier(Modifier::BOLD)),
                Span::styled(format!("  ({:.1}%)", if total > 0 { blocked as f64 / total as f64 * 100.0 } else { 0.0 }), theme::dim_style()),
            ]),
            Line::from(vec![
                Span::styled("  Top paths: ", theme::stat_label()),
                Span::styled({
                    let mut paths: Vec<_> = path_counts.iter().collect();
                    paths.sort_by(|a, b| b.1.cmp(a.1));
                    paths.iter().take(3).map(|(p, c)| format!("{} ({})", truncate_str(p, 20), c)).collect::<Vec<_>>().join(", ")
                }, theme::dim_style()),
            ]),
        ];
        let info_p = Paragraph::new(info)
            .block(Block::default().borders(Borders::ALL).border_style(theme::border_style())
                .title(Span::styled(" Summary ", theme::title_style())));
        f.render_widget(info_p, info_chunks[0]);

        // Event history
        let visible_height = info_chunks[1].height.saturating_sub(2) as usize;
        let rows: Vec<Row> = events.iter().rev().skip(app.scroll_offset).take(visible_height)
            .map(|e| {
                let time = e.timestamp.format("%H:%M:%S").to_string();
                let verdict_str = e.verdict.to_string();
                Row::new(vec![
                    Cell::from(time).style(theme::dim_style()),
                    Cell::from(e.method.as_str()).style(Style::default().fg(theme::PURPLE)),
                    Cell::from(truncate_str(&e.path, 25)).style(Style::default().fg(theme::TEXT_DIM)),
                    Cell::from(verdict_str.clone()).style(theme::verdict_style(&verdict_str)),
                    Cell::from(truncate_str(e.blocked_by_rule.as_deref().unwrap_or("-"), 18)).style(Style::default().fg(theme::YELLOW)),
                    Cell::from(format!("{:.1}ms", e.total_latency_ms)).style(theme::dim_style()),
                ])
            })
            .collect();

        let table = Table::new(rows, [
            Constraint::Length(8), Constraint::Length(7), Constraint::Min(15),
            Constraint::Length(12), Constraint::Length(18), Constraint::Length(10),
        ])
            .block(Block::default()
                .title(Span::styled(" Activity History ", theme::title_style()))
                .borders(Borders::ALL)
                .border_style(theme::border_style()));
        f.render_widget(table, info_chunks[1]);
    }
}

// ── Rules View ──────────────────────────────────────────────────────────────

fn render_rules(f: &mut Frame, app: &App, area: Rect) {
    let rules = app.state.pipeline.read()
        .map(|p| p.rule_stats())
        .unwrap_or_default();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(10)])
        .split(area);

    let header = Row::new(vec![
        Cell::from("NAME").style(theme::header_style()),
        Cell::from("DESCRIPTION").style(theme::header_style()),
        Cell::from("STATUS").style(theme::header_style()),
        Cell::from("PRI").style(theme::header_style()),
        Cell::from("HITS").style(theme::header_style()),
    ]).height(1);

    let rows: Vec<Row> = rules.iter().enumerate()
        .map(|(i, (name, desc, enabled, hits, priority))| {
            let style = if i == app.selected_index {
                theme::selected_style()
            } else {
                Style::default()
            };
            Row::new(vec![
                Cell::from(truncate_str(name, 22)).style(Style::default().fg(theme::CYAN)),
                Cell::from(truncate_str(desc, 30)).style(Style::default().fg(theme::TEXT_DIM)),
                Cell::from(if *enabled { "ON" } else { "OFF" }).style(if *enabled {
                    Style::default().fg(theme::GREEN)
                } else {
                    Style::default().fg(theme::RED)
                }),
                Cell::from(priority.to_string()).style(theme::dim_style()),
                Cell::from(format_number(*hits)).style(theme::stat_value()),
            ]).style(style)
        })
        .collect();

    let table = Table::new(rows, [
        Constraint::Length(24), Constraint::Min(20), Constraint::Length(8),
        Constraint::Length(6), Constraint::Length(10),
    ])
        .header(header)
        .block(Block::default()
            .title(Span::styled(
                format!(" Custom Rules ({}) ", rules.len()),
                theme::title_style(),
            ))
            .borders(Borders::ALL)
            .border_style(theme::border_style()));
    f.render_widget(table, chunks[0]);

    // WAF built-in status
    let waf_status = if let Ok(cfg) = app.state.config.read() {
        vec![
            Line::from(vec![
                Span::styled("  Built-in WAF Protections:", Style::default().fg(theme::YELLOW).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("  SQLi: ", theme::stat_label()),
                status_badge(cfg.waf.sqli_protection),
                Span::styled("  XSS: ", theme::stat_label()),
                status_badge(cfg.waf.xss_protection),
                Span::styled("  Path Traversal: ", theme::stat_label()),
                status_badge(cfg.waf.path_traversal_protection),
            ]),
            Line::from(vec![
                Span::styled("  Cmd Injection: ", theme::stat_label()),
                status_badge(cfg.waf.command_injection_protection),
                Span::styled("  Header Injection: ", theme::stat_label()),
                status_badge(cfg.waf.header_injection_protection),
                Span::styled("  Empty UA Block: ", theme::stat_label()),
                status_badge(cfg.waf.block_empty_user_agent),
            ]),
            Line::from(vec![
                Span::styled("  Max URL: ", theme::stat_label()),
                Span::styled(cfg.waf.max_url_length.to_string(), theme::stat_value()),
                Span::styled("  Max Body: ", theme::stat_label()),
                Span::styled(format_bytes_short(cfg.waf.max_body_size as u64), theme::stat_value()),
                Span::styled("  Blocked paths: ", theme::stat_label()),
                Span::styled(cfg.waf.blocked_paths.len().to_string(), theme::stat_value()),
                Span::styled("  Blocked UAs: ", theme::stat_label()),
                Span::styled(cfg.waf.blocked_user_agents.len().to_string(), theme::stat_value()),
            ]),
        ]
    } else {
        vec![Line::from(Span::styled("  Unable to read config", theme::dim_style()))]
    };

    let waf_p = Paragraph::new(waf_status)
        .block(Block::default()
            .title(Span::styled(" WAF Engine ", theme::title_style()))
            .borders(Borders::ALL)
            .border_style(theme::border_style()));
    f.render_widget(waf_p, chunks[1]);
}

// ── Stats View ──────────────────────────────────────────────────────────────

fn render_stats(f: &mut Frame, app: &App, area: Rect) {
    let snap = app.stats_snapshot();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),  // Summary
            Constraint::Min(0),    // Tables
        ])
        .split(area);

    // Summary stats
    let block_rate = if snap.total_requests > 0 {
        snap.total_blocked as f64 / snap.total_requests as f64 * 100.0
    } else {
        0.0
    };

    let summary = vec![
        Line::from(vec![
            Span::styled("  Uptime:          ", theme::stat_label()),
            Span::styled(snap.format_uptime(), theme::stat_value()),
        ]),
        Line::from(vec![
            Span::styled("  Total Requests:  ", theme::stat_label()),
            Span::styled(format_number(snap.total_requests), theme::stat_value()),
            Span::styled("    Blocked:  ", theme::stat_label()),
            Span::styled(format_number(snap.total_blocked), Style::default().fg(theme::RED).add_modifier(Modifier::BOLD)),
            Span::styled(format!("  ({:.1}%)", block_rate), theme::dim_style()),
        ]),
        Line::from(vec![
            Span::styled("  Rate Limited:    ", theme::stat_label()),
            Span::styled(format_number(snap.total_rate_limited), Style::default().fg(theme::ORANGE)),
            Span::styled("    Flagged:  ", theme::stat_label()),
            Span::styled(format_number(snap.total_flagged), Style::default().fg(theme::YELLOW)),
        ]),
        Line::from(vec![
            Span::styled("  Avg req/s:       ", theme::stat_label()),
            Span::styled(format!("{:.1}", snap.requests_per_second), theme::stat_value()),
            Span::styled("    Avg block/s:  ", theme::stat_label()),
            Span::styled(format!("{:.1}", snap.blocks_per_second), Style::default().fg(theme::RED)),
        ]),
        Line::from(vec![
            Span::styled("  Active Conns:    ", theme::stat_label()),
            Span::styled(snap.active_connections.to_string(), theme::stat_value()),
        ]),
    ];
    let summary_p = Paragraph::new(summary)
        .block(Block::default()
            .title(Span::styled(" Statistics Summary ", theme::title_style()))
            .borders(Borders::ALL)
            .border_style(theme::border_style()));
    f.render_widget(summary_p, chunks[0]);

    // Tables: Top paths + Verdict + Status codes
    let table_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(30), Constraint::Percentage(30)])
        .split(chunks[1]);

    // Top paths
    let path_rows: Vec<Row> = snap.top_paths.iter().take(15)
        .enumerate()
        .map(|(i, (path, count))| {
            Row::new(vec![
                Cell::from(format!("{}.", i + 1)).style(theme::dim_style()),
                Cell::from(truncate_str(path, 30)).style(Style::default().fg(theme::TEXT)),
                Cell::from(format_number(*count)).style(theme::stat_value()),
            ])
        })
        .collect();
    let path_table = Table::new(path_rows, [Constraint::Length(4), Constraint::Min(15), Constraint::Length(10)])
        .block(Block::default()
            .title(Span::styled(" Top Paths ", theme::title_style()))
            .borders(Borders::ALL)
            .border_style(theme::border_style()));
    f.render_widget(path_table, table_chunks[0]);

    // Verdict distribution
    let mut verdict_rows: Vec<Row> = Vec::new();
    for (verdict, count) in &snap.verdict_counts {
        verdict_rows.push(Row::new(vec![
            Cell::from(verdict.as_str()).style(theme::verdict_style(verdict)),
            Cell::from(format_number(*count)).style(theme::stat_value()),
            Cell::from({
                let pct = if snap.total_requests > 0 { *count as f64 / snap.total_requests as f64 * 100.0 } else { 0.0 };
                format!("{:.1}%", pct)
            }).style(theme::dim_style()),
        ]));
    }
    let verdict_table = Table::new(verdict_rows, [Constraint::Length(12), Constraint::Length(10), Constraint::Min(8)])
        .block(Block::default()
            .title(Span::styled(" Verdicts ", theme::title_style()))
            .borders(Borders::ALL)
            .border_style(theme::border_style()));
    f.render_widget(verdict_table, table_chunks[1]);

    // Status code distribution
    let mut status_entries: Vec<_> = snap.status_codes.iter().collect();
    status_entries.sort_by_key(|(code, _)| *code);
    let status_rows: Vec<Row> = status_entries.iter()
        .map(|(code, count)| {
            let color = match code {
                200..=299 => theme::GREEN,
                300..=399 => theme::CYAN,
                400..=499 => theme::YELLOW,
                500..=599 => theme::RED,
                _ => theme::DIM,
            };
            Row::new(vec![
                Cell::from(code.to_string()).style(Style::default().fg(color)),
                Cell::from(format_number(**count)).style(theme::stat_value()),
            ])
        })
        .collect();

    let status_table = Table::new(status_rows, [Constraint::Length(8), Constraint::Min(10)])
        .block(Block::default()
            .title(Span::styled(" Status Codes ", theme::title_style()))
            .borders(Borders::ALL)
            .border_style(theme::border_style()));
    f.render_widget(status_table, table_chunks[2]);
}

// ── Config View ─────────────────────────────────────────────────────────────

fn render_config(f: &mut Frame, app: &App, area: Rect) {
    let fields = app.config_fields();

    // Group fields into sections with headers
    let section_breaks: &[(usize, &str)] = &[
        (0, "SERVER"),
        (4, "SECURITY"),
        (8, "LIMITS"),
        (11, "PROTECTIONS"),
        (17, "AUTO-REPUTATION"),
    ];

    let mut rows: Vec<Row> = Vec::new();

    for (i, (label, value)) in fields.iter().enumerate() {
        // Insert section header if needed
        for &(break_at, section_name) in section_breaks {
            if i == break_at {
                rows.push(Row::new(vec![
                    Cell::from(format!("  {}", section_name))
                        .style(Style::default().fg(theme::CYAN).add_modifier(Modifier::BOLD)),
                    Cell::from("").style(theme::dim_style()),
                ]).height(1));
            }
        }

        let is_selected = i == app.config_selected;
        let pointer = if is_selected { " >" } else { "  " };
        let label_style = if is_selected {
            Style::default().fg(theme::CYAN).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::TEXT)
        };
        let value_style = if is_selected && app.config_editing {
            Style::default().fg(theme::CYAN).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else if is_selected {
            Style::default().fg(theme::WHITE).add_modifier(Modifier::BOLD)
        } else if value == "true" {
            Style::default().fg(theme::GREEN)
        } else if value == "false" {
            Style::default().fg(theme::RED)
        } else {
            theme::stat_value()
        };

        let display_value = if is_selected && app.config_editing {
            format!("{}|", app.config_edit_buf)
        } else {
            value.clone()
        };

        rows.push(Row::new(vec![
            Cell::from(format!("{} {}", pointer, label)).style(label_style),
            Cell::from(display_value).style(value_style),
        ]));
    }

    let table = Table::new(rows, [Constraint::Length(28), Constraint::Min(20)])
        .block(Block::default()
            .title(Span::styled(
                " Configuration ",
                theme::title_style(),
            ))
            .borders(Borders::ALL)
            .border_style(theme::border_style()));

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(4)])
        .split(area);

    f.render_widget(table, chunks[0]);

    // Context-sensitive help at bottom
    let config_path = app.state.config_path.as_deref().unwrap_or("infynon.toml");
    let file_info = format!("  file: {}", config_path);
    let help_text = if app.config_editing {
        vec![
            Line::from(vec![
                Span::styled("  Editing: ", theme::stat_label()),
                Span::styled("Type new value, Enter to apply, Esc to cancel", Style::default().fg(theme::CYAN)),
            ]),
        ]
    } else {
        vec![
            Line::from(vec![
                Span::styled("  Enter", theme::stat_value()),
                Span::styled(" edit  ", theme::dim_style()),
                Span::styled("s", theme::stat_value()),
                Span::styled(" save  ", theme::dim_style()),
                Span::styled("r", theme::stat_value()),
                Span::styled(" reload  ", theme::dim_style()),
                Span::styled("Up/Dn", theme::stat_value()),
                Span::styled(" navigate  ", theme::dim_style()),
                Span::styled(&file_info, theme::dim_style()),
            ]),
            Line::from(Span::styled(
                "  Server changes require restart. WAF/rate-limit changes apply immediately.",
                theme::dim_style(),
            )),
        ]
    };
    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::TOP).border_style(theme::border_style()));
    f.render_widget(help, chunks[1]);
}

// ── Search bar overlay ──────────────────────────────────────────────────────

fn render_search_bar(f: &mut Frame, input: &str, area: Rect) {
    let bar_area = Rect {
        x: area.x + 1,
        y: area.y + area.height.saturating_sub(3),
        width: area.width.saturating_sub(2),
        height: 3,
    };
    f.render_widget(Clear, bar_area);
    let search_p = Paragraph::new(format!(" Search: {}|", input))
        .style(Style::default().fg(theme::CYAN))
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::CYAN))
            .title(Span::styled(" Filter ", Style::default().fg(theme::CYAN).add_modifier(Modifier::BOLD))));
    f.render_widget(search_p, bar_area);
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn status_badge(enabled: bool) -> Span<'static> {
    if enabled {
        Span::styled("ON", Style::default().fg(theme::GREEN).add_modifier(Modifier::BOLD))
    } else {
        Span::styled("OFF", Style::default().fg(theme::RED).add_modifier(Modifier::BOLD))
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
