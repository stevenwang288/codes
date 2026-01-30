# TUI Alternate Screen and Terminal Multiplexers

## Overview

This document explains the design decision behind CODES's alternate screen handling, particularly in terminal multiplexers like Zellij. This addresses a fundamental conflict between fullscreen TUI behavior and terminal scrollback history preservation.

## The Problem

### Fullscreen TUI Benefits

CODES's TUI uses the terminal's **alternate screen buffer** to provide a clean fullscreen experience. This approach:

- Uses the entire viewport without polluting the terminal's scrollback history
- Provides a dedicated environment for the chat interface
- Mirrors the behavior of other terminal applications (vim, tmux, etc.)

### The Zellij Conflict

Terminal multiplexers like **Zellij** strictly follow the xterm specification, which defines that alternate screen buffers should **not** have scrollback. This is intentional design, not a bug:

- **Zellij PR:** https://github.com/zellij-org/zellij/pull/1032
- **Rationale:** The xterm spec explicitly states that alternate screen mode disallows scrollback
- **Configurability:** This is not configurable in Zellij—there is no option to enable scrollback in alternate screen mode

When using CODES's TUI in Zellij, users cannot scroll back through the conversation history because:

1. The TUI runs in alternate screen mode (fullscreen)
2. Zellij disables scrollback in alternate screen buffers (per xterm spec)
3. The entire conversation becomes inaccessible via normal terminal scrolling

## The Solution

CODES supports two rendering modes, controlled by `tui.alternate_screen` in `~/.codes/config.toml`:

### 1) Alternate screen (`tui.alternate_screen = true`, default)

- Fullscreen TUI.
- Keeps your terminal scrollback clean.
- Terminal multiplexers may disable scrollback in this mode (e.g., Zellij).

### 2) Standard buffer (`tui.alternate_screen = false`)

- Runs the TUI without the alternate screen.
- Preserves terminal scrollback.
- Trade-off: the UI output lives in your terminal scrollback.

## Runtime toggle

Press `Ctrl+T` inside the TUI to toggle between alternate-screen and standard-buffer mode. The preference is persisted to `~/.codes/config.toml`.

## Implementation Details

### Toggle + persistence

- Key binding: `Ctrl+T` toggles between alternate-screen and standard-buffer mode.
- Handler: `code-rs/tui/src/app/terminal.rs::toggle_screen_mode`.
- Persistence: `code-rs/core/src/config/sources.rs::set_tui_alternate_screen` writes `[tui].alternate_screen = true/false` into `~/.codes/config.toml`.

### Configuration schema

```toml
[tui]
alternate_screen = true  # default
```

## Related Issues and References

- **Original Issue:** [GitHub #2558](https://github.com/openai/codex/issues/2558) - "No scrollback in Zellij"
- **Implementation PR:** [GitHub #8555](https://github.com/openai/codex/pull/8555)
- **Zellij PR:** https://github.com/zellij-org/zellij/pull/1032 (why scrollback is disabled)
- **xterm Spec:** Alternate screen buffers should not have scrollback

## Future Considerations

### Alternative Approaches Considered

1. **Implement custom scrollback in TUI:** Would require significant architectural changes to buffer and render all historical output
2. **Request Zellij to add a config option:** Not viable—Zellij maintainers explicitly chose this behavior to follow the spec
3. **Disable alternate screen unconditionally:** Would degrade UX for non-Zellij users

### Transcript Pager

If you need terminal scrollback (e.g., inside Zellij), switch to standard-buffer mode (`Ctrl+T`) so your terminal/multiplexer can retain history.

## For Developers

When modifying TUI code, remember:

- Toggle lives in `code-rs/tui/src/app/terminal.rs`.
- Configuration is `config.tui.alternate_screen` (bool).
- Terminal glue is in `code-rs/tui/src/tui.rs` (`enter_alt_screen_only` / `leave_alt_screen_only`).

If you encounter issues with terminal state after running CODES, you can restore your terminal with:

```bash
reset
```
