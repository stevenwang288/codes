//! Language picker overlay key handling.

use super::{ChatWidget, LangOption};
use crossterm::event::{KeyCode, KeyEvent};

// Returns true if the key was handled by the language overlay.
pub(super) fn handle_lang_key(chat: &mut ChatWidget<'_>, key_event: KeyEvent) -> bool {
    let Some(ref mut overlay) = chat.lang.overlay else {
        return false;
    };

    match key_event.code {
        KeyCode::Esc => {
            chat.lang.overlay = None;
            chat.request_redraw();
            true
        }
        KeyCode::Up => {
            overlay.selected = overlay.selected.saturating_sub(1);
            chat.request_redraw();
            true
        }
        KeyCode::Down => {
            let max = LangOption::all().len().saturating_sub(1);
            overlay.selected = (overlay.selected + 1).min(max);
            chat.request_redraw();
            true
        }
        KeyCode::Enter => {
            let choice = overlay.current();
            chat.lang.overlay = None;
            chat.apply_language_selection(choice.to_language());
            chat.request_redraw();
            true
        }
        KeyCode::Char('1') => {
            chat.lang.overlay = None;
            chat.apply_language_selection(LangOption::ZhCn.to_language());
            chat.request_redraw();
            true
        }
        KeyCode::Char('2') => {
            chat.lang.overlay = None;
            chat.apply_language_selection(LangOption::En.to_language());
            chat.request_redraw();
            true
        }
        _ => false,
    }
}

