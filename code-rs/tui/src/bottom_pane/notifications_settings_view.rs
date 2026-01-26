#![allow(dead_code)]

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Widget};

use crate::app_event::AppEvent;
use crate::app_event_sender::AppEventSender;
use crate::chatwidget::BackgroundOrderTicket;

use super::bottom_pane_view::BottomPaneView;
use super::BottomPane;

#[derive(Clone)]
pub(crate) enum NotificationsMode {
    Toggle { enabled: bool },
    Custom { entries: Vec<String> },
}

pub(crate) struct NotificationsSettingsView {
    mode: NotificationsMode,
    app_event_tx: AppEventSender,
    ticket: BackgroundOrderTicket,
    selected_row: usize,
    is_complete: bool,
}

impl NotificationsSettingsView {
    pub fn new(
        mode: NotificationsMode,
        app_event_tx: AppEventSender,
        ticket: BackgroundOrderTicket,
    ) -> Self {
        Self {
            mode,
            app_event_tx,
            ticket,
            selected_row: 0,
            is_complete: false,
        }
    }

    fn toggle(&mut self) {
        match &mut self.mode {
            NotificationsMode::Toggle { enabled } => {
                *enabled = !*enabled;
                self.app_event_tx
                    .send(AppEvent::UpdateTuiNotifications(*enabled));
            }
            NotificationsMode::Custom { entries } => {
                let filters = if entries.is_empty() {
                    code_i18n::tr_plain("tui.common.none").to_string()
                } else {
                    entries.join(", ")
                };
                self.app_event_tx.send_background_event_with_ticket(
                    &self.ticket,
                    code_i18n::tr_args(
                        code_i18n::current_language(),
                        "tui.notifications.custom_filters",
                        &[("filters", &filters)],
                    ),
                );
                self.app_event_tx.send_background_event_with_ticket(
                    &self.ticket,
                    code_i18n::tr_plain("tui.notifications.custom_hint").to_string(),
                );
            }
        }
    }

    fn status_line(&self) -> Line<'static> {
        match &self.mode {
            NotificationsMode::Toggle { enabled } => {
                let status = if *enabled {
                    code_i18n::tr_plain("tui.common.enabled")
                } else {
                    code_i18n::tr_plain("tui.common.disabled")
                };
                let color = if *enabled {
                    crate::colors::success()
                } else {
                    crate::colors::warning()
                };
                Line::from(vec![
                    Span::styled(
                        code_i18n::tr_plain("tui.common.status_prefix"),
                        Style::default().fg(crate::colors::text_dim()),
                    ),
                    Span::styled(status, Style::default().fg(color).add_modifier(Modifier::BOLD)),
                ])
            }
            NotificationsMode::Custom { entries } => {
                let filters = if entries.is_empty() {
                    code_i18n::tr_plain("tui.common.none").to_string()
                } else {
                    entries.join(", ")
                };
                Line::from(vec![
                    Span::styled(
                        code_i18n::tr_plain("tui.common.status_prefix"),
                        Style::default().fg(crate::colors::text_dim()),
                    ),
                    Span::styled(
                        code_i18n::tr_plain("tui.notifications.custom_filter"),
                        Style::default()
                            .fg(crate::colors::info())
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("  "),
                    Span::styled(filters, Style::default().fg(crate::colors::dim())),
                ])
            }
        }
    }

    fn toggle_line(&self) -> Line<'static> {
        match &self.mode {
            NotificationsMode::Toggle { enabled } => {
                let label = if *enabled {
                    code_i18n::tr_plain("tui.common.enabled")
                } else {
                    code_i18n::tr_plain("tui.common.disabled")
                };
                Line::from(vec![
                    Span::styled(
                        format!("{}: ", code_i18n::tr_plain("tui.settings.section.notifications")),
                        Style::default().fg(crate::colors::text_dim()),
                    ),
                    Span::raw(label),
                ])
            }
            NotificationsMode::Custom { .. } => {
                Line::from(vec![
                    Span::styled(
                        code_i18n::tr_plain("tui.notifications.managed_by_config"),
                        Style::default().fg(crate::colors::text()),
                    ),
                ])
            }
        }
    }

    fn process_key_event(&mut self, key_event: KeyEvent) {
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

    pub(crate) fn handle_key_event_direct(&mut self, key_event: KeyEvent) -> bool {
        let handled = matches!(
            key_event,
            KeyEvent { code: KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right | KeyCode::Enter | KeyCode::Esc, .. }
                | KeyEvent { code: KeyCode::Char(' '), modifiers: KeyModifiers::NONE, .. }
        );
        self.process_key_event(key_event);
        handled
    }
}

impl<'a> BottomPaneView<'a> for NotificationsSettingsView {
    fn handle_key_event(&mut self, _pane: &mut BottomPane<'a>, key_event: KeyEvent) {
        self.process_key_event(key_event);
    }

    fn is_complete(&self) -> bool {
        self.is_complete
    }

    fn desired_height(&self, _width: u16) -> u16 {
        9
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(crate::colors::border()))
            .style(Style::default().bg(crate::colors::background()).fg(crate::colors::text()))
            .title(" Notifications ")
            .title_alignment(Alignment::Center);
        let inner = block.inner(area);
        block.render(area, buf);

        let mut lines = Vec::new();
        lines.push(self.status_line());
        lines.push(Line::from(""));
        let mut toggle_line = self.toggle_line();
        if self.selected_row == 0 {
            toggle_line = toggle_line
                .style(
                    Style::default()
                        .bg(crate::colors::selection())
                        .add_modifier(Modifier::BOLD),
                );
        }
        lines.push(toggle_line);
        lines.push(Line::from(""));
        let mut close_line = Line::from(vec![
            Span::raw(if self.selected_row == 1 { "> " } else { "  " }),
            Span::raw(code_i18n::tr_plain("tui.common.close_label")),
        ]);
        if self.selected_row == 1 {
            close_line = close_line
                .style(
                    Style::default()
                        .bg(crate::colors::selection())
                        .add_modifier(Modifier::BOLD),
                );
        }
        lines.push(close_line);
        lines.push(Line::from(""));

        let footer = match &self.mode {
            NotificationsMode::Toggle { .. } => Line::from(vec![
                Span::styled("Up/Down", Style::default().fg(crate::colors::light_blue())),
                Span::raw(format!(" {}  ", code_i18n::tr_plain("tui.common.navigate"))),
                Span::styled("Left/Right or Space", Style::default().fg(crate::colors::success())),
                Span::raw(format!(" {}  ", code_i18n::tr_plain("tui.common.toggle"))),
                Span::styled("Enter", Style::default().fg(crate::colors::success())),
                Span::raw(format!(
                    " {}  ",
                    code_i18n::tr_plain("tui.notifications.toggle_or_close")
                )),
                Span::styled("Esc", Style::default().fg(crate::colors::error())),
                Span::raw(format!(" {}", code_i18n::tr_plain("tui.common.cancel"))),
            ]),
            NotificationsMode::Custom { .. } => Line::from(vec![
                Span::styled(
                    code_i18n::tr_plain("tui.notifications.custom_hint"),
                    Style::default().fg(crate::colors::text_dim()),
                ),
            ]),
        };
        lines.push(footer);

        let paragraph = Paragraph::new(lines)
            .alignment(Alignment::Left)
            .style(Style::default().bg(crate::colors::background()).fg(crate::colors::text()));
        paragraph.render(
            Rect {
                x: inner.x.saturating_add(1),
                y: inner.y,
                width: inner.width.saturating_sub(2),
                height: inner.height,
            },
            buf,
        );
    }
}
