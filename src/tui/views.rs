use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Cell, Paragraph, Row, Sparkline, Table, Tabs, Wrap,
};
use ratatui::Frame;

use crate::firewall::events::Verdict;
use crate::tui::firewall_app::{App, FeedFilter, View};
use crate::tui::theme;

// ── Main render dispatcher ──────────────────────────────────────────────────

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Tab bar
            Constraint::Min(0),    // Content
            Constraint::Length(1), // Status line
        ])
        .split(f.size());

    render_tabs(f, app, chunks[0]);

    match app.current_view {
        View::Dashboard => render_dashboard(f, app, chunks[1]),
        View::LiveFeed => render_live_feed(f, app, chunks[1]),
        View::Blocked => render_blocked(f, app, chunks[1]),
        View::IpInspector => render_ip_inspector(f, app, chunks[1]),
        View::Rules => render_rules(f, app, chunks[1]),
        View::Stats => render_stats(f, app, chunks[1]),
        View::Config => render_config(f, app, chunks[1]),
    }

    render_status_line(f, app, chunks[2]);
}

// ── Tab bar ─────────────────────────────────────────────────────────────────

fn render_tabs(f: &mut Frame, app: &App, area: Rect) {
    let titles: Vec<Line> = View::all()
        .iter()
        .map(|v| {
            let style = if *v == app.current_view {
                Style::default().fg(theme::CYAN).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme::DIM)
            };
            Line::from(Span::styled(format!("[{}]{}", v.key(), v.label()), style))
        })
        .collect();

    let tabs = Tabs::new(titles)
        .block(Block::default()
            .borders(Borders::BOTTOM)
            .border_style(theme::border_style())
            .title(Span::styled(" INFYNON FIREWALL ", theme::title_style())))
        .highlight_style(Style::default().fg(theme::CYAN))
        .divider(Span::styled(" | ", theme::dim_style()));

    f.render_widget(tabs, area);
}

// ── Status line ─────────────────────────────────────────────────────────────

fn render_status_line(f: &mut Frame, app: &App, area: Rect) {
    let snap = app.stats_snapshot();
    let status = format!(
        " {} | {:.0} req/s | {:.0} block/s | {} conn | q:quit /:search {}",
        app.current_view.label(),
        snap.requests_per_second,
        snap.blocks_per_second,
        snap.active_connections,
        if app.paused { "| PAUSED" } else { "" },
    );
    let p = Paragraph::new(status).style(Style::default().fg(theme::TEXT_DIM).bg(theme::BG_HIGHLIGHT));
    f.render_widget(p, area);
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
    let status_text = vec![
        Span::styled("  Status: ", theme::stat_label()),
        Span::styled("● RUNNING", theme::status_running()),
        Span::styled("    Uptime: ", theme::stat_label()),
        Span::styled(snap.format_uptime(), theme::stat_value()),
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
                Cell::from(name.as_str()).style(Style::default().fg(theme::YELLOW)),
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

    // Recent events
    let events = app.recent_events();
    let event_rows: Vec<Row> = events.iter().rev().take(10)
        .map(|e| {
            let time = e.timestamp.format("%H:%M:%S").to_string();
            Row::new(vec![
                Cell::from(time).style(theme::dim_style()),
                Cell::from(e.source_ip.as_str()).style(Style::default().fg(theme::TEXT)),
                Cell::from(e.method.as_str()).style(Style::default().fg(theme::PURPLE)),
                Cell::from(truncate(&e.path, 15)).style(Style::default().fg(theme::TEXT_DIM)),
                Cell::from(e.verdict.to_string()).style(theme::verdict_style(&e.verdict.to_string())),
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

    let rows: Vec<Row> = events.iter().rev().skip(app.scroll_offset).take(area.height as usize - 3)
        .map(|e| {
            let time = e.timestamp.format("%H:%M:%S").to_string();
            let verdict_str = e.verdict.to_string();
            Row::new(vec![
                Cell::from(time).style(theme::dim_style()),
                Cell::from(e.source_ip.as_str()).style(Style::default().fg(theme::TEXT)),
                Cell::from(e.method.as_str()).style(Style::default().fg(theme::PURPLE)),
                Cell::from(truncate(&e.path, 30)).style(Style::default().fg(theme::TEXT_DIM)),
                Cell::from(verdict_str.clone()).style(theme::verdict_style(&verdict_str)),
                Cell::from(e.blocked_by_rule.as_deref().unwrap_or("-")).style(Style::default().fg(theme::YELLOW)),
                Cell::from(format!("{:.1}", e.total_latency_ms)).style(theme::dim_style()),
            ])
        })
        .collect();

    let title = format!(
        " Live Feed — Filter: {} — [p]ause [f]ilter [/]search ",
        app.feed_filter.label()
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

    let rows: Vec<Row> = events.iter().rev().skip(app.scroll_offset).take(area.height as usize - 3)
        .map(|e| {
            let time = e.timestamp.format("%H:%M:%S").to_string();
            let verdict_str = e.verdict.to_string();
            Row::new(vec![
                Cell::from(time).style(theme::dim_style()),
                Cell::from(e.source_ip.as_str()).style(Style::default().fg(theme::TEXT)),
                Cell::from(e.method.as_str()).style(Style::default().fg(theme::PURPLE)),
                Cell::from(truncate(&e.path, 20)).style(Style::default().fg(theme::TEXT_DIM)),
                Cell::from(verdict_str.clone()).style(theme::verdict_style(&verdict_str)),
                Cell::from(e.blocked_by_stage.as_deref().unwrap_or("-")).style(Style::default().fg(theme::ORANGE)),
                Cell::from(e.blocked_by_rule.as_deref().unwrap_or("-")).style(Style::default().fg(theme::YELLOW)),
                Cell::from(truncate(e.blocked_reason.as_deref().unwrap_or("-"), 30)).style(Style::default().fg(theme::TEXT_DIM)),
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
            .title(Span::styled(" Blocked Requests ", Style::default().fg(theme::RED).add_modifier(Modifier::BOLD)))
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
        format!("  IP: {}  [b]lock [u]nblock [/]search", app.ip_search)
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
                Row::new(vec![
                    Cell::from(ip.as_str()).style(Style::default().fg(theme::TEXT)),
                    Cell::from(format_number(*count)).style(theme::stat_value()),
                    Cell::from(format_number(blocked)).style(Style::default().fg(theme::RED)),
                    Cell::from(if blocked > 0 { format!("{:.1}%", blocked as f64 / *count as f64 * 100.0) } else { "0%".to_string() })
                        .style(theme::dim_style()),
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

        let info_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(6), Constraint::Min(0)])
            .split(chunks[1]);

        let info = vec![
            Line::from(vec![
                Span::styled("  IP: ", theme::stat_label()),
                Span::styled(&app.ip_search, theme::stat_value()),
                Span::styled("    Total: ", theme::stat_label()),
                Span::styled(format_number(total as u64), theme::stat_value()),
                Span::styled("    Blocked: ", theme::stat_label()),
                Span::styled(format_number(blocked as u64), Style::default().fg(theme::RED).add_modifier(Modifier::BOLD)),
                Span::styled(format!("  ({:.1}%)", if total > 0 { blocked as f64 / total as f64 * 100.0 } else { 0.0 }), theme::dim_style()),
            ]),
            Line::from(""),
        ];
        let info_p = Paragraph::new(info)
            .block(Block::default().borders(Borders::ALL).border_style(theme::border_style())
                .title(Span::styled(" Summary ", theme::title_style())));
        f.render_widget(info_p, info_chunks[0]);

        // Event history
        let rows: Vec<Row> = events.iter().rev().take(info_chunks[1].height as usize - 2)
            .map(|e| {
                let time = e.timestamp.format("%H:%M:%S").to_string();
                let verdict_str = e.verdict.to_string();
                Row::new(vec![
                    Cell::from(time).style(theme::dim_style()),
                    Cell::from(e.method.as_str()).style(Style::default().fg(theme::PURPLE)),
                    Cell::from(truncate(&e.path, 25)).style(Style::default().fg(theme::TEXT_DIM)),
                    Cell::from(verdict_str.clone()).style(theme::verdict_style(&verdict_str)),
                    Cell::from(e.blocked_by_rule.as_deref().unwrap_or("-")).style(Style::default().fg(theme::YELLOW)),
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
    let rules = app.state.pipeline.rule_stats();

    let header = Row::new(vec![
        Cell::from("NAME").style(theme::header_style()),
        Cell::from("DESCRIPTION").style(theme::header_style()),
        Cell::from("ENABLED").style(theme::header_style()),
        Cell::from("PRIORITY").style(theme::header_style()),
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
                Cell::from(name.as_str()).style(Style::default().fg(theme::CYAN)),
                Cell::from(truncate(desc, 30)).style(Style::default().fg(theme::TEXT_DIM)),
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

    // WAF built-in rules as extra info
    let waf_info = if rules.is_empty() {
        "\n  No custom rules defined. Built-in WAF rules are active.\n  Add rules in infynon.toml [[rules]] sections."
    } else {
        ""
    };

    let table = Table::new(rows, [
        Constraint::Length(25), Constraint::Min(20), Constraint::Length(8),
        Constraint::Length(10), Constraint::Length(10),
    ])
        .header(header)
        .block(Block::default()
            .title(Span::styled(" Rules — [e]nable/disable [Enter]details ", theme::title_style()))
            .borders(Borders::ALL)
            .border_style(theme::border_style()));

    if !waf_info.is_empty() {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        f.render_widget(table, chunks[0]);

        let info = Paragraph::new(waf_info)
            .style(theme::dim_style())
            .wrap(Wrap { trim: false })
            .block(Block::default()
                .title(Span::styled(" Built-in WAF Protections ", theme::title_style()))
                .borders(Borders::ALL)
                .border_style(theme::border_style()));
        f.render_widget(info, chunks[1]);
    } else {
        f.render_widget(table, area);
    }
}

// ── Stats View ──────────────────────────────────────────────────────────────

fn render_stats(f: &mut Frame, app: &App, area: Rect) {
    let snap = app.stats_snapshot();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),  // Summary
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
    ];
    let summary_p = Paragraph::new(summary)
        .block(Block::default()
            .title(Span::styled(" Statistics Summary ", theme::title_style()))
            .borders(Borders::ALL)
            .border_style(theme::border_style()));
    f.render_widget(summary_p, chunks[0]);

    // Tables: Top paths + Status codes
    let table_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(chunks[1]);

    // Top paths
    let path_rows: Vec<Row> = snap.top_paths.iter().take(15)
        .enumerate()
        .map(|(i, (path, count))| {
            Row::new(vec![
                Cell::from(format!("{}.", i + 1)).style(theme::dim_style()),
                Cell::from(truncate(path, 40)).style(Style::default().fg(theme::TEXT)),
                Cell::from(format_number(*count)).style(theme::stat_value()),
            ])
        })
        .collect();
    let path_table = Table::new(path_rows, [Constraint::Length(4), Constraint::Min(20), Constraint::Length(12)])
        .block(Block::default()
            .title(Span::styled(" Top Paths ", theme::title_style()))
            .borders(Borders::ALL)
            .border_style(theme::border_style()));
    f.render_widget(path_table, table_chunks[0]);

    // Status code + verdict distribution
    let status_rows: Vec<Row> = snap.status_codes.iter()
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
                Cell::from(format_number(*count)).style(theme::stat_value()),
            ])
        })
        .collect();
    // Status rows are already in HashMap arbitrary order — fine for display

    let status_table = Table::new(status_rows, [Constraint::Length(8), Constraint::Min(10)])
        .block(Block::default()
            .title(Span::styled(" Status Codes ", theme::title_style()))
            .borders(Borders::ALL)
            .border_style(theme::border_style()));
    f.render_widget(status_table, table_chunks[1]);
}

// ── Config View ─────────────────────────────────────────────────────────────

fn render_config(f: &mut Frame, app: &App, area: Rect) {
    let fields = app.config_fields();

    let header = Row::new(vec![
        Cell::from("SETTING").style(theme::header_style()),
        Cell::from("VALUE").style(theme::header_style()),
    ]).height(1);

    let rows: Vec<Row> = fields.iter().enumerate()
        .map(|(i, (label, value))| {
            let is_selected = i == app.config_selected;
            let label_style = if is_selected { theme::selected_style() } else { Style::default().fg(theme::TEXT) };
            let value_style = if is_selected && app.config_editing {
                Style::default().fg(theme::CYAN).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
            } else if is_selected {
                theme::selected_style()
            } else {
                theme::stat_value()
            };

            let display_value = if is_selected && app.config_editing {
                format!("{}|", app.config_edit_buf)
            } else {
                value.clone()
            };

            Row::new(vec![
                Cell::from(*label).style(label_style),
                Cell::from(display_value).style(value_style),
            ])
        })
        .collect();

    let table = Table::new(rows, [Constraint::Length(25), Constraint::Min(20)])
        .header(header)
        .block(Block::default()
            .title(Span::styled(
                " Config — [Enter]edit [s]ave to file [Up/Down]navigate ",
                theme::title_style(),
            ))
            .borders(Borders::ALL)
            .border_style(theme::border_style()));

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(4)])
        .split(area);

    f.render_widget(table, chunks[0]);

    // Help text
    let help = Paragraph::new(vec![
        Line::from(Span::styled(
            "  Config file: infynon.toml (or ~/.infynon/infynon.toml). Changes saved here also apply on file edit.",
            theme::dim_style(),
        )),
        Line::from(Span::styled(
            "  Edit the file directly or use this TUI. Restart required for server settings.",
            theme::dim_style(),
        )),
    ])
        .block(Block::default().borders(Borders::TOP).border_style(theme::border_style()));
    f.render_widget(help, chunks[1]);
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}...", &s[..max.saturating_sub(3)])
    } else {
        s.to_string()
    }
}

fn format_number(n: u64) -> String {
    if n >= 1_000_000 { format!("{:.1}M", n as f64 / 1_000_000.0) }
    else if n >= 1_000 { format!("{:.1}K", n as f64 / 1_000.0) }
    else { n.to_string() }
}
