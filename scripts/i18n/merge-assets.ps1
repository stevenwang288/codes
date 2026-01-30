param(
  [Parameter(Mandatory = $true)][string]$Base,
  [Parameter(Mandatory = $true)][string]$Existing,
  [string]$Out
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
$script = Join-Path $repo "scripts/i18n/merge-assets.mjs"
if (-not (Test-Path $script)) { throw "missing: $script" }

$args = @(
  $script,
  "--base", $Base,
  "--existing", $Existing
)
if ($Out) { $args += @("--out", $Out) }

& node @args
