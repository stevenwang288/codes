use crate::app_event::{AppEvent, AutoContinueMode};
use crate::app_event_sender::AppEventSender;
use crate::colors;
use code_core::config_types::ReasoningEffort;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Widget;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};

use super::bottom_pane_view::{BottomPaneView, ConditionalUpdate};
use super::BottomPane;
use super::settings_panel::{render_panel, PanelFrameStyle};

pub(crate) struct AutoDriveSettingsView {
    app_event_tx: AppEventSender,
    selected_index: usize,
    model: String,
    model_reasoning: ReasoningEffort,
    use_chat_model: bool,
    review_enabled: bool,
    agents_enabled: bool,
    cross_check_enabled: bool,
    qa_automation_enabled: bool,
    diagnostics_enabled: bool,
    continue_mode: AutoContinueMode,
    closing: bool,
}

impl AutoDriveSettingsView {
    pub fn new(
        app_event_tx: AppEventSender,
        model: String,
        model_reasoning: ReasoningEffort,
        use_chat_model: bool,
        review_enabled: bool,
        agents_enabled: bool,
        cross_check_enabled: bool,
        qa_automation_enabled: bool,
        continue_mode: AutoContinueMode,
    ) -> Self {
        let diagnostics_enabled = qa_automation_enabled
            && (review_enabled || cross_check_enabled);
        Self {
            app_event_tx,
            selected_index: 0,
            model,
            model_reasoning,
            use_chat_model,
            review_enabled,
            agents_enabled,
            cross_check_enabled,
            qa_automation_enabled,
            diagnostics_enabled,
            continue_mode,
            closing: false,
        }
    }

    fn option_count() -> usize {
        4
    }

    fn send_update(&self) {
        self.app_event_tx.send(AppEvent::AutoDriveSettingsChanged {
            review_enabled: self.review_enabled,
            agents_enabled: self.agents_enabled,
            cross_check_enabled: self.cross_check_enabled,
            qa_automation_enabled: self.qa_automation_enabled,
            continue_mode: self.continue_mode,
        });
    }

    pub fn set_model(&mut self, model: String, effort: ReasoningEffort) {
        self.model = model;
        self.model_reasoning = effort;
    }

    pub fn set_use_chat_model(&mut self, use_chat: bool, model: String, effort: ReasoningEffort) {
        self.use_chat_model = use_chat;
        if use_chat {
            self.model = model;
            self.model_reasoning = effort;
        }
    }

    fn set_diagnostics(&mut self, enabled: bool) {
        self.review_enabled = enabled;
        self.cross_check_enabled = enabled;
        self.qa_automation_enabled = enabled;
        self.diagnostics_enabled =
            self.qa_automation_enabled && (self.review_enabled || self.cross_check_enabled);
    }

    fn reasoning_label(effort: ReasoningEffort) -> &'static str {
        match effort {
            ReasoningEffort::XHigh => code_i18n::tr_plain("tui.reasoning_effort.xhigh"),
            ReasoningEffort::High => code_i18n::tr_plain("tui.reasoning_effort.high"),
            ReasoningEffort::Medium => code_i18n::tr_plain("tui.reasoning_effort.medium"),
            ReasoningEffort::Low => code_i18n::tr_plain("tui.reasoning_effort.low"),
            ReasoningEffort::Minimal => code_i18n::tr_plain("tui.reasoning_effort.minimal"),
            ReasoningEffort::None => code_i18n::tr_plain("tui.reasoning_effort.none"),
        }
    }

    fn format_model_label(model: &str) -> String {
        let mut parts = Vec::new();
        for (idx, part) in model.split('-').enumerate() {
            if idx == 0 {
                parts.push(part.to_ascii_uppercase());
                continue;
            }
            let mut chars = part.chars();
            let formatted = match chars.next() {
                Some(first) if first.is_ascii_alphabetic() => {
                    let mut s = String::new();
                    s.push(first.to_ascii_uppercase());
                    s.push_str(chars.as_str());
                    s
                }
                Some(first) => {
                    let mut s = String::new();
                    s.push(first);
                    s.push_str(chars.as_str());
                    s
                }
                None => String::new(),
            };
            parts.push(formatted);
        }
        parts.join("-")
    }

    fn cycle_continue_mode(&mut self, forward: bool) {
        self.continue_mode = if forward {
            self.continue_mode.cycle_forward()
        } else {
            self.continue_mode.cycle_backward()
        };
        self.send_update();
    }

    fn toggle_selected(&mut self) {
        match self.selected_index {
            0 => {
                self.app_event_tx.send(AppEvent::ShowAutoDriveModelSelector);
            }
            1 => {
                self.agents_enabled = !self.agents_enabled;
                self.send_update();
            }
            2 => {
                let next = !self.diagnostics_enabled;
                self.set_diagnostics(next);
                self.send_update();
            }
            3 => self.cycle_continue_mode(true),
            _ => {}
        }
    }

    fn render_panel_body(&self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let lines = self.info_lines();
        Paragraph::new(lines)
            .wrap(Wrap { trim: true })
            .style(Style::default().bg(colors::background()).fg(colors::text()))
            .render(area, buf);
    }

    pub(crate) fn render_without_frame(&self, area: Rect, buf: &mut Buffer) {
        self.render_panel_body(area, buf);
    }

    fn close(&mut self) {
        if !self.closing {
            self.closing = true;
            self.app_event_tx.send(AppEvent::CloseAutoDriveSettings);
        }
    }

    fn option_label(&self, index: usize) -> Line<'static> {
        let selected = index == self.selected_index;
        let indicator = if selected { "›" } else { " " };
        let prefix = format!("{indicator} ");
        let (label, enabled) = match index {
            0 => (code_i18n::tr_plain("tui.auto_drive_settings.option.model"), true),
            1 => (
                code_i18n::tr_plain("tui.auto_drive_settings.option.agents_enabled"),
                self.agents_enabled,
            ),
            2 => (
                code_i18n::tr_plain("tui.auto_drive_settings.option.diagnostics_enabled"),
                self.diagnostics_enabled,
            ),
            3 => (
                code_i18n::tr_plain("tui.auto_drive_settings.option.auto_continue_delay"),
                matches!(self.continue_mode, AutoContinueMode::Manual),
            ),
            _ => ("", false),
        };

        let label_style = if selected {
            Style::default()
                .fg(colors::primary())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(colors::text())
        };

        let mut spans = vec![Span::styled(prefix, label_style)];
        match index {
            0 => {
                if self.use_chat_model {
                    spans.push(Span::styled(
                        code_i18n::tr_plain("tui.model_selection.follow_chat.title"),
                        label_style,
                    ));
                    if selected {
                        spans.push(Span::raw(format!(
                            "  {}",
                            code_i18n::tr_plain("tui.auto_drive_settings.enter_to_change")
                        )));
                    }
                } else {
                    let model_label = self.model.trim();
                    let display = if model_label.is_empty() {
                        code_i18n::tr_plain("tui.auto_drive_settings.not_set").to_string()
                    } else {
                        format!(
                            "{} · {}",
                            Self::format_model_label(model_label),
                            Self::reasoning_label(self.model_reasoning)
                        )
                    };
                    spans.push(Span::styled(display, label_style));
                    if selected {
                        spans.push(Span::raw(format!(
                            "  {}",
                            code_i18n::tr_plain("tui.auto_drive_settings.enter_to_change")
                        )));
                    }
                }
            }
            1 | 2 => {
                let checkbox = if enabled { "[x]" } else { "[ ]" };
                spans.push(Span::styled(
                    format!("{checkbox} {label}"),
                    label_style,
                ));
            }
            3 => {
                spans.push(Span::styled(label.to_string(), label_style));
                spans.push(Span::raw("  "));
                spans.push(Span::styled(
                    Self::continue_mode_label(self.continue_mode).to_string(),
                    Style::default()
                        .fg(colors::text_dim())
                        .add_modifier(if selected { Modifier::BOLD } else { Modifier::empty() }),
                ));
            }
            _ => {}
        }

        Line::from(spans)
    }

    fn info_lines(&self) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        lines.push(self.option_label(0));
        lines.push(self.option_label(1));
        lines.push(self.option_label(2));
        lines.push(self.option_label(3));
        lines.push(Line::default());

        let footer_style = Style::default().fg(colors::text_dim());
        lines.push(Line::from(vec![
            Span::styled(
                code_i18n::tr_plain("tui.common.key.enter"),
                Style::default().fg(colors::primary()),
            ),
            Span::styled(
                format!(" {}", code_i18n::tr_plain("tui.auto_drive_settings.footer.select_toggle")),
                footer_style,
            ),
            Span::raw("   "),
            Span::styled("←/→", Style::default().fg(colors::primary())),
            Span::styled(
                format!(" {}", code_i18n::tr_plain("tui.auto_drive_settings.footer.adjust_delay")),
                footer_style,
            ),
            Span::raw("   "),
            Span::styled(
                code_i18n::tr_plain("tui.common.key.esc"),
                Style::default().fg(colors::primary()),
            ),
            Span::styled(format!(" {}", code_i18n::tr_plain("tui.common.close")), footer_style),
            Span::raw("   "),
            Span::styled(
                code_i18n::tr_plain("tui.common.key.ctrl_s"),
                Style::default().fg(colors::primary()),
            ),
            Span::styled(format!(" {}", code_i18n::tr_plain("tui.common.close")), footer_style),
        ]));

        lines
    }

    fn continue_mode_label(mode: AutoContinueMode) -> &'static str {
        match mode {
            AutoContinueMode::Immediate => code_i18n::tr_plain("tui.auto_continue.immediate"),
            AutoContinueMode::TenSeconds => code_i18n::tr_plain("tui.auto_continue.ten_seconds"),
            AutoContinueMode::SixtySeconds => code_i18n::tr_plain("tui.auto_continue.sixty_seconds"),
            AutoContinueMode::Manual => code_i18n::tr_plain("tui.auto_continue.manual"),
        }
    }

    pub fn handle_key_event_direct(&mut self, key_event: KeyEvent) {
        if !matches!(key_event.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
            return;
        }

        if key_event.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key_event.code, KeyCode::Char('s') | KeyCode::Char('S'))
        {
            self.close();
            self.app_event_tx.send(AppEvent::RequestRedraw);
            return;
        }

        match key_event.code {
            KeyCode::Esc => {
                self.close();
                self.app_event_tx.send(AppEvent::RequestRedraw);
            }
            KeyCode::Up => {
                if self.selected_index == 0 {
                    self.selected_index = Self::option_count() - 1;
                } else {
                    self.selected_index -= 1;
                }
                self.app_event_tx.send(AppEvent::RequestRedraw);
            }
            KeyCode::Down => {
                self.selected_index = (self.selected_index + 1) % Self::option_count();
                self.app_event_tx.send(AppEvent::RequestRedraw);
            }
            KeyCode::Left => {
                if self.selected_index == 2 {
                    self.cycle_continue_mode(false);
                    self.app_event_tx.send(AppEvent::RequestRedraw);
                }
            }
            KeyCode::Right => {
                if self.selected_index == 2 {
                    self.cycle_continue_mode(true);
                    self.app_event_tx.send(AppEvent::RequestRedraw);
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.toggle_selected();
                self.app_event_tx.send(AppEvent::RequestRedraw);
            }
            _ => {}
        }
    }

    pub fn is_view_complete(&self) -> bool {
        self.closing
    }
}

impl<'a> BottomPaneView<'a> for AutoDriveSettingsView {
    fn handle_key_event(&mut self, pane: &mut BottomPane<'a>, key_event: KeyEvent) {
        if !matches!(key_event.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
            return;
        }

        if key_event.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key_event.code, KeyCode::Char('s') | KeyCode::Char('S'))
        {
            self.close();
            pane.request_redraw();
            return;
        }

        match key_event.code {
            KeyCode::Esc => {
                self.close();
                pane.request_redraw();
            }
            KeyCode::Up => {
                if self.selected_index == 0 {
                    self.selected_index = Self::option_count() - 1;
                } else {
                    self.selected_index -= 1;
                }
                pane.request_redraw();
            }
            KeyCode::Down => {
                self.selected_index = (self.selected_index + 1) % Self::option_count();
                pane.request_redraw();
            }
            KeyCode::Left => {
                if self.selected_index == 2 {
                    self.cycle_continue_mode(false);
                    pane.request_redraw();
                }
            }
            KeyCode::Right => {
                if self.selected_index == 2 {
                    self.cycle_continue_mode(true);
                    pane.request_redraw();
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.toggle_selected();
                pane.request_redraw();
            }
            _ => {}
        }
    }

    fn desired_height(&self, _width: u16) -> u16 {
        9
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        render_panel(
            area,
            buf,
            code_i18n::tr_plain("tui.settings.panel_title.auto_drive"),
            PanelFrameStyle::bottom_pane(),
            |inner, buf| self.render_panel_body(inner, buf),
        );
    }

    fn update_status_text(&mut self, _text: String) -> ConditionalUpdate {
        ConditionalUpdate::NoRedraw
    }

    fn is_complete(&self) -> bool {
        self.closing
    }
}
