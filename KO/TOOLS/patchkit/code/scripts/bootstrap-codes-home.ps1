param(
  [Parameter(Mandatory = $true)]
  [string]$RepoRoot,

  [switch]$Force
)

Set-StrictMode -Version Latest

. "${PSScriptRoot}/_lib.ps1"
. "${PSScriptRoot}/config.ps1"

$RepoRoot = Resolve-RepoRoot -RepoRoot $RepoRoot

$codesHome = Resolve-CodeHome -RepoRoot $RepoRoot
New-Item -ItemType Directory -Force -Path $codesHome | Out-Null

$dst = Join-Path $codesHome "config.toml"

$srcCandidates = @(
  "$env:USERPROFILE/.codex/config.toml",
  "$env:USERPROFILE/.code/config.toml",
  (Join-Path $RepoRoot ".code-home/config.toml"),
  (Join-Path $RepoRoot ".codes-home/config.toml")
)

$src = $null
foreach ($p in $srcCandidates) {
  if (Test-Path -Path $p) { $src = $p; break }
}

Write-Section "Bootstrap CODES home"
Write-Host "[code-patchkit] CODES_HOME=$codesHome" -ForegroundColor DarkGray

if (-not $src) {
  throw "No source config.toml found to copy. Checked: $($srcCandidates -join ', ')"
}

if ((Test-Path -Path $dst) -and (-not $Force)) {
  Write-Host "[code-patchkit] Exists, skip: $dst" -ForegroundColor Yellow
  Write-Host "[code-patchkit] Use -Force to overwrite." -ForegroundColor Yellow
  exit 0
}

Copy-Item -Force $src $dst
Write-Host "[code-patchkit] Copied config: $src -> $dst" -ForegroundColor Green

# Ensure notify/tui.notifications are correctly placed.
& (Join-Path $PSScriptRoot "ensure-config.ps1") -RepoRoot $RepoRoot -Force:$Force

