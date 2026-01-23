# Purpose:
# - Convenience wrapper to run OpenCode with Claude-Code compatibility disabled.
# - Useful when you want the official "no .claude" behavior without patching runtime flags manually.
#
# Usage:
#   pwsh -ExecutionPolicy Bypass -File "./opencode-patchkit/scripts/run-opencode.ps1" <opencode args>
#
# Notes:
# - This script does NOT build OpenCode. It only sets env vars for the current process.

param(
  # Path to the OpenCode executable you want to run.
  # Default assumes `opencode` is available in PATH.
  [string]$OpencodeExe = "opencode",

  [Parameter(ValueFromRemainingArguments = $true)]
  [string[]]$Args
)

$ErrorActionPreference = "Stop"

$env:OPENCODE_DISABLE_CLAUDE_CODE = "1"

# Optional: keep explicit toggles too (in case upstream changes the meaning of the umbrella var).
$env:OPENCODE_DISABLE_CLAUDE_CODE_PROMPT = "1"
$env:OPENCODE_DISABLE_CLAUDE_CODE_SKILLS = "1"

# Force Chinese UI/CLI language (injection-time, no reliance on OS locale)
$env:OPENCODE_LANGUAGE = "zh-CN"

function Get-AgentsPath {
  if ($env:XDG_CONFIG_HOME) {
    return (Join-Path $env:XDG_CONFIG_HOME "opencode/AGENTS.md")
  }
  return (Join-Path $env:USERPROFILE ".config/opencode/AGENTS.md")
}

$agentsPath = Get-AgentsPath

Write-Host "[opencode-patchkit] 注入摘要" -ForegroundColor Cyan
Write-Host ("  - cwd: " + (Get-Location).Path)
Write-Host "  - OPENCODE_LANGUAGE=zh-CN"
Write-Host "  - OPENCODE_DISABLE_CLAUDE_CODE=1 (prompt+skills)"
Write-Host ("  - AGENTS.md: " + $agentsPath + ((Test-Path -LiteralPath $agentsPath) ? " (found)" : " (missing)"))
Write-Host "  - 续接策略: 仅续接当前 cwd 的历史会话（TUI Home 会提示是否恢复）"

& $OpencodeExe @Args
