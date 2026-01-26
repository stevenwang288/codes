use code_tui::test_helpers::{render_chat_widget_to_vt100, ChatWidgetHarness};

#[test]
fn lang_overlay_shows_choices() {
    // Safe: tests run single-threaded by design in this crate's harness.
    unsafe { std::env::set_var("CODEX_LANG", "zh-CN"); }

    let mut harness = ChatWidgetHarness::new();
    harness.open_lang_overlay();
    assert!(harness.is_lang_overlay_visible());

    let frame = render_chat_widget_to_vt100(&mut harness, 100, 28);
    assert!(
        frame.contains("语言"),
        "expected lang overlay title in frame:\n{frame}"
    );
    assert!(
        frame.contains("简体中文"),
        "expected zh-CN choice in frame:\n{frame}"
    );
    assert!(
        frame.contains("English"),
        "expected en choice in frame:\n{frame}"
    );
}
