param(
  [Parameter(Position = 0)][ValidateSet("status", "compare-version", "update-upstream", "merge-i18n", "apply-patch", "build-release", "smoke", "sync")][string]$Command = "status",
  [string]$UpstreamRef = "upstream/main"
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

function Get-GitText([string[]]$GitArgs) {
  $out = & git @GitArgs
  if ($LASTEXITCODE -ne 0) { throw ("git {0} failed" -f ($GitArgs -join " ")) }
  return ($out | Out-String).Trim()
}

function Get-JsonFromGitShow([string]$Spec) {
  $raw = & git show $Spec 2>$null
  if ($LASTEXITCODE -ne 0) { return $null }
  try { return ($raw | Out-String | ConvertFrom-Json) } catch { return $null }
}

function Get-JsonFile([string]$Path) {
  if (-not (Test-Path $Path)) { return $null }
  return (Get-Content $Path -Raw | ConvertFrom-Json)
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

function Invoke-Bash([string]$RepoRoot, [string]$Cmd, [hashtable]$Env = @{}) {
  $bash = Get-GitBashExe
  $repoUnix = Convert-ToMsysPath $RepoRoot
  $prefix = ""
  foreach ($k in $Env.Keys) {
    $v = $Env[$k]
    $prefix += ("export {0}={1}; " -f $k, ("'" + ($v -replace "'", "'\\''") + "'"))
  }
  $bashCmd = ('cd "' + $repoUnix + '"; ' + $prefix + $Cmd)
  & $bash -lc $bashCmd
  return $LASTEXITCODE
}

function Compare-Version([string]$RepoRoot, [string]$Ref) {
  Write-Host ("[sync] fetch {0}" -f $Ref)
  & git fetch upstream main --quiet
  if ($LASTEXITCODE -ne 0) { throw "git fetch upstream failed" }

  $upCommit = Get-GitText -GitArgs @("rev-parse", "--short", $Ref)
  $head = Get-GitText -GitArgs @("rev-parse", "--short", "HEAD")

  $upPkg = Get-JsonFromGitShow ("{0}:codex-cli/package.json" -f $Ref)
  $localPkg = Get-JsonFile (Join-Path $RepoRoot "codex-cli/package.json")

  Write-Host ("[sync] repo HEAD: {0}" -f $head)
  Write-Host ("[sync] upstream : {0} ({1})" -f $Ref, $upCommit)

  if ($upPkg -and $localPkg) {
    Write-Host ("[sync] codex-cli version: local={0} upstream={1}" -f $localPkg.version, $upPkg.version)
  } elseif ($upPkg) {
    Write-Host ("[sync] codex-cli upstream version: {0}" -f $upPkg.version)
  } else {
    Write-Host "[sync] codex-cli upstream version: (unavailable)"
  }
}

function Update-Upstream([string]$Ref) {
  Write-Host ("[sync] update snapshot from {0} -> codex-rs/, codex-cli/" -f $Ref)
  & git fetch upstream main --quiet
  if ($LASTEXITCODE -ne 0) { throw "git fetch upstream failed" }

  & git checkout $Ref -- codex-rs codex-cli
  if ($LASTEXITCODE -ne 0) { throw "git checkout snapshot failed" }
}

function Merge-I18n([string]$RepoRoot) {
  $base = Join-Path $RepoRoot "code-rs/i18n/assets/en.json"
  $existing = Join-Path $RepoRoot "code-rs/i18n/assets/zh-CN.json"
  if (-not (Test-Path $base)) { throw "missing base: $base" }
  if (-not (Test-Path $existing)) { throw "missing existing: $existing" }

  Write-Host "[sync] merge i18n assets (en -> zh-CN, keep existing translations)"
  & (Join-Path $RepoRoot "scripts/i18n/merge-assets.ps1") -Base $base -Existing $existing
  if ($LASTEXITCODE -ne 0) { throw "merge-assets failed" }
}

function Apply-Patch([string]$RepoRoot) {
  Write-Host "[sync] apply local feature patch (idempotent)"
  & (Join-Path $RepoRoot "scripts/codes-patch/apply.ps1")
  if ($LASTEXITCODE -ne 0) { throw "apply patch failed" }
}

function Build-Release([string]$RepoRoot) {
  Write-Host "[sync] build (PROFILE=release-prod) via build-fast.sh"
  $cacheHome = Convert-ToMsysPath (Join-Path $RepoRoot ".codes-home")
  Invoke-Bash $RepoRoot "./build-fast.sh" @{
    PROFILE = "release-prod"
    CODE_HOME = $cacheHome
    CODEX_HOME = $cacheHome
  } | Out-Host
  if ($LASTEXITCODE -ne 0) { throw ("build-fast.sh failed (rc={0})" -f $LASTEXITCODE) }
}

function Smoke([string]$RepoRoot) {
  Write-Host "[sync] smoke: launcher diagnostics + --version"
  & (Join-Path $RepoRoot "codes.cmd") which
  if ($LASTEXITCODE -ne 0) { throw "codes.cmd which failed" }
  & (Join-Path $RepoRoot "codes.cmd") --version
  if ($LASTEXITCODE -ne 0) { throw "codes.cmd --version failed" }
}

$repo = Resolve-RepoRoot
Set-Location $repo

switch ($Command) {
  "status" {
    Compare-Version $repo $UpstreamRef
    break
  }
  "compare-version" {
    Compare-Version $repo $UpstreamRef
    break
  }
  "update-upstream" {
    Update-Upstream $UpstreamRef
    break
  }
  "merge-i18n" {
    Merge-I18n $repo
    break
  }
  "apply-patch" {
    Apply-Patch $repo
    break
  }
  "build-release" {
    Build-Release $repo
    break
  }
  "smoke" {
    Smoke $repo
    break
  }
  "sync" {
    Compare-Version $repo $UpstreamRef
    Update-Upstream $UpstreamRef
    Apply-Patch $repo
    Merge-I18n $repo
    Build-Release $repo
    Smoke $repo
    break
  }
}
