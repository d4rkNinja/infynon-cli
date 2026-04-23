use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::api::types::FlowRunResult;
use crate::tui::api_app::ApiApp;
use crate::tui::theme::*;

use super::{dashboard::render_no_flows_hint, truncate};

// ── Environment context view ──────────────────────────────────────────────────

pub(super) fn render_env_context(f: &mut Frame, app: &mut ApiApp, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    render_env_panel(f, app, chunks[0]);
    render_flow_context_panel(f, app, chunks[1]);
}

fn render_env_panel(f: &mut Frame, app: &mut ApiApp, area: Rect) {
    use crate::api::commands::env as env_cmd;

    let entries = env_cmd::env_list();

    if entries.is_empty() {
        app.env_selected = 0;
    } else {
        app.env_selected = app.env_selected.min(entries.len() - 1);
    }

    let editing = app.env_edit.is_some();
    let bottom_height: u16 = if editing { 5 } else { 2 };

    let border = if editing {
        Style::default().fg(YELLOW)
    } else {
        border_style()
    };
    let block_title = if editing {
        " \u{2699} Environment Variables \u{2014} editing "
    } else {
        " \u{2699} Environment Variables (.infynon/.env) "
    };
    let block = Block::default()
        .title(Span::styled(block_title, title_style()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let splits = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(bottom_height),
        ])
        .split(inner);

    // Header row with KEY / VALUE columns
    let key_w = (splits[0].width as usize / 3).max(10).min(30);
    let sep_w = splits[0].width.saturating_sub(4) as usize;
    let header = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(
                format!("  {:<w$}", "KEY", w = key_w),
                Style::default().fg(DIMMER).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "VALUE",
                Style::default().fg(DIMMER).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![Span::styled(
            format!("  {}", "\u{2500}".repeat(sep_w)),
            Style::default().fg(DIM),
        )]),
    ]);
    f.render_widget(header, splits[0]);

    // List
    if entries.is_empty() {
        let hint = Paragraph::new(vec![
            Line::raw(""),
            Line::from(vec![Span::styled(
                "  No variables set.",
                Style::default().fg(DIM),
            )]),
            Line::raw(""),
            Line::from(vec![
                Span::styled("  Press ", Style::default().fg(DIMMER)),
                Span::styled("[n]", Style::default().fg(YELLOW)),
                Span::styled(" to add one.", Style::default().fg(DIMMER)),
            ]),
            Line::from(vec![Span::styled(
                "  Reference in nodes as {$KEY}",
                Style::default().fg(DIMMER),
            )]),
        ]);
        f.render_widget(hint, splits[1]);
    } else {
        let val_w = splits[1].width.saturating_sub(key_w as u16 + 6) as usize;
        let items: Vec<ListItem> = entries
            .iter()
            .enumerate()
            .map(|(i, (key, value))| {
                let selected = i == app.env_selected;
                let sensitive = env_cmd::looks_sensitive(key);
                let display = if !app.env_reveal && sensitive {
                    env_cmd::mask(value)
                } else {
                    truncate(value, val_w.max(8)).to_string()
                };
                let (key_style, val_style) = if selected {
                    (
                        Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
                        Style::default().fg(WHITE),
                    )
                } else {
                    (Style::default().fg(CYAN), Style::default().fg(TEXT_DIM))
                };
                let tag = if sensitive && !app.env_reveal {
                    Span::styled(" [hidden]", Style::default().fg(DIMMER))
                } else {
                    Span::raw("")
                };
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("  {:<w$}", truncate(key, key_w), w = key_w),
                        key_style,
                    ),
                    Span::styled(display, val_style),
                    tag,
                ]))
            })
            .collect();

        let mut list_state = ListState::default();
        list_state.select(Some(app.env_selected));

        let list = List::new(items).highlight_style(Style::default().bg(BG_HIGHLIGHT));

        f.render_stateful_widget(list, splits[1], &mut list_state);
    }

    // Editor section or keyboard hints
    if let Some(edit) = &app.env_edit {
        let key_cursor = if edit.editing_key { "\u{258C}" } else { "" };
        let val_cursor = if !edit.editing_key { "\u{258C}" } else { "" };
        let key_label_style = if edit.editing_key {
            Style::default().fg(YELLOW)
        } else {
            Style::default().fg(TEXT_DIM)
        };
        let val_label_style = if !edit.editing_key {
            Style::default().fg(YELLOW)
        } else {
            Style::default().fg(TEXT_DIM)
        };
        let field_hint = if edit.is_new() {
            "  [Enter] Next/Save  [Tab] Switch field  [Esc] Cancel"
        } else {
            "  [Enter] Save  [Esc] Cancel"
        };
        let editor = Paragraph::new(vec![
            Line::from(vec![Span::styled(
                format!("  {}", "\u{2500}".repeat(sep_w)),
                Style::default().fg(DIM),
            )]),
            Line::from(vec![
                Span::styled("  Key:   ", Style::default().fg(DIMMER)),
                Span::styled(format!("{}{}", edit.key_input, key_cursor), key_label_style),
            ]),
            Line::from(vec![
                Span::styled("  Value: ", Style::default().fg(DIMMER)),
                Span::styled(
                    format!("{}{}", edit.value_input, val_cursor),
                    val_label_style,
                ),
            ]),
            Line::from(vec![Span::styled(field_hint, Style::default().fg(DIMMER))]),
        ]);
        f.render_widget(editor, splits[2]);
    } else {
        let hints = Paragraph::new(vec![Line::from(vec![
            Span::styled("  [n]", Style::default().fg(YELLOW)),
            Span::styled(" Add  ", Style::default().fg(DIMMER)),
            Span::styled("[Enter]", Style::default().fg(YELLOW)),
            Span::styled(" Edit  ", Style::default().fg(DIMMER)),
            Span::styled("[d]", Style::default().fg(YELLOW)),
            Span::styled(" Delete  ", Style::default().fg(DIMMER)),
            Span::styled("[v]", Style::default().fg(YELLOW)),
            Span::styled(" Reveal  ", Style::default().fg(DIMMER)),
            Span::styled("\u{2191}\u{2193}/jk", Style::default().fg(YELLOW)),
            Span::styled(" Navigate", Style::default().fg(DIMMER)),
        ])]);
        f.render_widget(hints, splits[2]);
    }
}

fn render_flow_context_panel(f: &mut Frame, app: &ApiApp, area: Rect) {
    let mut lines: Vec<Line> = vec![Line::raw("")];
    let panel_w = area.width.saturating_sub(4) as usize;

    match &app.last_run {
        None => {
            lines.push(Line::from(vec![Span::styled(
                "  No flow run yet.",
                Style::default().fg(DIM),
            )]));
            lines.push(Line::raw(""));
            lines.push(Line::from(vec![Span::styled(
                "  Run a flow to see",
                Style::default().fg(DIMMER),
            )]));
            lines.push(Line::from(vec![Span::styled(
                "  extracted context here.",
                Style::default().fg(DIMMER),
            )]));
            lines.push(Line::raw(""));
            lines.push(Line::from(vec![Span::styled(
                "  Tip: seed initial vars with",
                Style::default().fg(DIMMER),
            )]));
            lines.push(Line::from(vec![Span::styled(
                "  --set KEY=VALUE at run time.",
                Style::default().fg(DIMMER),
            )]));
        }
        Some(run) if run.final_context.is_empty() => {
            lines.push(Line::from(vec![Span::styled(
                "  No context captured.",
                Style::default().fg(DIM),
            )]));
            lines.push(Line::raw(""));
            lines.push(Line::from(vec![Span::styled(
                "  Nodes must declare capture",
                Style::default().fg(DIMMER),
            )]));
            lines.push(Line::from(vec![Span::styled(
                "  rules to extract variables.",
                Style::default().fg(DIMMER),
            )]));
        }
        Some(run) => {
            // Header with column labels
            let var_w = (panel_w / 3).max(8).min(22);
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {:<w$}", "VARIABLE", w = var_w),
                    Style::default().fg(DIMMER).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "VALUE",
                    Style::default().fg(DIMMER).add_modifier(Modifier::BOLD),
                ),
            ]));
            lines.push(Line::from(vec![Span::styled(
                format!("  {}", "\u{2500}".repeat(panel_w)),
                Style::default().fg(DIM),
            )]));

            // Sort alphabetically
            let mut sorted: Vec<_> = run.final_context.iter().collect();
            sorted.sort_by_key(|(k, _)| k.as_str());

            for (key, val) in &sorted {
                use crate::api::commands::env as env_cmd;
                let type_tag = match val {
                    serde_json::Value::String(_) => "str",
                    serde_json::Value::Number(_) => "num",
                    serde_json::Value::Bool(_) => "bool",
                    serde_json::Value::Array(_) => "arr",
                    serde_json::Value::Object(_) => "obj",
                    serde_json::Value::Null => "null",
                };
                let val_max = panel_w.saturating_sub(var_w + 8);
                let raw = match val {
                    serde_json::Value::String(s) => truncate(s, val_max).to_string(),
                    other => truncate(&other.to_string(), val_max).to_string(),
                };

                let display = if env_cmd::looks_sensitive(key) {
                    env_cmd::mask(&raw)
                } else {
                    raw
                };

                lines.push(Line::from(vec![
                    Span::styled(
                        format!("  {:<w$}", truncate(key, var_w), w = var_w),
                        Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(display, Style::default().fg(TEXT_DIM)),
                    Span::styled(format!(" {}", type_tag), Style::default().fg(DIMMER)),
                ]));
            }

            // Count footer
            lines.push(Line::raw(""));
            lines.push(Line::from(vec![Span::styled(
                format!("  {} var(s) from last run", run.final_context.len()),
                Style::default().fg(DIMMER),
            )]));
        }
    }

    let block = Block::default()
        .title(Span::styled(" Flow Context (last run) ", title_style()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style());
    let para = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });
    f.render_widget(para, area);
}

// ── State inspector (Runner > Context subview) ────────────────────────────────

pub(super) fn render_state_inspector(f: &mut Frame, app: &ApiApp, area: Rect) {
    if app.flows.is_empty() {
        render_no_flows_hint(f, area, "State Inspector");
        return;
    }
    let run = match &app.last_run {
        Some(r) => r,
        None => {
            f.render_widget(
                Paragraph::new(vec![
                    Line::raw(""),
                    Line::from(vec![Span::styled("  No run yet.", dim_style())]),
                    Line::raw(""),
                    Line::from(vec![Span::styled(
                        "  Run a flow to inspect its context.",
                        Style::default().fg(DIMMER),
                    )]),
                ])
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(border_style()),
                ),
                area,
            );
            return;
        }
    };

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_context_panel(f, run, chunks[0]);
    render_schema_drift_panel(f, app, run, chunks[1]);
}

fn render_context_panel(f: &mut Frame, run: &FlowRunResult, area: Rect) {
    let mut lines: Vec<Line> = vec![Line::raw("")];
    let panel_w = area.width.saturating_sub(4) as usize;

    // Section header
    let header_w = panel_w.min(20);
    lines.push(Line::from(vec![Span::styled(
        format!("  \u{2500}\u{2500} Final Context {}", header_w),
        Style::default().fg(DIMMER),
    )]));

    if run.final_context.is_empty() {
        lines.push(Line::raw(""));
        lines.push(Line::from(vec![Span::styled(
            "  No context captured",
            dim_style(),
        )]));
        lines.push(Line::from(vec![Span::styled(
            "  \u{00B7} no data",
            Style::default().fg(DIM),
        )]));
    } else {
        let var_w = (panel_w / 3).max(8).min(20);
        // Sort alphabetically
        let mut sorted: Vec<_> = run.final_context.iter().collect();
        sorted.sort_by_key(|(k, _)| k.as_str());

        for (key, val) in &sorted {
            let val_max = panel_w.saturating_sub(var_w + 6);
            let display = match val {
                serde_json::Value::String(s) => {
                    if s.len() > val_max {
                        format!("{}\u{2026}", &s[..val_max])
                    } else {
                        s.clone()
                    }
                }
                other => truncate(&other.to_string(), val_max),
            };
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {:<w$}", truncate(key, var_w), w = var_w),
                    Style::default().fg(CYAN),
                ),
                Span::styled(" = ", Style::default().fg(DIM)),
                Span::styled(display, Style::default().fg(TEXT_DIM)),
            ]));
        }
    }

    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .title(Span::styled(" Final Context ", title_style()))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(border_style()),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(p, area);
}

fn render_schema_drift_panel(f: &mut Frame, app: &ApiApp, run: &FlowRunResult, area: Rect) {
    let mut lines: Vec<Line> = vec![Line::raw("")];
    let panel_w = area.width.saturating_sub(4) as usize;

    // Section header
    let header_w = panel_w.min(20);
    lines.push(Line::from(vec![Span::styled(
        format!("  \u{2500}\u{2500} Schema Drift {}", header_w),
        Style::default().fg(DIMMER),
    )]));

    if let Some(previous_run) = &app.compare_run {
        let mut has_drift = false;

        for step in &run.steps {
            let prev_step = previous_run
                .steps
                .iter()
                .find(|s| s.node_id == step.node_id);
            if let Some(prev) = prev_step {
                let curr_keys = extract_json_keys(step.response_body.as_deref());
                let prev_keys = extract_json_keys(prev.response_body.as_deref());

                let added: Vec<String> = curr_keys
                    .iter()
                    .filter(|k| !prev_keys.contains(k))
                    .cloned()
                    .collect();
                let removed: Vec<String> = prev_keys
                    .iter()
                    .filter(|k| !curr_keys.contains(k))
                    .cloned()
                    .collect();

                if !added.is_empty() || !removed.is_empty() {
                    has_drift = true;
                    lines.push(Line::raw(""));
                    lines.push(Line::from(vec![Span::styled(
                        format!("  {} ", step.node_id),
                        Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
                    )]));
                    for k in added {
                        lines.push(Line::from(vec![Span::styled(
                            format!("    + {}", k),
                            Style::default().fg(GREEN),
                        )]));
                    }
                    for k in removed {
                        lines.push(Line::from(vec![Span::styled(
                            format!("    - {}", k),
                            Style::default().fg(RED),
                        )]));
                    }
                }
            }
        }

        if !has_drift {
            lines.push(Line::raw(""));
            lines.push(Line::from(vec![
                Span::styled("  \u{2714} ", Style::default().fg(GREEN)),
                Span::styled("No schema changes detected", Style::default().fg(GREEN)),
            ]));
            lines.push(Line::raw(""));
            lines.push(Line::from(vec![Span::styled(
                "  Response schemas are identical.",
                Style::default().fg(DIMMER),
            )]));
        }
    } else {
        lines.push(Line::raw(""));
        lines.push(Line::from(vec![
            Span::styled("  \u{00B7} ", Style::default().fg(DIM)),
            Span::styled("No comparison run loaded", dim_style()),
        ]));
        lines.push(Line::raw(""));
        lines.push(Line::from(vec![
            Span::styled("  Press ", Style::default().fg(DIMMER)),
            Span::styled("[d]", Style::default().fg(YELLOW)),
            Span::styled(" to load comparison run", Style::default().fg(DIMMER)),
        ]));
        lines.push(Line::from(vec![Span::styled(
            "  This will diff response schemas",
            Style::default().fg(DIMMER),
        )]));
        lines.push(Line::from(vec![Span::styled(
            "  between the two most recent runs.",
            Style::default().fg(DIMMER),
        )]));
    }

    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .title(Span::styled(" Schema Drift ", title_style()))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(border_style()),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(p, area);
}

fn extract_json_keys(body: Option<&str>) -> Vec<String> {
    body.and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
        .and_then(|v| v.as_object().map(|o| o.keys().cloned().collect()))
        .unwrap_or_default()
}
