use code_tui::test_helpers::{render_chat_widget_to_vt100, ChatWidgetHarness};
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
fn settings_model_overlay_left_returns_to_overview() {
    // Safe: tests run single-threaded by design in this crate's harness.
    unsafe { std::env::set_var("CODES_LANG", "zh-CN") };

    let mut harness = ChatWidgetHarness::new();
    harness.open_model_settings_overlay();
    assert!(harness.is_settings_overlay_visible());

    let in_model = render_chat_widget_to_vt100(&mut harness, 100, 28);
    assert!(
        in_model.contains("选择模型与推理"),
        "expected model settings title in frame:\n{in_model}"
    );

    harness.send_key(key(KeyCode::Left));
    let in_overview = render_chat_widget_to_vt100(&mut harness, 100, 28);
    assert!(
        in_overview.contains("概览"),
        "expected settings overview after Left:\n{in_overview}"
    );
}
