use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Widget};

use crate::app_event::AppEvent;
use crate::app_event_sender::AppEventSender;

use super::bottom_pane_view::BottomPaneView;
use super::BottomPane;

/// Interactive UI for GitHub workflow monitoring settings.
/// Shows token status and allows toggling the watcher on/off.
pub(crate) struct GithubSettingsView {
    watcher_enabled: bool,
    token_status: String,
    token_ready: bool,
    app_event_tx: AppEventSender,
    is_complete: bool,
    /// Selection index: 0 = toggle, 1 = close
    selected_row: usize,
}

impl GithubSettingsView {
    pub fn new(watcher_enabled: bool, token_status: String, ready: bool, app_event_tx: AppEventSender) -> Self {
        Self {
            watcher_enabled,
            token_status,
            token_ready: ready,
            app_event_tx,
            is_complete: false,
            selected_row: 0,
        }
    }

    fn toggle(&mut self) {
        self.watcher_enabled = !self.watcher_enabled;
        self.app_event_tx
            .send(AppEvent::UpdateGithubWatcher(self.watcher_enabled));
    }

    pub fn handle_key_event_direct(&mut self, key_event: KeyEvent) {
        match key_event {
            KeyEvent { code: KeyCode::Up, modifiers: KeyModifiers::NONE, .. } => {
                if self.selected_row > 0 {
                    self.selected_row -= 1;
                }
            }
            KeyEvent { code: KeyCode::Down, modifiers: KeyModifiers::NONE, .. } => {
                if self.selected_row < 1 {
                    self.selected_row += 1;
                }
            }
            KeyEvent { code: KeyCode::Left | KeyCode::Right, modifiers: KeyModifiers::NONE, .. } => {
                if self.selected_row == 0 {
                    self.toggle();
                }
            }
            KeyEvent { code: KeyCode::Enter, modifiers: KeyModifiers::NONE, .. } => {
                if self.selected_row == 0 {
                    self.toggle();
                } else {
                    self.is_complete = true;
                }
            }
            KeyEvent { code: KeyCode::Char(' '), modifiers: KeyModifiers::NONE, .. } => {
                if self.selected_row == 0 {
                    self.toggle();
                }
            }
            KeyEvent { code: KeyCode::Esc, .. } => {
                self.is_complete = true;
            }
            _ => {}
        }
    }

    pub fn is_view_complete(&self) -> bool {
        self.is_complete
    }
}

impl<'a> BottomPaneView<'a> for GithubSettingsView {
    fn handle_key_event(&mut self, _pane: &mut BottomPane<'a>, key_event: KeyEvent) {
        self.handle_key_event_direct(key_event);
    }

    fn is_complete(&self) -> bool { self.is_complete }

    fn desired_height(&self, _width: u16) -> u16 { 9 }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);
        let ui_language = code_i18n::current_language();
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(crate::colors::border()))
            .style(Style::default().bg(crate::colors::background()).fg(crate::colors::text()))
            .title(format!(" {} ", code_i18n::tr(ui_language, "tui.github_settings.title")))
            .title_alignment(Alignment::Center);
        let inner = block.inner(area);
        block.render(area, buf);

        let status_line = if self.token_ready {
            Line::from(vec![
                Span::styled(
                    code_i18n::tr(ui_language, "tui.common.status_prefix"),
                    Style::default().fg(crate::colors::text_dim()),
                ),
                Span::styled(
                    code_i18n::tr(ui_language, "tui.state.ready"),
                    Style::default()
                        .fg(crate::colors::success())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled(&self.token_status, Style::default().fg(crate::colors::dim())),
            ])
        } else {
            Line::from(vec![
                Span::styled(
                    code_i18n::tr(ui_language, "tui.common.status_prefix"),
                    Style::default().fg(crate::colors::text_dim()),
                ),
                Span::styled(
                    code_i18n::tr(ui_language, "tui.github_settings.no_token"),
                    Style::default()
                        .fg(crate::colors::warning())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled(
                    code_i18n::tr(ui_language, "tui.github_settings.no_token_hint"),
                    Style::default().fg(crate::colors::dim()),
                ),
            ])
        };

        let toggle_label = if self.watcher_enabled {
            code_i18n::tr(ui_language, "tui.common.enabled")
        } else {
            code_i18n::tr(ui_language, "tui.common.disabled")
        };
        let mut toggle_style = Style::default().fg(crate::colors::text());
        if self.selected_row == 0 { toggle_style = toggle_style.bg(crate::colors::selection()).add_modifier(Modifier::BOLD); }

        let lines = vec![
            status_line,
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    code_i18n::tr(ui_language, "tui.github_settings.workflow_monitoring"),
                    Style::default().fg(crate::colors::text_dim()),
                ),
                Span::styled(toggle_label, toggle_style),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(if self.selected_row == 1 { "› " } else { "  " }, Style::default()),
                Span::styled(
                    code_i18n::tr(ui_language, "tui.common.close_label"),
                    if self.selected_row == 1 {
                        Style::default()
                            .bg(crate::colors::selection())
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    },
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("↑↓", Style::default().fg(crate::colors::light_blue())),
                Span::raw(format!(" {}  ", code_i18n::tr(ui_language, "tui.common.navigate"))),
                Span::styled("←→/Space", Style::default().fg(crate::colors::success())),
                Span::raw(format!(" {}  ", code_i18n::tr(ui_language, "tui.common.toggle"))),
                Span::styled("Enter", Style::default().fg(crate::colors::success())),
                Span::raw(format!(
                    " {}/{}  ",
                    code_i18n::tr(ui_language, "tui.common.toggle"),
                    code_i18n::tr(ui_language, "tui.common.close_label")
                )),
                Span::styled("Esc", Style::default().fg(crate::colors::error())),
                Span::raw(format!(" {}", code_i18n::tr(ui_language, "tui.common.cancel"))),
            ]),
        ];

        let paragraph = Paragraph::new(lines)
            .alignment(Alignment::Left)
            .style(Style::default().bg(crate::colors::background()).fg(crate::colors::text()));
        paragraph.render(Rect { x: inner.x.saturating_add(1), y: inner.y, width: inner.width.saturating_sub(2), height: inner.height }, buf);
    }
}
