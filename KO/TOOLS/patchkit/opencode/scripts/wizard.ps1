# PatchKit interactive workflow.
#
# A step-by-step guide (not fully automatic) to:
# - check updates
# - apply patches
# - show i18n pending stats
# - build
# - install launchers
# - run

param(
  [string]$RepoRoot,
  [switch]$NonInteractive
)

$ErrorActionPreference = "Stop"

$ScriptRoot = Split-Path -Parent $PSCommandPath
. (Join-Path $ScriptRoot "_lib.ps1")

$repo = Resolve-OpencodeRepoRoot -RepoRoot $RepoRoot
$builtExe = Get-OpencodeBuiltExePath $repo

function Confirm([string]$Message) {
  if ($NonInteractive) { return $true }
  $ans = Read-Host ($Message + " (y/N)")
  return $ans -match '^(y|yes)$'
}

Write-Host "[patchkit] Workflow" -ForegroundColor Cyan
Write-Host ("  - opencode repo: " + $repo)
Write-Host ("  - built exe:     " + $builtExe)

$origin = Remember-Upstream $repo
if ($origin) {
  Write-Host ("  - upstream:      " + $origin)
}
Write-Host ""

# Step 1: update (optional)
if (Test-Path -LiteralPath (Join-Path $repo ".git")) {
  try {
    git -C $repo fetch origin --quiet | Out-Null
    $behind = (& git -C $repo rev-list --count HEAD..origin/dev 2>$null)
    if ($behind -match '^\d+$' -and [int]$behind -gt 0) {
      Write-Host ("[patchkit] upstream updates available: " + $behind) -ForegroundColor Yellow
      if (Confirm "Update opencode repo (git pull --ff-only)?") {
        git -C $repo pull --ff-only
      }
    } else {
      Write-Host "[patchkit] opencode repo is up to date" -ForegroundColor Green
    }
  } catch {
    Write-Host "[patchkit] update check failed" -ForegroundColor Yellow
    Write-Host ("  " + ($_ | Out-String).Trim()) -ForegroundColor Yellow
  }
} else {
  Write-Host "[patchkit] no .git found (zip/vendor checkout)" -ForegroundColor Yellow
}

Write-Host ""

# Step 2: apply patches
if (Confirm "Apply PatchKit patches now?") {
  & (Join-Path $ScriptRoot "apply-patches.ps1") -RepoRoot $repo
  Write-Host "[patchkit] patches applied" -ForegroundColor Green
}

Write-Host ""

# Step 3: i18n stats
Write-Host "[patchkit] i18n pending (from .opencode/i18n-missing.json)" -ForegroundColor Cyan
& (Join-Path $ScriptRoot "i18n-stats.ps1") -TargetDir $repo
Write-Host ""

# Step 4: build (smart)
if (Confirm "Build opencode now (smart build)?") {
  & (Join-Path $ScriptRoot "start.ps1") -RepoRoot $repo -Build -NoRun
} else {
  Write-Host "[patchkit] build skipped" -ForegroundColor Yellow
}

Write-Host ""

# Step 5: run
if (Confirm "Run opencode now?") {
  $exe = (Test-Path -LiteralPath $builtExe) ? $builtExe : "opencode"
  & (Join-Path $ScriptRoot "run-opencode.ps1") -OpencodeExe $exe
}
