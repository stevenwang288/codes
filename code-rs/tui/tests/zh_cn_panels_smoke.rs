use code_tui::test_helpers::{render_chat_widget_to_vt100, ChatWidgetHarness};
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
fn ctrl_panels_are_chinese_by_default() {
    // Safe: tests run single-threaded by design in this crate's harness.
    unsafe { std::env::set_var("CODES_LANG", "zh-CN") };

    let mut harness = ChatWidgetHarness::new();

    // Ctrl+G: help/guide overlay.
    harness.send_key(key(KeyCode::Char('g'), KeyModifiers::CONTROL));
    assert!(harness.is_help_overlay_visible());
    let frame = render_chat_widget_to_vt100(&mut harness, 100, 28);
    assert!(frame.contains("快捷键"), "expected help overlay title in zh-CN:\n{frame}");
    assert!(
        frame.contains("输入框"),
        "expected help overlay sections in zh-CN:\n{frame}"
    );

    // Close help overlay.
    harness.send_key(key(KeyCode::Esc, KeyModifiers::NONE));

    // Ctrl+L: mode overlay.
    harness.send_key(key(KeyCode::Char('l'), KeyModifiers::CONTROL));
    assert!(harness.is_mode_overlay_visible());
    let frame = render_chat_widget_to_vt100(&mut harness, 100, 28);
    assert!(frame.contains("模式"), "expected mode overlay title in zh-CN:\n{frame}");
    assert!(
        frame.contains("开发模式") || frame.contains("严格模式"),
        "expected mode overlay options in zh-CN:\n{frame}"
    );

    // Close mode overlay.
    harness.send_key(key(KeyCode::Esc, KeyModifiers::NONE));

    // Ctrl+A: agents terminal.
    harness.send_key(key(KeyCode::Char('a'), KeyModifiers::CONTROL));
    assert!(
        harness.is_agents_terminal_active(),
        "expected agents terminal to activate via Ctrl+A"
    );
    let frame = render_chat_widget_to_vt100(&mut harness, 100, 28);
    assert!(frame.contains("代理"), "expected agents panel title in zh-CN:\n{frame}");
}
