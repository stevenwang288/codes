param(
  [Parameter(Mandatory = $true)]
  [string]$RepoRoot,

  [switch]$Configure,
  [switch]$Update,
  [switch]$Apply,
  [switch]$Build,

  [switch]$StartWatchdog,

  [int]$WatchdogIntervalSeconds = 60,
  [int]$WatchdogStallSeconds = 180
)

Set-StrictMode -Version Latest

. "${PSScriptRoot}/_lib.ps1"
. "${PSScriptRoot}/config.ps1"

$RepoRoot = Resolve-RepoRoot -RepoRoot $RepoRoot
Assert-GitRepo -RepoRoot $RepoRoot

if (-not ($Update -or $Apply -or $Build)) {
  $Configure = $true
  $Apply = $true
  $Build = $true
}

Push-Location $RepoRoot
try {
  if ($Update) {
    Write-Section "Update upstream (ff-only)"
    $dirty = git status --porcelain=v1
    if ($LASTEXITCODE -ne 0) { throw "git status failed" }
    if ($dirty) {
      throw "Refusing to update with a dirty working tree. Commit/stash or export a patch first."
    }
    git pull --ff-only
    if ($LASTEXITCODE -ne 0) {
      throw "git pull --ff-only failed (exit $LASTEXITCODE)."
    }
  }

  if ($Configure) {
    & (Join-Path $PSScriptRoot "ensure-config.ps1") -RepoRoot $RepoRoot
    if ($LASTEXITCODE -ne 0) {
      throw "ensure-config failed (exit $LASTEXITCODE)"
    }
  }

  if ($Apply) {
    & (Join-Path $PSScriptRoot "apply-patches.ps1") -RepoRoot $RepoRoot
    if ($LASTEXITCODE -ne 0) {
      throw "apply-patches failed (exit $LASTEXITCODE)"
    }
  }

  if ($Build) {
    Write-Section "Build (./build-fast.sh)"
    Invoke-Bash -RepoRoot $RepoRoot -Command "./build-fast.sh"
  }

  if ($StartWatchdog) {
    Write-Section "Start watchdog"
    Start-Process -FilePath "pwsh" -ArgumentList @(
      "-NoProfile",
      "-ExecutionPolicy",
      "Bypass",
      "-File",
      (Join-Path $PSScriptRoot "watchdog.ps1"),
      "-RepoRoot",
      $RepoRoot,
      "-IntervalSeconds",
      "$WatchdogIntervalSeconds",
      "-StallSeconds",
      "$WatchdogStallSeconds"
    ) | Out-Null
    Write-Host "[codes-patchkit] Watchdog started." -ForegroundColor Green
  }
} finally {
  Pop-Location
}

Write-Host "[codes-patchkit] run completed." -ForegroundColor Green
