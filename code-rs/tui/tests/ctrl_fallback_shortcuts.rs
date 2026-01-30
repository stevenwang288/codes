use code_tui::test_helpers::{render_chat_widget_to_vt100, ChatWidgetHarness};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

fn key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent {
        code,
        modifiers,
        kind: crossterm::event::KeyEventKind::Press,
        state: crossterm::event::KeyEventState::empty(),
    }
}

#[test]
fn ctrl_char_fallbacks_open_overlays() {
    // Safe: tests run single-threaded by design in this crate's harness.
    unsafe { std::env::set_var("CODES_LANG", "zh-CN"); }

    let mut harness = ChatWidgetHarness::new();

    // Ctrl+G sometimes arrives as ASCII BEL (0x07) with no modifiers.
    harness.send_key(key(KeyCode::Char('\u{7}'), KeyModifiers::NONE));
    assert!(harness.is_help_overlay_visible());
    let frame = render_chat_widget_to_vt100(&mut harness, 100, 28);
    assert!(frame.contains("快捷键"), "expected help overlay in frame:\n{frame}");

    // Dismiss help overlay.
    harness.send_key(key(KeyCode::Esc, KeyModifiers::NONE));

    // Ctrl+L sometimes arrives as ASCII FF (0x0C) with no modifiers.
    harness.send_key(key(KeyCode::Char('\u{c}'), KeyModifiers::NONE));
    assert!(harness.is_mode_overlay_visible());
    let frame = render_chat_widget_to_vt100(&mut harness, 100, 28);
    assert!(frame.contains("模式"), "expected mode overlay in frame:\n{frame}");

    // Close mode overlay so global shortcuts are routed again.
    harness.send_key(key(KeyCode::Esc, KeyModifiers::NONE));

    // Ctrl+A sometimes arrives as ASCII SOH (0x01) with no modifiers.
    harness.send_key(key(KeyCode::Char('\u{1}'), KeyModifiers::NONE));
    assert!(
        harness.is_agents_terminal_active(),
        "expected agents terminal to activate via ctrl-a fallback"
    );
}
