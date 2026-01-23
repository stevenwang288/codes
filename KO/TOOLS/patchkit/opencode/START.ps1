# 一键入口：apply patches ->（智能 build）-> run（强制中文 + 禁 Claude 注入）
# 用法：
#   pwsh -ExecutionPolicy Bypass -File "./tools/opencode-patchkit/START.ps1" -RepoRoot "D:/path/to/opencode"

param(
  [Parameter(ValueFromRemainingArguments = $true)]
  [string[]]$RemainingArgs
)

$ErrorActionPreference = "Stop"

$Here = Split-Path -Parent $PSCommandPath
$Script = Join-Path $Here "scripts/start.ps1"

pwsh -NoProfile -ExecutionPolicy Bypass -File $Script @RemainingArgs
