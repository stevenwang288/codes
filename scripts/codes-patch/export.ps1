param(
  [string]$OutPatch = "patches/codes-features.patch"
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

$files = @(
  "codes.cmd",
  "code-rs/tui/src/theme.rs",
  "code-rs/tui/src/bottom_pane/theme_selection_view.rs",
  "code-rs/core/src/config/sources.rs"
)

$outPath = Join-Path $repo $OutPatch
$outDir = Split-Path $outPath -Parent
New-Item -ItemType Directory -Force -Path $outDir | Out-Null

$diff = & git diff --binary -- @files
if ($LASTEXITCODE -ne 0) { throw "git diff failed" }

$diff | Set-Content -Path $outPath -Encoding UTF8
Write-Host ("[codes-patch] wrote: {0}" -f $outPath)
