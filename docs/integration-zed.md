# Zed Integration

To point Zed at CODES’s ACP server, add this block to `settings.json`:

```jsonc
{
  "agent_servers": {
    "CODES": {
      "command": "codes",
      "args": ["acp"]
    }
  }
}
```

Adjust the `command` or `args` only if you pin a different version or use an absolute path to your `codes` binary.

## Zed prerequisites

- Zed Stable `0.201.5` (released August 27, 2025) or newer adds ACP support with the Agent Panel. Update via `Zed → Check for Updates` before wiring CODES in. Zed’s docs call out ACP as the mechanism powering Gemini CLI and other external agents.
- External agents live inside the Agent Panel (`cmd-?`). Use the `+` button to start a new thread and pick `CODES` from the external agent list. Zed runs our CLI as a subprocess over JSON‑RPC, so all prompts and diff previews stay local.

## How CODES implements ACP

- The Rust MCP server exposes ACP tools: `session/new`, `session/prompt`, and fast interrupts via `session/cancel`. These are backed by the same conversation manager that powers the TUI, so approvals, confirm guards, and sandbox policies remain intact.
- Streaming `session/update` notifications bridge Code events into Zed. You get Answer/Reasoning updates, shell command progress, approvals, and apply_patch diffs in the Zed UI without losing terminal parity.
- MCP configuration stays centralized in `CODES_HOME/config.toml` (defaults to `~/.codes/config.toml`). Use `[experimental_client_tools]` to delegate file read/write and permission requests back to Zed when you want its UI to handle approvals. A minimal setup looks like:

```toml
[experimental_client_tools]
request_permission = { mcp_server = "zed", tool_name = "requestPermission" }
read_text_file = { mcp_server = "zed", tool_name = "readTextFile" }
write_text_file = { mcp_server = "zed", tool_name = "writeTextFile" }
```

Zed wires these tools automatically when you add the CODES agent, so the identifiers above match the defaults.
- The CLI entry point (`codes acp`) is a thin wrapper over the Rust binary (`cargo run -p code-mcp-server -- --stdio`) that ships alongside the rest of CODES. Build-from-source workflows plug in by swapping `command` for an absolute path to that binary.

## Tips and troubleshooting

- Need to inspect the handshake? Run Zed’s `dev: open acp logs` command from the Command Palette; the log shows JSON‑RPC requests and Code replies.
- If prompts hang, make sure no other process is bound to the same MCP port and that your `CODES_HOME` points to the intended config directory. The ACP server inherits all of CODES’s sandbox settings, so restrictive policies (e.g., `approval_policy = "never"`) still apply.
- Zed currently skips history restores and checkpoint UI for third-party agents. Stick to the TUI if you rely on those features; ACP support is still evolving upstream.
- After a session starts, the model selector in Zed lists CODES’s built-in presets (e.g., `gpt-5.1-codex`, `gpt-5.1` high/medium/low). Picking a new preset updates the running CODES session immediately, so you don’t have to restart the agent to change models.
