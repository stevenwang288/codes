# Purpose:
# - One command to: apply patches -> (optional) build -> run OpenCode.
# - Always disables Claude Code compatibility via official env vars.
#
# Usage:
#   # Apply patches + run (uses PATH opencode)
#   pwsh -ExecutionPolicy Bypass -File "./opencode-patchkit/scripts/run-all.ps1"
#
#   # Apply patches + build a local Windows binary + run it
#   pwsh -ExecutionPolicy Bypass -File "./opencode-patchkit/scripts/run-all.ps1" -Build

param(
  [switch]$Update,
  [switch]$Build,
  [string]$RepoRoot,
  [string]$OpencodeExe,

  [Parameter(ValueFromRemainingArguments = $true)]
  [string[]]$Args
)

$ErrorActionPreference = "Stop"

$ScriptRoot = Split-Path -Parent $PSCommandPath
. (Join-Path $ScriptRoot "_lib.ps1")

$RepoRoot = Resolve-OpencodeRepoRoot -RepoRoot $RepoRoot

if ($Update) {
  $gitDir = Join-Path $RepoRoot ".git"
  if (Test-Path -LiteralPath $gitDir) {
    Write-Host "[opencode-patchkit] Updating upstream (git pull --ff-only)" -ForegroundColor Cyan
    Push-Location $RepoRoot
    try {
      git pull --ff-only
    } finally {
      Pop-Location
    }
  } else {
    Write-Host "[opencode-patchkit] No .git found, skip update. (This looks like a zip/vendor checkout.)" -ForegroundColor Yellow
  }
}

& (Join-Path $ScriptRoot "apply-patches.ps1") -RepoRoot $RepoRoot

if ($Build) {
  Push-Location (Join-Path $RepoRoot "packages/opencode")
  try {
    # For zip checkouts (no .git), avoid scripts trying to infer a git branch.
    $env:OPENCODE_CHANNEL = $env:OPENCODE_CHANNEL ?? "dev"
    bun run build
  } finally {
    Pop-Location
  }
}

if (!$OpencodeExe) {
  $built = Join-Path $RepoRoot "packages/opencode/dist/opencode-windows-x64/bin/opencode.exe"
  $OpencodeExe = (Test-Path -LiteralPath $built) ? $built : "opencode"
}

& (Join-Path $ScriptRoot "run-opencode.ps1") -OpencodeExe $OpencodeExe @Args
