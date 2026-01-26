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
fn tab_toggles_mode_when_input_empty() {
    // Safe: tests run single-threaded by design in this crate's harness.
    unsafe { std::env::set_var("CODEX_LANG", "zh-CN") };

    let mut harness = ChatWidgetHarness::new();

    let before = render_chat_widget_to_vt100(&mut harness, 100, 28);
    let before_is_strict = before.contains("模式: 严格") || before.contains("模式：严格");
    let before_is_fast = before.contains("模式: 开发") || before.contains("模式：开发");
    assert!(
        before_is_strict || before_is_fast,
        "expected mode indicator in footer before toggle:\n{before}"
    );

    harness.send_key(key(KeyCode::Tab, KeyModifiers::NONE));
    let after = render_chat_widget_to_vt100(&mut harness, 100, 28);

    if before_is_strict {
        assert!(
            after.contains("模式: 开发") || after.contains("模式：开发"),
            "expected mode to toggle strict->fast:\n{after}"
        );
    } else {
        assert!(
            after.contains("模式: 严格") || after.contains("模式：严格"),
            "expected mode to toggle fast->strict:\n{after}"
        );
    }
}

