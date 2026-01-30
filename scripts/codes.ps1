param(
  [Parameter(ValueFromRemainingArguments = $true)]
  [string[]]$RemainingArgs = @()
)

$ErrorActionPreference = "Stop"

function Resolve-RepoRoot {
  $p = (Get-Location).Path
  while ($true) {
    if (Test-Path (Join-Path $p "build-fast.sh")) { return $p }
    $parent = Split-Path $p -Parent
    if ($parent -eq $p) { throw "repo root not found (build-fast.sh missing)" }
    $p = $parent
  }
}

function Get-GitBashExe {
  if ($env:BASH_EXE -and (Test-Path $env:BASH_EXE)) { return $env:BASH_EXE }
  $default = "C:/Program Files/Git/usr/bin/bash.exe"
  if (Test-Path $default) { return $default }
  throw "Git Bash not found; set BASH_EXE or install Git for Windows"
}

function Convert-ToMsysPath([string]$WindowsPath) {
  $p = $WindowsPath -replace "\\\\", "/"
  if ($p -match "^([A-Za-z]):/(.*)$") {
    return ("/" + $Matches[1].ToLower() + "/" + $Matches[2])
  }
  return $p
}

function Read-FirstLine([string]$Path) {
  if (-not (Test-Path $Path)) { return $null }
  $line = Get-Content -Path $Path -TotalCount 1 -ErrorAction Stop
  if ($null -eq $line) { return $null }
  $trim = $line.Trim()
  if ($trim.Length -eq 0) { return $null }
  return $trim
}

function Invoke-Bash([string]$RepoRoot, [string]$Cmd, [hashtable]$Env = @{}) {
  $bash = Get-GitBashExe
  $repoUnix = Convert-ToMsysPath $RepoRoot
  $prefix = ""
  foreach ($k in $Env.Keys) {
    $v = $Env[$k]
    $prefix += ("export {0}={1}; " -f $k, ("'" + ($v -replace "'", "'\\''") + "'"))
  }
  $bashCmd = ('cd "' + $repoUnix + '"; ' + $prefix + $Cmd)
  & $bash -lc $bashCmd | Out-Host
  return $LASTEXITCODE
}

$repoRoot = Resolve-RepoRoot
Set-Location $repoRoot

$trace = $env:CODES_TRACE -eq "1"
if ($trace) {
  Write-Host ("[codes] pwsh RemainingArgs: {0}" -f ($RemainingArgs -join " | "))
  Write-Host ("[codes] pwsh args: {0}" -f ($args -join " | "))
}

$cacheHome = Join-Path $repoRoot ".codes-home"
New-Item -ItemType Directory -Force -Path $cacheHome | Out-Null
$cacheHomeMsys = Convert-ToMsysPath $cacheHome

$langHome = Join-Path $HOME ".codes"

if (-not $env:CODES_LANG) {
  $lang = Read-FirstLine (Join-Path $langHome "ui-language.txt")
  if ($lang) { $env:CODES_LANG = $lang }
}
if (-not $env:CODES_LANG) { $env:CODES_LANG = "zh-CN" }

if (-not $env:CODE_PALETTE_MODE) { $env:CODE_PALETTE_MODE = "ansi256" }

$env:CODES_AUTO_TRUST = "1"

Remove-Item Env:\CODES_HOME -ErrorAction SilentlyContinue
$env:CODES_HOME = $langHome
$env:CODES_BUILD_HOME = $cacheHomeMsys

if ($RemainingArgs.Count -gt 0 -and $RemainingArgs[0] -ieq "which") {
  Write-Host ("[codes] repo-root: ""{0}""" -f $repoRoot)
  Write-Host ("[codes] CACHE_HOME: ""{0}""" -f $cacheHome)
  Write-Host ("[codes] run-config: global home ""{0}""" -f (Join-Path $HOME ".codes"))
  Write-Host ("[codes] CODES_LANG: ""{0}""" -f $env:CODES_LANG)
  Write-Host ("[codes] CODE_PALETTE_MODE: ""{0}""" -f $env:CODE_PALETTE_MODE)
  $lb = Read-FirstLine (Join-Path $cacheHome "last-built-bin.txt")
  if ($lb) { Write-Host ("[codes] last-built-bin.txt: {0}" -f $lb) } else { Write-Host "[codes] last-built-bin.txt: missing" }
  exit 0
}

$forceRebuild = $false
$forwardArgs = @()
foreach ($a in $RemainingArgs) {
  if ($a -ieq "--rebuild" -or $a -ieq "build") { $forceRebuild = $true; continue }
  $forwardArgs += $a
}

$lastBuilt = Read-FirstLine (Join-Path $cacheHome "last-built-bin.txt")
if (-not $forceRebuild -and $lastBuilt) {
  $cmd = $lastBuilt
  foreach ($a in $forwardArgs) { $cmd += (" " + $a) }
  $rc = Invoke-Bash $repoRoot $cmd @{}
  exit $rc
}

# Build fast by default; keep CODES_BUILD_HOME pointing at the repo cache so we record last-built-bin.txt.
$rcBuild = Invoke-Bash $repoRoot "./build-fast.sh" @{
  CODES_BUILD_HOME = $cacheHomeMsys
}
if ($rcBuild -ne 0) { exit $rcBuild }

$lastBuilt2 = Read-FirstLine (Join-Path $cacheHome "last-built-bin.txt")
if (-not $lastBuilt2) {
  Write-Error "[codes] build succeeded but last-built-bin.txt missing"
  exit 1
}

$cmd2 = $lastBuilt2
foreach ($a in $forwardArgs) { $cmd2 += (" " + $a) }
$rc2 = Invoke-Bash $repoRoot $cmd2 @{}
exit $rc2
