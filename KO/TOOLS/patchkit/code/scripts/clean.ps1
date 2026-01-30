param(
  [Parameter(Mandatory = $true)]
  [string]$RepoRoot,

  [switch]$All
)

Set-StrictMode -Version Latest

. "${PSScriptRoot}/_lib.ps1"
. "${PSScriptRoot}/config.ps1"

$RepoRoot = Resolve-RepoRoot -RepoRoot $RepoRoot

$codeHome = Resolve-CodeHome -RepoRoot $RepoRoot

$targetCache = Join-Path $codeHome "working/_target-cache"
$sessions = Join-Path $codeHome "sessions"
$working = Join-Path $codeHome "working"

Write-Section "Clean"
Write-Host "[codes-patchkit] Home=$codeHome" -ForegroundColor DarkGray

if (Test-Path -Path $targetCache) {
  Write-Host "[codes-patchkit] Removing $targetCache" -ForegroundColor Yellow
  Remove-Item -Recurse -Force $targetCache
}

if ($All) {
  foreach ($p in @($sessions, $working)) {
    if (Test-Path -Path $p) {
      Write-Host "[codes-patchkit] Removing $p" -ForegroundColor Yellow
      Remove-Item -Recurse -Force $p
    }
  }
}

Write-Host "[codes-patchkit] Clean done." -ForegroundColor Green
