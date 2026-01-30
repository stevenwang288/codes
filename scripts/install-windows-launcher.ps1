param(
  [string]$BinDir = "$HOME/bin"
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
$src = Join-Path $repo "codes.cmd"
if (-not (Test-Path $src)) { throw "missing: $src" }

New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
$dst = Join-Path $BinDir "codes.cmd"
Copy-Item -Force $src $dst

Write-Host ("[install] installed: {0}" -f $dst)
Write-Host ("[install] PATH check: run `where.exe codes` and ensure {0} is first" -f $dst)
