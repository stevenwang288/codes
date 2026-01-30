param(
  # When set, installs without building (expects code-rs/bin/codes.exe already present).
  [switch]$NoBuild = $false
)

$ErrorActionPreference = "Stop"

function Resolve-RepoRoot {
  $p = (Split-Path -Parent $PSScriptRoot)
  while ($true) {
    if (Test-Path (Join-Path $p "code-rs")) { return $p }
    $parent = Split-Path $p -Parent
    if ($parent -eq $p) { throw "repo root not found (code-rs missing)" }
    $p = $parent
  }
}

function Ensure-Dir([string]$Path) {
  New-Item -ItemType Directory -Force -Path $Path | Out-Null
}

function Add-UserPathEntry([string]$Dir) {
  $dirNorm = (Resolve-Path -LiteralPath $Dir).Path.TrimEnd("\")
  $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
  if (-not $userPath) { $userPath = "" }
  $parts = $userPath.Split(";", [System.StringSplitOptions]::RemoveEmptyEntries) | ForEach-Object { $_.TrimEnd("\") }
  $parts = $parts | Where-Object { $_ -ne $dirNorm }

  $newParts = @($dirNorm) + @($parts)
  $newUserPath = ($newParts -join ";")
  [Environment]::SetEnvironmentVariable("Path", $newUserPath, "User")
  $env:Path = "$dirNorm;$env:Path"
  return $true
}

$repoRoot = Resolve-RepoRoot
Set-Location $repoRoot

if (-not $NoBuild) {
  # Build the Windows binary with the native MSVC toolchain.
  # Use the repo-local build script when available; fall back to cargo build.
  $gitBash = "C:/Program Files/Git/usr/bin/bash.exe"
  if (Test-Path $gitBash) {
    & $gitBash -lc "./build-fast.sh" | Out-Host
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
  } else {
    Push-Location "code-rs"
    cargo build --profile dev-fast --bin codes
    Pop-Location
  }
}

$srcExe = Join-Path $repoRoot "code-rs/bin/codes.exe"
if (-not (Test-Path $srcExe)) {
  throw "missing built binary: $srcExe (run scripts/install-codes.ps1 without -NoBuild)"
}

$codesHome = Join-Path $HOME ".codes"
$destBin = Join-Path $codesHome "bin"
Ensure-Dir $destBin

$destExe = Join-Path $destBin "codes.exe"
Copy-Item -Force -LiteralPath $srcExe -Destination $destExe

$destCmd = Join-Path $destBin "codes.cmd"
@"
@echo off
setlocal
"%~dp0codes.exe" %*
"@ | Set-Content -LiteralPath $destCmd -Encoding ASCII

$pathUpdated = Add-UserPathEntry $destBin

Write-Host ("[codes] Installed: {0}" -f $destExe)
Write-Host ("[codes] Installed: {0}" -f $destCmd)
if ($pathUpdated) {
  Write-Host ("[codes] PATH updated for current user: {0}" -f $destBin)
  Write-Host "[codes] Restart your terminal to ensure PATH is refreshed everywhere."
} else {
  Write-Host ("[codes] PATH already contains: {0}" -f $destBin)
}
