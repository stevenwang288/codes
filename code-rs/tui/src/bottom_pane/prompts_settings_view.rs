use std::fs;
use std::path::PathBuf;

use code_core::config::find_code_home;
use code_core::protocol::Op;
use code_i18n;
use code_protocol::custom_prompts::CustomPrompt;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::prelude::Widget;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app_event::AppEvent;
use crate::app_event_sender::AppEventSender;
use crate::colors;
use crate::slash_command::built_in_slash_commands;

use super::form_text_field::{FormTextField, InputFilter};
// Panel helpers unused now that we render inline

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Focus {
    List,
    Name,
    Body,
    Save,
    Delete,
    Cancel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    List,
    Edit,
}

pub(crate) struct PromptsSettingsView {
    prompts: Vec<CustomPrompt>,
    selected: usize,
    focus: Focus,
    name_field: FormTextField,
    body_field: FormTextField,
    status: Option<(String, Style)>,
    app_event_tx: AppEventSender,
    is_complete: bool,
    mode: Mode,
}

impl PromptsSettingsView {
    pub fn new(prompts: Vec<CustomPrompt>, app_event_tx: AppEventSender) -> Self {
        let mut name_field = FormTextField::new_single_line();
        name_field.set_filter(InputFilter::Id);
        let body_field = FormTextField::new_multi_line();
        let view = Self {
            prompts,
            selected: 0,
            focus: Focus::List,
            name_field,
            body_field,
            status: None,
            app_event_tx,
            is_complete: false,
            mode: Mode::List,
        };
        view
    }

    pub fn handle_key_event_direct(&mut self, key: KeyEvent) -> bool {
        if self.is_complete {
            return true;
        }
        match self.mode {
            Mode::List => match key {
                KeyEvent { code: KeyCode::Esc, .. } => {
                    self.is_complete = true;
                    true
                }
                KeyEvent { code: KeyCode::Enter, modifiers: KeyModifiers::NONE, .. } => {
                    self.enter_editor();
                    true
                }
                KeyEvent { code: KeyCode::Char('n'), modifiers, .. }
                    if modifiers.contains(KeyModifiers::CONTROL) =>
                {
                    self.start_new_prompt();
                    true
                }
                other => self.handle_list_key(other),
            },
            Mode::Edit => match key {
                KeyEvent { code: KeyCode::Esc, .. } => {
                    self.mode = Mode::List;
                    self.focus = Focus::List;
                    self.status = None;
                    true
                }
                KeyEvent { code: KeyCode::Tab, .. } => {
                    self.cycle_focus(true);
                    true
                }
                KeyEvent { code: KeyCode::BackTab, .. } => {
                    self.cycle_focus(false);
                    true
                }
                KeyEvent { code: KeyCode::Enter, modifiers: KeyModifiers::NONE, .. } => {
                    match self.focus {
                        Focus::Save => self.save_current(),
                        Focus::Delete => self.delete_current(),
                        Focus::Cancel => {
                            self.mode = Mode::List;
                            self.focus = Focus::List;
                            self.status = None;
                        }
                        _ => {}
                    }
                    true
                }
                KeyEvent { code: KeyCode::Char('n'), modifiers, .. }
                    if modifiers.contains(KeyModifiers::CONTROL) =>
                {
                    self.start_new_prompt();
                    true
                }
                _ => match self.focus {
                    Focus::Name => {
                        self.name_field.handle_key(key);
                        true
                    }
                    Focus::Body => {
                        self.body_field.handle_key(key);
                        true
                    }
                    Focus::Save | Focus::Delete | Focus::Cancel => false,
                    Focus::List => self.handle_list_key(key),
                },
            },
        }
    }

    pub fn is_complete(&self) -> bool { self.is_complete }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 { return; }
        self.render_body(area, buf);
    }

    fn render_body(&self, area: Rect, buf: &mut Buffer) {
        match self.mode {
            Mode::List => {
                self.render_list(area, buf);
            }
            Mode::Edit => {
            self.render_form(area, buf);
        }
    }
    }

    fn render_list(&self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 { return; }
        let mut lines: Vec<Line> = Vec::new();
        for (idx, p) in self.prompts.iter().enumerate() {
            let preview = p.content.lines().next().unwrap_or("").trim();
            let arrow = if idx == self.selected { "›" } else { " " };
            let name_style = if idx == self.selected {
                Style::default().fg(colors::primary()).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(colors::text())
            };
            let name_span = Span::styled(format!("{arrow} /{}", p.name), name_style);
            let preview_span = Span::styled(
                format!("  {}", preview),
                Style::default().fg(colors::text_dim()),
            );
            let mut spans = vec![name_span];
            if !preview.is_empty() { spans.push(preview_span); }
            let line = Line::from(spans);
            lines.push(line);
        }
        if lines.is_empty() {
            lines.push(Line::from(code_i18n::tr_plain("tui.prompts_settings.empty")));
        }

        // Add new row
        let add_arrow = if self.selected == self.prompts.len() { "›" } else { " " };
        let add_style = if self.selected == self.prompts.len() {
            Style::default().fg(colors::primary()).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(colors::success()).add_modifier(Modifier::BOLD)
        };
        let add_line = Line::from(vec![Span::styled(
            format!("{add_arrow} {}", code_i18n::tr_plain("tui.prompts_settings.add_new")),
            add_style,
        )]);
        lines.push(add_line);

        let title = Paragraph::new(vec![Line::from(Span::styled(
            code_i18n::tr_plain("tui.prompts_settings.description"),
            Style::default().fg(colors::text_dim()),
        ))])
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true })
        .style(Style::default().bg(colors::background()));

        let list = Paragraph::new(lines)
            .alignment(Alignment::Left)
            .style(Style::default().bg(colors::background()));

        let outer = Block::default().borders(Borders::ALL).style(Style::default().bg(colors::background()));
        let inner = outer.inner(area);
        outer.render(area, buf);
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(1),
            ])
            .split(inner);

        title.render(chunks[0], buf);
        list.render(chunks[1], buf);
    }

    fn render_form(&self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 { return; }
        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // name block
                Constraint::Min(6),    // body block
                Constraint::Length(1), // buttons
                Constraint::Length(1), // status
            ])
            .split(area);

        // Name field with border
        let name_title = if matches!(self.focus, Focus::Name) {
            code_i18n::tr_plain("tui.prompts_settings.name_title_active")
        } else {
            code_i18n::tr_plain("tui.prompts_settings.name_title")
        };
        let mut name_block = Block::default().borders(Borders::ALL).title(name_title);
        if matches!(self.focus, Focus::Name) {
            name_block = name_block.border_style(Style::default().fg(colors::primary()));
        }
        let name_inner = name_block.inner(vertical[0]);
        name_block.render(vertical[0], buf);
        self.name_field.render(name_inner, buf, matches!(self.focus, Focus::Name));

        // Body field with border
        let body_title = if matches!(self.focus, Focus::Body) {
            code_i18n::tr_plain("tui.prompts_settings.body_title_active")
        } else {
            code_i18n::tr_plain("tui.prompts_settings.body_title")
        };
        let mut body_block = Block::default().borders(Borders::ALL).title(body_title);
        if matches!(self.focus, Focus::Body) {
            body_block = body_block.border_style(Style::default().fg(colors::primary()));
        }
        let body_inner = body_block.inner(vertical[1]);
        body_block.render(vertical[1], buf);
        self.body_field.render(body_inner, buf, matches!(self.focus, Focus::Body));

        // Buttons
        let buttons_area = vertical[2];
        let save_base = code_i18n::tr_plain("tui.prompts_settings.button.save");
        let delete_base = code_i18n::tr_plain("tui.prompts_settings.button.delete");
        let cancel_base = code_i18n::tr_plain("tui.prompts_settings.button.cancel");
        let save_label = if matches!(self.focus, Focus::Save) {
            format!("[{save_base}]")
        } else {
            save_base.to_string()
        };
        let delete_label = if matches!(self.focus, Focus::Delete) {
            format!("[{delete_base}]")
        } else {
            delete_base.to_string()
        };
        let cancel_label = if matches!(self.focus, Focus::Cancel) {
            format!("[{cancel_base}]")
        } else {
            cancel_base.to_string()
        };
        let btn_span = |label: &str, focus: Focus, color: Style| {
            if self.focus == focus {
                Span::styled(label.to_string(), color.bg(colors::primary()).fg(colors::background()))
            } else {
                Span::styled(label.to_string(), color)
            }
        };
        let line = Line::from(vec![
            btn_span(&save_label, Focus::Save, Style::default().fg(colors::success()).add_modifier(Modifier::BOLD)),
            Span::raw("   "),
            btn_span(&delete_label, Focus::Delete, Style::default().fg(colors::error()).add_modifier(Modifier::BOLD)),
            Span::raw("   "),
            btn_span(&cancel_label, Focus::Cancel, Style::default().fg(colors::text_dim()).add_modifier(Modifier::BOLD)),
            Span::raw(format!("    {}", code_i18n::tr_plain("tui.prompts_settings.hints.tab_cycle_enter_activate"))),
        ]);
        Paragraph::new(line).render(buttons_area, buf);

        // Status
        if let Some((msg, style)) = &self.status {
            Paragraph::new(Line::from(Span::styled(msg.clone(), *style)))
                .alignment(Alignment::Left)
                .render(vertical[3], buf);
        }
    }

    fn handle_list_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Up => {
                if self.selected > 0 { self.selected -= 1; }
                return true;
            }
            KeyCode::Down => {
                let max = self.prompts.len();
                if self.selected < max { self.selected += 1; }
                return true;
            }
            KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.start_new_prompt();
                return true;
            }
            _ => {}
        }
        false
    }

    fn start_new_prompt(&mut self) {
        self.selected = self.prompts.len();
        self.name_field.set_text("");
        self.body_field.set_text("");
        self.focus = Focus::Name;
        self.status = Some((
            code_i18n::tr_plain("tui.prompts_settings.status.new_prompt").to_string(),
            Style::default().fg(colors::info()),
        ));
        self.mode = Mode::Edit;
    }

    fn load_selected_into_form(&mut self) {
        if let Some(p) = self.prompts.get(self.selected) {
            self.name_field.set_text(&p.name);
            self.body_field.set_text(&p.content);
            self.focus = Focus::Name;
        }
    }

    fn enter_editor(&mut self) {
        if self.selected >= self.prompts.len() {
            self.start_new_prompt();
        } else {
            self.load_selected_into_form();
            self.mode = Mode::Edit;
        }
    }

    fn cycle_focus(&mut self, forward: bool) {
        let order = [Focus::List, Focus::Name, Focus::Body, Focus::Save, Focus::Delete, Focus::Cancel];
        let mut idx = order.iter().position(|f| *f == self.focus).unwrap_or(0);
        if forward {
            idx = (idx + 1) % order.len();
        } else {
            idx = idx.checked_sub(1).unwrap_or(order.len() - 1);
        }
        self.focus = order[idx];
    }

    fn validate(&self, name: &str) -> Result<(), String> {
        let slug = name.trim();
        if slug.is_empty() {
            return Err(code_i18n::tr_plain("tui.prompts_settings.error.name_required").to_string());
        }
        if !slug
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
        {
            return Err(
                code_i18n::tr_plain("tui.prompts_settings.error.name_invalid_chars").to_string(),
            );
        }

        let builtin: Vec<String> = built_in_slash_commands()
            .into_iter()
            .map(|(n, _)| n.to_ascii_lowercase())
            .collect();
        if builtin.contains(&slug.to_ascii_lowercase()) {
            return Err(code_i18n::tr_plain("tui.prompts_settings.error.name_conflicts_builtin").to_string());
        }

        let dup = self
            .prompts
            .iter()
            .enumerate()
            .any(|(idx, p)| idx != self.selected && p.name.eq_ignore_ascii_case(slug));
        if dup {
            return Err(code_i18n::tr_plain("tui.prompts_settings.error.name_exists").to_string());
        }
        Ok(())
    }

    fn save_current(&mut self) {
        let name = self.name_field.text().trim().to_string();
        let body = self.body_field.text().to_string();
        match self.validate(&name) {
            Ok(()) => {}
            Err(msg) => {
                self.status = Some((msg, Style::default().fg(colors::error())));
                return;
            }
        }

        let code_home = match find_code_home() {
            Ok(path) => path,
            Err(e) => {
                let msg = format!(
                    "{}: {e}",
                    code_i18n::tr_plain("tui.prompts_settings.error.code_home_unavailable")
                );
                self.status = Some((msg, Style::default().fg(colors::error())));
                return;
            }
        };
        let mut dir = code_home;
        dir.push("prompts");
        if let Err(e) = fs::create_dir_all(&dir) {
            let msg = format!(
                "{}: {e}",
                code_i18n::tr_plain("tui.prompts_settings.error.create_dir_failed")
            );
            self.status = Some((msg, Style::default().fg(colors::error())));
            return;
        }
        let mut path = PathBuf::from(&dir);
        path.push(format!("{name}.md"));
        if let Err(e) = fs::write(&path, &body) {
            let msg =
                format!("{}: {e}", code_i18n::tr_plain("tui.prompts_settings.error.save_failed"));
            self.status = Some((msg, Style::default().fg(colors::error())));
            return;
        }

        // Update local list
        let mut updated = self.prompts.clone();
        let new_entry = CustomPrompt { name: name.clone(), path, content: body.clone(), description: None, argument_hint: None };
        if self.selected < updated.len() {
            updated[self.selected] = new_entry;
        } else {
            updated.push(new_entry);
            self.selected = updated.len() - 1;
        }
        self.prompts = updated;
        self.status = Some((
            code_i18n::tr_plain("tui.prompts_settings.status.saved").to_string(),
            Style::default().fg(colors::success()),
        ));

        // Trigger reload so composer autocomplete picks it up.
        self.app_event_tx.send(AppEvent::CodexOp(Op::ListCustomPrompts));
    }

    fn delete_current(&mut self) {
        if self.selected >= self.prompts.len() {
            self.status = Some((
                code_i18n::tr_plain("tui.prompts_settings.status.nothing_to_delete").to_string(),
                Style::default().fg(colors::warning()),
            ));
            self.mode = Mode::List;
            self.focus = Focus::List;
            return;
        }
        let prompt = self.prompts[self.selected].clone();
        if let Err(e) = fs::remove_file(&prompt.path) {
            // Ignore missing file but surface other errors
            if e.kind() != std::io::ErrorKind::NotFound {
                let msg = format!(
                    "{}: {e}",
                    code_i18n::tr_plain("tui.prompts_settings.error.delete_failed")
                );
                self.status = Some((msg, Style::default().fg(colors::error())));
                return;
            }
        }
        self.prompts.remove(self.selected);
        if self.selected > 0 && self.selected >= self.prompts.len() {
            self.selected -= 1;
        }
        self.mode = Mode::List;
        self.focus = Focus::List;
        self.status = Some((
            code_i18n::tr_plain("tui.prompts_settings.status.deleted").to_string(),
            Style::default().fg(colors::success()),
        ));
        self.app_event_tx.send(AppEvent::CodexOp(Op::ListCustomPrompts));
    }
}
