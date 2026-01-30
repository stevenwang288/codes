use code_tui::test_helpers::{ChatWidgetHarness, SettingsOverlayFocus};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::empty(),
    }
}

#[test]
fn settings_overlay_esc_walks_back_to_chat() {
    // Safe: tests run single-threaded by design in this crate's harness.
    unsafe { std::env::set_var("CODES_LANG", "zh-CN") };

    let mut harness = ChatWidgetHarness::new();
    harness.open_model_settings_overlay();
    assert_eq!(
        harness.settings_overlay_focus(),
        Some(SettingsOverlayFocus::Content),
        "expected Settings to start focused on content"
    );

    // Esc: back to sidebar.
    harness.send_key(key(KeyCode::Esc));
    assert_eq!(
        harness.settings_overlay_focus(),
        Some(SettingsOverlayFocus::Sidebar),
        "expected Esc to return to sidebar"
    );

    // Esc: back to overview menu.
    harness.send_key(key(KeyCode::Esc));
    assert_eq!(
        harness.settings_overlay_focus(),
        Some(SettingsOverlayFocus::Menu),
        "expected Esc to return to overview menu"
    );

    // Esc: close overlay.
    harness.send_key(key(KeyCode::Esc));
    assert!(!harness.is_settings_overlay_visible());
}
