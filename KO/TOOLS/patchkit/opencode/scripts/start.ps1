# Purpose:
# - Fast local workflow for an external opencode checkout:
#   apply patches -> (smart build) -> (optional install) -> run
# - Default behavior is optimized for speed:
#   - does NOT run cleanup-legacy
#   - only builds if outputs are missing or inputs changed
#
# Usage:
#   pwsh -ExecutionPolicy Bypass -File "./tools/opencode-patchkit/scripts/start.ps1"
#   pwsh -ExecutionPolicy Bypass -File "./tools/opencode-patchkit/scripts/start.ps1" -RepoRoot "D:/path/to/opencode" -- tui
#
# Switches:
#   -Build        Force rebuild
#   -Install      Force reinstall global launchers
#   -CleanupLegacy  Remove old conflicting shims (slow; runs npm checks)

param(
  [string]$RepoRoot,
  [switch]$Build,
  [switch]$Install,
  [switch]$CleanupLegacy,
  [switch]$NoRun,

  [Parameter(ValueFromRemainingArguments = $true)]
  [string[]]$Args
)

$ErrorActionPreference = "Stop"

$ScriptRoot = Split-Path -Parent $PSCommandPath
. (Join-Path $ScriptRoot "_lib.ps1")

$RepoRoot = Resolve-OpencodeRepoRoot -RepoRoot $RepoRoot

if ($CleanupLegacy) {
  & (Join-Path $ScriptRoot "cleanup-legacy.ps1") -WithNpmUninstall
}

# 1) Apply patches
& (Join-Path $ScriptRoot "apply-patches.ps1") -RepoRoot $RepoRoot

# 2) Smart build (skip when nothing relevant changed)
$builtExe = Get-OpencodeBuiltExePath $RepoRoot
$currentStamp = Get-OpencodeBuildStamp -RepoRoot $RepoRoot
$db = Read-StampDb
$prevStamp = $db[$RepoRoot]

$needsBuild = $Build -or !(Test-Path -LiteralPath $builtExe) -or ($prevStamp -ne $currentStamp)
if ($needsBuild) {
  Push-Location (Join-Path $RepoRoot "packages/opencode")
  try {
    # For zip checkouts (no .git), avoid scripts trying to infer a git branch.
    $env:OPENCODE_CHANNEL = $env:OPENCODE_CHANNEL ?? "dev"
    bun run build
  } finally {
    Pop-Location
  }

  $db[$RepoRoot] = $currentStamp
  Write-StampDb $db

  # Installing launchers only matters when the built binary changed.
  $Install = $true
}

# 2.5) Optional global install
if ($Install) {
  & (Join-Path $ScriptRoot "install-global.ps1") -RepoRoot $RepoRoot
}

if ($NoRun) {
  return
}

# 3) Run built exe (preferred), fallback to PATH opencode
$exe = (Test-Path -LiteralPath $builtExe) ? $builtExe : "opencode"
& (Join-Path $ScriptRoot "run-opencode.ps1") -OpencodeExe $exe @Args
