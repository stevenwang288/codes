use super::ChatWidget;
use super::LangOption;
use crossterm::event::{KeyCode, KeyEvent};

pub(super) fn handle_lang_key(chat: &mut ChatWidget<'_>, key_event: KeyEvent) -> bool {
    let Some(ref mut overlay) = chat.lang.overlay else {
        return false;
    };

    match key_event.code {
        KeyCode::Up => {
            overlay.selected = overlay.selected.saturating_sub(1);
            chat.request_redraw();
            true
        }
        KeyCode::Down => {
            overlay.selected = (overlay.selected + 1).min(LangOption::all().len().saturating_sub(1));
            chat.request_redraw();
            true
        }
        KeyCode::Char('1') => {
            overlay.selected = 0;
            let language = overlay.current().to_language();
            chat.lang.overlay = None;
            chat.apply_language_selection(language);
            true
        }
        KeyCode::Char('2') => {
            overlay.selected = 1;
            let language = overlay.current().to_language();
            chat.lang.overlay = None;
            chat.apply_language_selection(language);
            true
        }
        KeyCode::Enter => {
            let language = overlay.current().to_language();
            chat.lang.overlay = None;
            chat.apply_language_selection(language);
            true
        }
        KeyCode::Esc => {
            chat.lang.overlay = None;
            chat.request_redraw();
            true
        }
        _ => false,
    }
}

