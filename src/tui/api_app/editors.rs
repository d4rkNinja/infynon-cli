use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent};

use super::types::*;

// ── Editor / modal key handlers on ApiApp ─────────────────────────────────────

impl super::app_state::ApiApp {
    fn advance_prompt_field_or_submit(&mut self, field_count: usize) {
        let should_submit = self.prompt_modal.as_ref()
            .map(|m| m.current_field + 1 >= field_count)
            .unwrap_or(false);
        if should_submit {
            self.submit_prompt_modal();
        } else if let Some(modal) = self.prompt_modal.as_mut() {
            modal.current_field += 1;
        }
    }

    pub fn handle_prompt_modal_key(&mut self, key: KeyEvent) {
        use crate::api::types::PromptType;
        let (field_count, ci, pt) = match self.prompt_modal.as_ref() {
            None => return,
            Some(m) => {
                let fc = m.inputs.len();
                if fc == 0 { return; }
                let ci = m.current_field;
                let pt = m.inputs.get(ci).map(|p| p.prompt_type.clone()).unwrap_or(PromptType::Text);
                (fc, ci, pt)
            }
        };

        match key.code {
            KeyCode::Esc => {
                if let Some(tx) = &self.prompt_reply_tx {
                    tx.send(HashMap::new()).ok();
                }
                self.prompt_modal = None;
                return;
            }
            KeyCode::Tab => {
                if let Some(modal) = self.prompt_modal.as_mut() {
                    modal.current_field = (modal.current_field + 1) % field_count;
                }
                return;
            }
            _ => {}
        }

        match pt {
            PromptType::Boolean => {
                match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        if let Some(modal) = self.prompt_modal.as_mut() { modal.values[ci] = "true".to_string(); }
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') => {
                        if let Some(modal) = self.prompt_modal.as_mut() { modal.values[ci] = "false".to_string(); }
                    }
                    KeyCode::Char(' ') | KeyCode::Left | KeyCode::Right => {
                        if let Some(modal) = self.prompt_modal.as_mut() {
                            modal.values[ci] = if modal.values[ci] == "true" { "false".to_string() } else { "true".to_string() };
                        }
                    }
                    KeyCode::Down => {
                        if let Some(modal) = self.prompt_modal.as_mut() {
                            if modal.current_field + 1 < field_count { modal.current_field += 1; }
                        }
                    }
                    KeyCode::Up => {
                        if let Some(modal) = self.prompt_modal.as_mut() {
                            if modal.current_field > 0 { modal.current_field -= 1; }
                        }
                    }
                    KeyCode::Enter => {
                        self.advance_prompt_field_or_submit(field_count);
                    }
                    _ => {}
                }
            }
            PromptType::Select => {
                match key.code {
                    KeyCode::Up => {
                        if let Some(modal) = self.prompt_modal.as_mut() {
                            if modal.option_cursors[ci] > 0 { modal.option_cursors[ci] -= 1; }
                        }
                    }
                    KeyCode::Down => {
                        if let Some(modal) = self.prompt_modal.as_mut() {
                            let max = modal.inputs[ci].options.len().saturating_sub(1);
                            if modal.option_cursors[ci] < max { modal.option_cursors[ci] += 1; }
                        }
                    }
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        if let Some(modal) = self.prompt_modal.as_mut() {
                            let idx = modal.option_cursors[ci];
                            let chosen = modal.inputs[ci].options.get(idx).cloned().unwrap_or_default();
                            modal.values[ci] = chosen;
                        }
                        self.advance_prompt_field_or_submit(field_count);
                    }
                    _ => {}
                }
            }
            PromptType::Multiselect => {
                match key.code {
                    KeyCode::Up => {
                        if let Some(modal) = self.prompt_modal.as_mut() {
                            if modal.option_cursors[ci] > 0 { modal.option_cursors[ci] -= 1; }
                        }
                    }
                    KeyCode::Down => {
                        if let Some(modal) = self.prompt_modal.as_mut() {
                            let max = modal.inputs[ci].options.len().saturating_sub(1);
                            if modal.option_cursors[ci] < max { modal.option_cursors[ci] += 1; }
                        }
                    }
                    KeyCode::Char(' ') => {
                        if let Some(modal) = self.prompt_modal.as_mut() {
                            let idx = modal.option_cursors[ci];
                            if idx < modal.multi_checked[ci].len() {
                                modal.multi_checked[ci][idx] = !modal.multi_checked[ci][idx];
                            }
                            let checked_vals: Vec<String> = modal.inputs[ci].options.iter().enumerate()
                                .filter(|(j, _)| modal.multi_checked[ci].get(*j).copied().unwrap_or(false))
                                .map(|(_, o)| o.clone())
                                .collect();
                            modal.values[ci] = checked_vals.join(",");
                        }
                    }
                    KeyCode::Enter => {
                        self.advance_prompt_field_or_submit(field_count);
                    }
                    _ => {}
                }
            }
            PromptType::Text => {
                match key.code {
                    KeyCode::Char(c) => {
                        if let Some(modal) = self.prompt_modal.as_mut() { modal.values[ci].push(c); }
                    }
                    KeyCode::Backspace => {
                        if let Some(modal) = self.prompt_modal.as_mut() { modal.values[ci].pop(); }
                    }
                    KeyCode::Down => {
                        if let Some(modal) = self.prompt_modal.as_mut() {
                            if modal.current_field + 1 < field_count { modal.current_field += 1; }
                        }
                    }
                    KeyCode::Up => {
                        if let Some(modal) = self.prompt_modal.as_mut() {
                            if modal.current_field > 0 { modal.current_field -= 1; }
                        }
                    }
                    KeyCode::Enter => {
                        self.advance_prompt_field_or_submit(field_count);
                    }
                    _ => {}
                }
            }
        }
    }

    fn submit_prompt_modal(&mut self) {
        let modal = match &self.prompt_modal {
            Some(m) => m,
            None => return,
        };
        let mut map: HashMap<String, serde_json::Value> = HashMap::new();
        for (i, pi) in modal.inputs.iter().enumerate() {
            let raw = modal.values.get(i).cloned().unwrap_or_default();
            let val = if raw.is_empty() {
                pi.default.clone().unwrap_or_default()
            } else {
                raw
            };
            map.insert(pi.var.clone(), serde_json::Value::String(val));
        }
        if let Some(tx) = &self.prompt_reply_tx {
            tx.send(map).ok();
        }
        self.prompt_modal = None;
    }

    pub fn open_body_editor(&mut self) {
        let mut node_ids: Vec<String> = self.nodes.keys().cloned().collect();
        node_ids.sort();
        let node_id = match node_ids.get(self.selected_index.min(node_ids.len().saturating_sub(1))) {
            Some(id) => id.clone(),
            None => return,
        };
        let body = self.nodes.get(&node_id).and_then(|n| n.body_json.as_deref());
        self.body_editor = Some(BodyEditor::new(node_id, body));
    }

    pub fn open_body_editor_for_node(&mut self, node_id: &str) {
        let body = self.nodes.get(node_id).and_then(|n| n.body_json.as_deref());
        self.body_editor = Some(BodyEditor::new(node_id.to_string(), body));
    }

    pub fn handle_body_editor_key(&mut self, key: KeyEvent) {
        use crossterm::event::KeyModifiers;
        let editor = match self.body_editor.as_mut() {
            Some(e) => e,
            None => return,
        };

        match key.code {
            KeyCode::Esc => {
                self.body_editor = None;
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Save body to node
                let content = editor.to_string();
                let node_id = editor.node_id.clone();
                self.body_editor = None;

                if let Some(node) = self.nodes.get_mut(&node_id) {
                    // Validate JSON (compact it for storage)
                    let body_val = serde_json::from_str::<serde_json::Value>(&content).ok();
                    node.body_json = Some(body_val
                        .map(|v| serde_json::to_string(&v).unwrap_or(content.clone()))
                        .unwrap_or(content));
                    let node_clone = node.clone();
                    match crate::api::storage::save_node(&node_clone) {
                        Ok(_) => self.notify("Body saved"),
                        Err(e) => self.notify(&format!("Save failed: {}", e)),
                    }
                }
            }
            KeyCode::Enter => {
                // Split current line at cursor
                let rest = {
                    let line = editor.lines.get_mut(editor.cursor_row).unwrap();
                    let rest = line[editor.cursor_col..].to_string();
                    line.truncate(editor.cursor_col);
                    rest
                };
                editor.cursor_row += 1;
                editor.cursor_col = 0;
                editor.lines.insert(editor.cursor_row, rest);
            }
            KeyCode::Backspace => {
                if editor.cursor_col > 0 {
                    let line = editor.lines.get_mut(editor.cursor_row).unwrap();
                    editor.cursor_col -= 1;
                    line.remove(editor.cursor_col);
                } else if editor.cursor_row > 0 {
                    // Merge with previous line
                    let current = editor.lines.remove(editor.cursor_row);
                    editor.cursor_row -= 1;
                    editor.cursor_col = editor.lines[editor.cursor_row].len();
                    editor.lines[editor.cursor_row].push_str(&current);
                }
            }
            KeyCode::Delete => {
                let line_len = editor.current_line_len();
                if editor.cursor_col < line_len {
                    editor.lines[editor.cursor_row].remove(editor.cursor_col);
                } else if editor.cursor_row + 1 < editor.lines.len() {
                    // Merge next line into current
                    let next = editor.lines.remove(editor.cursor_row + 1);
                    editor.lines[editor.cursor_row].push_str(&next);
                }
            }
            KeyCode::Left => {
                if editor.cursor_col > 0 {
                    editor.cursor_col -= 1;
                } else if editor.cursor_row > 0 {
                    editor.cursor_row -= 1;
                    editor.cursor_col = editor.current_line_len();
                }
            }
            KeyCode::Right => {
                let len = editor.current_line_len();
                if editor.cursor_col < len {
                    editor.cursor_col += 1;
                } else if editor.cursor_row + 1 < editor.lines.len() {
                    editor.cursor_row += 1;
                    editor.cursor_col = 0;
                }
            }
            KeyCode::Up => {
                if editor.cursor_row > 0 {
                    editor.cursor_row -= 1;
                    editor.cursor_col = editor.cursor_col.min(editor.current_line_len());
                    // Adjust scroll
                    if editor.cursor_row < editor.scroll_top {
                        editor.scroll_top = editor.cursor_row;
                    }
                }
            }
            KeyCode::Down => {
                if editor.cursor_row + 1 < editor.lines.len() {
                    editor.cursor_row += 1;
                    editor.cursor_col = editor.cursor_col.min(editor.current_line_len());
                }
            }
            KeyCode::Home => {
                editor.cursor_col = 0;
            }
            KeyCode::End => {
                editor.cursor_col = editor.current_line_len();
            }
            KeyCode::Char(c) => {
                // Insert character at cursor
                if editor.lines.is_empty() {
                    editor.lines.push(String::new());
                }
                editor.lines[editor.cursor_row].insert(editor.cursor_col, c);
                editor.cursor_col += 1;
            }
            KeyCode::Tab => {
                // Insert 2 spaces
                for _ in 0..2 {
                    editor.lines[editor.cursor_row].insert(editor.cursor_col, ' ');
                    editor.cursor_col += 1;
                }
            }
            _ => {}
        }

        // Adjust scroll_top so cursor stays visible (assume ~20 visible lines)
        let visible_lines = 20usize;
        let editor = self.body_editor.as_mut().unwrap();
        if editor.cursor_row >= editor.scroll_top + visible_lines {
            editor.scroll_top = editor.cursor_row.saturating_sub(visible_lines - 1);
        }
        if editor.cursor_row < editor.scroll_top {
            editor.scroll_top = editor.cursor_row;
        }
    }

    pub(crate) fn handle_live_key(&mut self, key: KeyEvent) {
        // If modal is open, handle scroll/close
        if let Some(ref mut modal) = self.step_detail {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') => { self.step_detail = None; }
                KeyCode::Up   => { if modal.scroll > 0 { modal.scroll -= 1; } }
                KeyCode::Down => { modal.scroll += 1; }
                KeyCode::Home => { modal.scroll = 0; }
                _ => {}
            }
            return;
        }

        let step_count = if self.live_steps.is_empty() {
            self.last_run.as_ref().map(|r| r.steps.len()).unwrap_or(0)
        } else {
            self.live_steps.len()
        };

        match key.code {
            KeyCode::Up => {
                if self.live_selected_step > 0 { self.live_selected_step -= 1; }
            }
            KeyCode::Down => {
                if self.live_selected_step + 1 < step_count { self.live_selected_step += 1; }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                let steps: Vec<_> = if self.live_steps.is_empty() {
                    self.last_run.as_ref().map(|r| r.steps.clone()).unwrap_or_default()
                } else {
                    self.live_steps.clone()
                };
                if let Some(step) = steps.get(self.live_selected_step) {
                    self.step_detail = Some(StepDetailModal { step: step.clone(), scroll: 0 });
                }
            }
            KeyCode::Char('r') => {
                if !self.flow_running {
                    self.start_flow_run();
                }
            }
            KeyCode::Char('b') => {
                let steps: Vec<_> = if self.live_steps.is_empty() {
                    self.last_run.as_ref().map(|r| r.steps.clone()).unwrap_or_default()
                } else {
                    self.live_steps.clone()
                };
                if let Some(step) = steps.get(self.live_selected_step) {
                    let node_id = step.node_id.clone();
                    self.open_body_editor_for_node(&node_id);
                }
            }
            _ => {}
        }
    }

    pub fn open_node_field_editor(&mut self, field: NodeField) {
        let mut node_ids: Vec<String> = self.nodes.keys().cloned().collect();
        node_ids.sort();
        let node_id = match node_ids.get(self.selected_index.min(node_ids.len().saturating_sub(1))) {
            Some(id) => id.clone(),
            None => return,
        };
        let initial = match &field {
            NodeField::Name => self.nodes.get(&node_id).map(|n| n.name.clone()).unwrap_or_default(),
            NodeField::Path => self.nodes.get(&node_id).map(|n| n.path.clone()).unwrap_or_default(),
            NodeField::Description => self.nodes.get(&node_id).and_then(|n| n.description.clone()).unwrap_or_default(),
            NodeField::Method => self.nodes.get(&node_id).map(|n| n.method.clone()).unwrap_or_default(),
        };
        self.node_field_editor = Some(NodeFieldEditor { node_id, field, input: initial });
    }

    pub fn cycle_node_method(&mut self) {
        let methods = ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD"];
        let mut node_ids: Vec<String> = self.nodes.keys().cloned().collect();
        node_ids.sort();
        let node_id = match node_ids.get(self.selected_index.min(node_ids.len().saturating_sub(1))) {
            Some(id) => id.clone(),
            None => return,
        };
        if let Some(node) = self.nodes.get_mut(&node_id) {
            let curr_idx = methods.iter().position(|&m| m == node.method.as_str()).unwrap_or(0);
            node.method = methods[(curr_idx + 1) % methods.len()].to_string();
            let node_clone = node.clone();
            match crate::api::storage::save_node(&node_clone) {
                Ok(_) => self.notify(&format!("Method → {}", node_clone.method)),
                Err(e) => self.notify(&format!("Save failed: {}", e)),
            }
        }
    }

    pub fn handle_node_field_editor_key(&mut self, key: KeyEvent) {
        let editor = match self.node_field_editor.as_mut() {
            Some(e) => e,
            None => return,
        };
        match key.code {
            KeyCode::Esc => { self.node_field_editor = None; }
            KeyCode::Backspace => { editor.input.pop(); }
            KeyCode::Enter => {
                // Save
                let node_id = editor.node_id.clone();
                let field = editor.field.clone();
                let value = editor.input.clone();
                self.node_field_editor = None;

                if let Some(node) = self.nodes.get_mut(&node_id) {
                    match field {
                        NodeField::Name => node.name = value,
                        NodeField::Path => {
                            let path = if value.starts_with('/') { value } else { format!("/{}", value) };
                            node.path = path;
                        }
                        NodeField::Description => {
                            node.description = if value.is_empty() { None } else { Some(value) };
                        }
                        NodeField::Method => {} // handled by cycle
                    }
                    let node_clone = node.clone();
                    match crate::api::storage::save_node(&node_clone) {
                        Ok(_) => self.notify("Node saved"),
                        Err(e) => self.notify(&format!("Save failed: {}", e)),
                    }
                }
            }
            KeyCode::Char(c) => { editor.input.push(c); }
            _ => {}
        }
    }
}
