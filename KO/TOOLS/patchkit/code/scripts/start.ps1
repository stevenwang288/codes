param(
  [Parameter(Mandatory = $true)]
  [string]$RepoRoot,

  [switch]$StartCodeWindow,
  [switch]$Watchdog,
  [switch]$I18nWatch,

  [int]$WatchdogIntervalSeconds = 60,
  [int]$WatchdogStallSeconds = 180
)

Set-StrictMode -Version Latest

. "${PSScriptRoot}/_lib.ps1"
. "${PSScriptRoot}/config.ps1"

$RepoRoot = Resolve-RepoRoot -RepoRoot $RepoRoot

if (-not ($StartCodeWindow -or $Watchdog -or $I18nWatch)) {
  $StartCodeWindow = $true
  $Watchdog = $true
  $I18nWatch = $true
}

& (Join-Path $PSScriptRoot "ensure-config.ps1") -RepoRoot $RepoRoot
if ($LASTEXITCODE -ne 0) { throw "ensure-config failed (exit $LASTEXITCODE)" }

if ($Watchdog) {
  Write-Section "Start watchdog"
  Start-Process -FilePath "pwsh" -WorkingDirectory $RepoRoot -ArgumentList @(
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
}

if ($I18nWatch) {
  Write-Section "Start i18n sync watch"
  Start-Process -FilePath "pwsh" -WorkingDirectory $RepoRoot -ArgumentList @(
    "-NoProfile",
    "-ExecutionPolicy",
    "Bypass",
    "-File",
    (Join-Path $PSScriptRoot "i18n-sync.ps1"),
    "-RepoRoot",
    $RepoRoot,
    "-Watch"
  ) | Out-Null
}

if ($StartCodeWindow) {
  Write-Section "Start codes"
  $codesCmd = Join-Path $RepoRoot "codes.cmd"
  if (-not (Test-Path -Path $codesCmd)) {
    throw "Missing: $codesCmd"
  }

  Start-Process -FilePath "cmd.exe" -WorkingDirectory $RepoRoot -ArgumentList @(
    "/c",
    "\"$codesCmd\""
  ) | Out-Null
}

Write-Host "[codes-patchkit] start completed." -ForegroundColor Green
