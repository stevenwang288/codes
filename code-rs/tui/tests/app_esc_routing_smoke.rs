use code_tui::test_helpers::ChatWidgetHarness;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

fn key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent {
        code,
        modifiers,
        kind: KeyEventKind::Press,
        state: KeyEventState::empty(),
    }
}

#[test]
fn app_level_esc_closes_mode_overlay_before_global_policy() {
    // Safe: tests run single-threaded by design in this crate's harness.
    unsafe { std::env::set_var("CODES_LANG", "zh-CN") };

    let mut harness = ChatWidgetHarness::new();

    harness.send_key(key(KeyCode::Char('l'), KeyModifiers::CONTROL));
    assert!(harness.is_mode_overlay_visible());

    // App-level routing: Esc should close the overlay (not trigger global backtrack/undo).
    harness.send_app_key(key(KeyCode::Esc, KeyModifiers::NONE));
    assert!(!harness.is_mode_overlay_visible());
}

#[test]
fn app_level_esc_closes_browser_overlay_before_global_policy() {
    // Safe: tests run single-threaded by design in this crate's harness.
    unsafe { std::env::set_var("CODES_LANG", "zh-CN") };

    let mut harness = ChatWidgetHarness::new();

    harness.send_key(key(KeyCode::Char('b'), KeyModifiers::CONTROL));
    assert!(harness.is_browser_overlay_visible());

    harness.send_app_key(key(KeyCode::Esc, KeyModifiers::NONE));
    assert!(!harness.is_browser_overlay_visible());
}

#[test]
fn app_level_esc_prefers_closing_slash_popup_over_clearing_composer() {
    // Safe: tests run single-threaded by design in this crate's harness.
    unsafe { std::env::set_var("CODES_LANG", "zh-CN") };

    let mut harness = ChatWidgetHarness::new();

    // Type "/" to open the slash popup.
    harness.send_key(key(KeyCode::Char('/'), KeyModifiers::NONE));
    assert!(
        harness.is_composer_popup_visible(),
        "expected slash popup to be visible after typing '/'"
    );
    assert_eq!(harness.composer_text(), "/");

    // App-level routing: Esc should close the popup and keep the input intact.
    harness.send_app_key(key(KeyCode::Esc, KeyModifiers::NONE));
    assert!(!harness.is_composer_popup_visible());
    assert_eq!(harness.composer_text(), "/");
}

