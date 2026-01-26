//! Mode picker overlay key handling.

use super::ChatWidget;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

// Returns true if the key was handled by the mode overlay (or toggled it open/closed).
pub(super) fn handle_mode_key(chat: &mut ChatWidget<'_>, key_event: KeyEvent) -> bool {
    if chat.mode.overlay.is_none() {
        let is_ctrl_l = matches!(
            key_event,
            KeyEvent {
                code: KeyCode::Char('l'),
                modifiers: KeyModifiers::CONTROL,
                kind: crossterm::event::KeyEventKind::Press | crossterm::event::KeyEventKind::Repeat,
                ..
            }
        ) || matches!(
            key_event,
            KeyEvent {
                code: KeyCode::Char('\u{c}'),
                modifiers,
                kind: crossterm::event::KeyEventKind::Press | crossterm::event::KeyEventKind::Repeat,
                ..
            } if modifiers.is_empty()
        );
        if is_ctrl_l {
            chat.show_mode_popup();
            return true;
        }
        return false;
    }

    let Some(ref mut overlay) = chat.mode.overlay else {
        return false;
    };

    match key_event.code {
        KeyCode::Up => {
            overlay.selected = overlay.selected.saturating_sub(1);
            chat.request_redraw();
            true
        }
        KeyCode::Down => {
            overlay.selected = (overlay.selected + 1).min(1);
            chat.request_redraw();
            true
        }
        KeyCode::Tab | KeyCode::BackTab => {
            overlay.selected = if overlay.selected == 0 { 1 } else { 0 };
            let selection = overlay.current();
            chat.mode.overlay = None;
            chat.apply_mode_selection(selection);
            chat.request_redraw();
            true
        }
        KeyCode::Char('1') => {
            overlay.selected = 0;
            chat.request_redraw();
            true
        }
        KeyCode::Char('2') => {
            overlay.selected = 1;
            chat.request_redraw();
            true
        }
        KeyCode::Enter => {
            let selection = overlay.current();
            chat.mode.overlay = None;
            chat.apply_mode_selection(selection);
            chat.request_redraw();
            true
        }
        KeyCode::Esc => {
            chat.mode.overlay = None;
            chat.request_redraw();
            true
        }
        _ => false,
    }
}
