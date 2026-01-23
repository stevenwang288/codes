# 一键入口（短名）：apply patches ->（智能 build）-> run
# 用法：
#   pwsh -ExecutionPolicy Bypass -File "./tools/opencode-patchkit/op.ps1" -RepoRoot "D:/path/to/opencode"

param(
  [Parameter(ValueFromRemainingArguments = $true)]
  [string[]]$RemainingArgs
)

$ErrorActionPreference = "Stop"

$Here = Split-Path -Parent $PSCommandPath
$Script = Join-Path $Here "scripts/start.ps1"

pwsh -NoProfile -ExecutionPolicy Bypass -File $Script @RemainingArgs
