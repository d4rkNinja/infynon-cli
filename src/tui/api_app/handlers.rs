use crossterm::event::{KeyCode, KeyEvent};

use crate::api::storage;

use super::app_state::EnvEditState;
use super::types::*;

// ── View and main key handler methods on ApiApp ───────────────────────────────

impl super::app_state::ApiApp {
    pub fn handle_key(&mut self, key: KeyEvent) {
        // Prompt modal intercepts all keys
        if self.prompt_modal.is_some() {
            self.handle_prompt_modal_key(key);
            return;
        }

        // Step detail modal intercepts all keys (before body editor check)
        if self.step_detail.is_some() {
            self.handle_live_key(key);
            return;
        }

        // Body editor intercepts all keys
        if self.body_editor.is_some() {
            self.handle_body_editor_key(key);
            return;
        }

        // Node field editor intercepts all keys
        if self.node_field_editor.is_some() {
            self.handle_node_field_editor_key(key);
            return;
        }

        // Help overlay — any key closes it
        if self.show_help {
            self.show_help = false;
            return;
        }

        // Attach mode input
        if let AttachMode::SelectingTarget { .. } = &self.attach_mode {
            match key.code {
                KeyCode::Esc => {
                    self.attach_mode = AttachMode::Idle;
                    self.attach_input.clear();
                }
                KeyCode::Enter => self.confirm_attach(),
                KeyCode::Backspace => {
                    self.attach_input.pop();
                }
                KeyCode::Char(c) => {
                    self.attach_input.push(c);
                }
                _ => {}
            }
            return;
        }

        // Search mode
        if self.search_active {
            match key.code {
                KeyCode::Esc | KeyCode::Enter => {
                    self.search_active = false;
                }
                KeyCode::Backspace => {
                    self.search_input.pop();
                }
                KeyCode::Char(c) => {
                    self.search_input.push(c);
                }
                _ => {}
            }
            return;
        }

        // Env editor — intercept before global digit keys
        if self.env_edit.is_some() {
            self.handle_env_key(key);
            return;
        }

        // Global keys
        match key.code {
            KeyCode::Char('q') => {
                self.should_quit = true;
                return;
            }
            KeyCode::Char('1') => {
                self.current_view = ApiView::Dashboard;
                self.reset_scroll();
                return;
            }
            KeyCode::Char('2') => {
                self.current_view = ApiView::Nodes;
                self.reset_scroll();
                return;
            }
            KeyCode::Char('3') => {
                self.current_view = ApiView::Flows;
                self.reset_scroll();
                return;
            }
            KeyCode::Char('4') => {
                self.current_view = ApiView::Runner;
                self.reset_scroll();
                return;
            }
            KeyCode::Char('5') => {
                self.current_view = ApiView::Environment;
                self.reset_scroll();
                return;
            }
            KeyCode::Char('6') => {
                self.current_view = ApiView::Settings;
                self.reset_scroll();
                return;
            }
            KeyCode::Char('?') => {
                self.show_help = true;
                return;
            }
            KeyCode::Char('/') => {
                self.search_active = true;
                self.search_input.clear();
                return;
            }
            KeyCode::Char('R') => {
                self.refresh_data();
                return;
            }
            // Switch active flow with [ ]
            KeyCode::Char('[') => {
                if self.active_flow_idx > 0 {
                    self.switch_flow(self.active_flow_idx - 1);
                }
                return;
            }
            KeyCode::Char(']') => {
                if self.active_flow_idx + 1 < self.flows.len() {
                    self.switch_flow(self.active_flow_idx + 1);
                }
                return;
            }
            _ => {}
        }

        // View-specific keys
        match self.current_view {
            ApiView::Dashboard => self.handle_dashboard_key(key),
            ApiView::Nodes => self.handle_nodes_key(key),
            ApiView::Flows => self.handle_flows_key(key),
            ApiView::Runner => self.handle_runner_key(key),
            ApiView::Environment => self.handle_env_key(key),
            ApiView::Settings => self.handle_settings_key(key),
        }
    }

    fn handle_dashboard_key(&mut self, key: KeyEvent) {
        let max = self.flows.len();
        match key.code {
            KeyCode::Enter => {
                self.start_flow_run();
            }
            KeyCode::Char('a') => {
                // Run all flows sequentially (simplified: just run active for now)
                self.start_flow_run();
            }
            _ => self.handle_list_key(key, max),
        }
    }

    fn handle_nodes_key(&mut self, key: KeyEvent) {
        let max = self.nodes.len();
        match key.code {
            KeyCode::Char('r') | KeyCode::Enter => {
                // Get sorted node list and pick selected
                let mut node_ids: Vec<String> = self.nodes.keys().cloned().collect();
                node_ids.sort();
                if let Some(node_id) =
                    node_ids.get(self.selected_index.min(node_ids.len().saturating_sub(1)))
                {
                    let node_id = node_id.clone();
                    self.start_node_run(&node_id);
                }
            }
            KeyCode::Char('b') => {
                self.open_body_editor();
            }
            KeyCode::Char('n') => {
                self.open_node_field_editor(NodeField::Name);
            }
            KeyCode::Char('p') => {
                self.open_node_field_editor(NodeField::Path);
            }
            KeyCode::Char('d') => {
                self.open_node_field_editor(NodeField::Description);
            }
            KeyCode::Char('m') => {
                self.cycle_node_method();
            }
            KeyCode::Char('f') => {
                self.nodes_filter = self.nodes_filter.cycle();
                self.selected_index = 0;
                self.notify(&format!("Filter: {}", self.nodes_filter.label()));
            }
            _ => self.handle_list_key(key, max),
        }
    }

    fn handle_flows_key(&mut self, key: KeyEvent) {
        // Collect node IDs first to avoid borrow issues
        let node_ids: Vec<String> = match self.active_flow() {
            Some(f) => f.all_node_ids(),
            None => return,
        };
        let node_count = node_ids.len();
        match key.code {
            KeyCode::Up => {
                if self.pipeline_selected > 0 {
                    self.pipeline_selected -= 1;
                    // Sync visual selection immediately
                    if let Some(nid) = node_ids.get(self.pipeline_selected) {
                        self.graph_selected_id = Some(nid.clone());
                    }
                }
            }
            KeyCode::Down => {
                if self.pipeline_selected + 1 < node_count {
                    self.pipeline_selected += 1;
                    // Sync visual selection immediately
                    if let Some(nid) = node_ids.get(self.pipeline_selected) {
                        self.graph_selected_id = Some(nid.clone());
                    }
                }
            }
            KeyCode::Enter => {
                // Open detail panel for selected pipeline step
                self.toggle_detail_panel();
            }
            KeyCode::Char('a') => {
                // Start attach from selected pipeline step
                if let Some(nid) = node_ids.get(self.pipeline_selected).cloned() {
                    self.graph_selected_id = Some(nid.clone());
                    self.start_attach();
                }
            }
            KeyCode::Char('d') => {
                // Detach selected step from its successor
                if let Some(sel_id) = node_ids.get(self.pipeline_selected).cloned() {
                    if let Some(flow) = self.flows.get_mut(self.active_flow_idx) {
                        if let Some(succ) = flow.successors(&sel_id).first().map(|e| e.to.clone()) {
                            flow.edges.retain(|e| !(e.from == sel_id && e.to == succ));
                            storage::save_flow(flow).ok();
                            self.notify(&format!("Detached {} → {}", sel_id, succ));
                            self.refresh_data();
                        }
                    }
                }
            }
            KeyCode::Char('r') => {
                if !self.flow_running {
                    self.start_flow_run();
                }
            }
            _ => {}
        }
    }

    fn handle_runner_key(&mut self, key: KeyEvent) {
        // Handle step detail modal if open
        if let Some(ref mut modal) = self.step_detail {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.step_detail = None;
                }
                KeyCode::Up => {
                    if modal.scroll > 0 {
                        modal.scroll -= 1;
                    }
                }
                KeyCode::Down => {
                    modal.scroll += 1;
                }
                KeyCode::Home => {
                    modal.scroll = 0;
                }
                _ => {}
            }
            return;
        }

        // Tab to cycle sub-views
        if key.code == KeyCode::Tab {
            let all = RunnerSubview::all();
            let idx = all
                .iter()
                .position(|v| *v == self.runner_subview)
                .unwrap_or(0);
            self.runner_subview = all[(idx + 1) % all.len()];
            return;
        }

        // Sub-view specific keys
        match self.runner_subview {
            RunnerSubview::Steps => self.handle_live_key(key),
            RunnerSubview::Diff => match key.code {
                KeyCode::Char('d') => self.load_comparison_run(),
                _ => self.handle_scroll_key(key),
            },
            _ => self.handle_scroll_key(key),
        }
    }

    fn handle_settings_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('m') => {
                self.config_output_markdown = !self.config_output_markdown;
                self.notify(if self.config_output_markdown {
                    "Markdown output: ON"
                } else {
                    "Markdown output: OFF"
                });
            }
            KeyCode::Char('p') => {
                self.config_output_pdf = !self.config_output_pdf;
                self.notify(if self.config_output_pdf {
                    "PDF output: ON"
                } else {
                    "PDF output: OFF"
                });
            }
            KeyCode::Char('R') => self.refresh_data(),
            _ => {}
        }
    }

    pub fn handle_env_key(&mut self, key: KeyEvent) {
        use crate::api::commands::env as env_cmd;

        // ── Edit mode ──────────────────────────────────────────────────────────
        if self.env_edit.is_some() {
            let action: Option<(String, String, String)> = {
                let edit = self.env_edit.as_mut().unwrap();
                match key.code {
                    KeyCode::Esc => {
                        self.env_edit = None;
                        return;
                    }
                    KeyCode::Tab => {
                        if edit.is_new() {
                            edit.editing_key = !edit.editing_key;
                        }
                        return;
                    }
                    KeyCode::Enter => {
                        if edit.is_new() && edit.editing_key {
                            if !edit.key_input.trim().is_empty() {
                                edit.editing_key = false;
                            }
                            return;
                        }
                        Some((
                            edit.key_input.trim().to_string(),
                            edit.value_input.clone(),
                            edit.original_key.clone(),
                        ))
                    }
                    KeyCode::Backspace => {
                        if edit.editing_key {
                            edit.key_input.pop();
                        } else {
                            edit.value_input.pop();
                        }
                        return;
                    }
                    KeyCode::Char(c) => {
                        if edit.editing_key {
                            edit.key_input.push(c);
                        } else {
                            edit.value_input.push(c);
                        }
                        return;
                    }
                    _ => return,
                }
            };
            if let Some((key_str, val_str, orig_key)) = action {
                self.env_edit = None;
                self.env_reveal = false;
                if !key_str.is_empty() {
                    if !orig_key.is_empty() && orig_key != key_str {
                        let _ = env_cmd::env_delete_key(&orig_key);
                    }
                    match env_cmd::env_upsert(&key_str, &val_str) {
                        Ok(_) => self.notify(&format!("Saved: {}", key_str)),
                        Err(e) => self.notify(&format!("Save failed: {}", e)),
                    }
                }
            }
            return;
        }

        // ── Browse mode ────────────────────────────────────────────────────────
        let entries = env_cmd::env_list();
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.env_selected > 0 {
                    self.env_selected -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.env_selected + 1 < entries.len() {
                    self.env_selected += 1;
                }
            }
            KeyCode::Char('n') => {
                self.env_edit = Some(EnvEditState::new_entry());
                self.env_reveal = false;
            }
            KeyCode::Enter => {
                if let Some((k, v)) = entries.get(self.env_selected) {
                    self.env_edit = Some(EnvEditState::edit_existing(k, v));
                    self.env_reveal = false;
                }
            }
            KeyCode::Char('d') | KeyCode::Delete => {
                if let Some((k, _)) = entries.get(self.env_selected) {
                    let key_clone = k.clone();
                    match env_cmd::env_delete_key(&key_clone) {
                        Ok(true) => {
                            self.notify(&format!("Deleted: {}", key_clone));
                            let new_len = entries.len().saturating_sub(1);
                            self.env_selected = self.env_selected.min(new_len.saturating_sub(1));
                        }
                        Ok(false) => self.notify("Key not found"),
                        Err(e) => self.notify(&format!("Error: {}", e)),
                    }
                }
            }
            KeyCode::Char('v') => {
                self.env_reveal = !self.env_reveal;
                self.notify(if self.env_reveal {
                    "Values revealed"
                } else {
                    "Sensitive values hidden"
                });
            }
            _ => {}
        }
    }

    fn handle_list_key(&mut self, key: KeyEvent, max: usize) {
        match key.code {
            KeyCode::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
            }
            KeyCode::Down => {
                if self.selected_index + 1 < max {
                    self.selected_index += 1;
                }
            }
            _ => {}
        }
    }

    fn handle_scroll_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => {
                if self.scroll_offset > 0 {
                    self.scroll_offset -= 1;
                }
            }
            KeyCode::Down => {
                self.scroll_offset += 1;
            }
            KeyCode::Home => {
                self.scroll_offset = 0;
            }
            _ => {}
        }
    }

    fn reset_scroll(&mut self) {
        self.scroll_offset = 0;
        self.selected_index = 0;
    }
}
