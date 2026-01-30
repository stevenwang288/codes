use super::ChatWidget;

impl ChatWidget<'_> {
    pub(crate) fn has_active_modal_view(&self) -> bool {
        // Treat bottom‑pane views (approval, selection popups) and top‑level overlays
        // (diff viewer, help overlay) as "modals" for Esc routing. This ensures that
        // a single Esc keypress closes the visible overlay instead of engaging the
        // global Esc policy (clear input / backtrack).
        self.bottom_pane.has_active_modal_view()
            || self.settings.overlay.is_some()
            || self.diffs.overlay.is_some()
            || self.help.overlay.is_some()
            || self.lang.overlay.is_some()
            || self.mode.overlay.is_some()
            || self.terminal.overlay.is_some()
            || self.browser_overlay_visible
    }
}
