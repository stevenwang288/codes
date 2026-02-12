use super::ChatWidget;
use super::ModeOption;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub(super) fn handle_mode_key(chat: &mut ChatWidget<'_>, key_event: KeyEvent) -> bool {
    if chat.mode.overlay.is_none() {
        if key_event.modifiers.contains(KeyModifiers::CONTROL) {
            if matches!(key_event.code, KeyCode::Char('l') | KeyCode::Char('L')) {
                chat.show_mode_popup();
                return true;
            }
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
            overlay.selected = (overlay.selected + 1).min(ModeOption::all().len().saturating_sub(1));
            chat.request_redraw();
            true
        }
        KeyCode::Char('1') => {
            overlay.selected = 0;
            let selection = overlay.current();
            chat.mode.overlay = None;
            chat.apply_mode_selection(selection);
            chat.request_redraw();
            true
        }
        KeyCode::Char('2') => {
            overlay.selected = 1;
            let selection = overlay.current();
            chat.mode.overlay = None;
            chat.apply_mode_selection(selection);
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

