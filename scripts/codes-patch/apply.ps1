param(
  [string]$Patch = "patches/codes-features.patch"
)

$ErrorActionPreference = "Stop"

function Resolve-RepoRoot {
  $here = (Get-Location).Path
  $p = $here
  while ($true) {
    if (Test-Path (Join-Path $p "build-fast.sh")) { return $p }
    $parent = Split-Path $p -Parent
    if ($parent -eq $p) { throw "repo root not found (build-fast.sh missing)" }
    $p = $parent
  }
}

$repo = Resolve-RepoRoot
Set-Location $repo

$patchPath = Join-Path $repo $Patch
if (-not (Test-Path $patchPath)) { throw "missing patch: $patchPath" }

function Test-FeatureApplied {
  function File-ContainsLiteral([string]$Path, [string]$Needle) {
    if (-not (Test-Path $Path)) { return $false }
    $raw = Get-Content -Path $Path -Raw -ErrorAction Stop
    return $raw.Contains($Needle)
  }

  $checks = @(
    @{ Path = "code-rs/tui/src/theme.rs"; Needle = "CODE_PALETTE_MODE" },
    @{ Path = "code-rs/tui/src/bottom_pane/theme_selection_view.rs"; Needle = "cancel_detail(" },
    @{ Path = "code-rs/core/src/config/sources.rs"; Needle = "CODES_PREFER_CODEX_HOME" },
    @{ Path = "codes.cmd"; Needle = "CODE_PALETTE_MODE" }
  )

  foreach ($c in $checks) {
    $p = Join-Path $repo $c.Path
    if (-not (File-ContainsLiteral $p $c.Needle)) { return $false }
  }

  return $true
}

if (Test-FeatureApplied) {
  Write-Host "[codes-patch] already applied (feature checks)"
  exit 0
}

& git apply --whitespace=nowarn $patchPath
if ($LASTEXITCODE -ne 0) { throw "git apply failed" }

Write-Host "[codes-patch] applied"
