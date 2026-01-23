Set-StrictMode -Version Latest

. "${PSScriptRoot}/_lib.ps1"

function Read-PatchKitConfig {
  param(
    [Parameter(Mandatory = $true)]
    [string]$RepoRoot
  )

  $patchKitRoot = Join-Path $RepoRoot "KO/TOOLS/patchkit/code"
  $cfgPath = Join-Path $patchKitRoot "patchkit.json"
  if (-not (Test-Path -Path $cfgPath)) {
    return $null
  }

  $raw = Get-Content -Raw -Path $cfgPath
  try {
    return ($raw | ConvertFrom-Json)
  } catch {
    throw "Invalid JSON in $cfgPath"
  }
}

function Resolve-UpstreamRemote {
  param(
    [Parameter(Mandatory = $true)]
    [string]$RepoRoot
  )

  $cfg = Read-PatchKitConfig -RepoRoot $RepoRoot
  if ($cfg -and $cfg.upstream -and $cfg.upstream.remote) {
    return [string]$cfg.upstream.remote
  }

  Push-Location $RepoRoot
  try {
    $remotes = @(git remote)
    if ($LASTEXITCODE -ne 0) { throw "git remote failed" }
    if ($remotes -contains 'upstream') { return 'upstream' }
    if ($remotes -contains 'origin') { return 'origin' }
    if ($remotes.Length -gt 0) { return [string]$remotes[0] }
  } finally {
    Pop-Location
  }

  throw "No git remotes found. Configure KO/TOOLS/patchkit/code/patchkit.json"
}

function Resolve-UpstreamBranch {
  param(
    [Parameter(Mandatory = $true)]
    [string]$RepoRoot,
    [Parameter(Mandatory = $true)]
    [string]$Remote
  )

  $cfg = Read-PatchKitConfig -RepoRoot $RepoRoot
  if ($cfg -and $cfg.upstream -and $cfg.upstream.branch) {
    return [string]$cfg.upstream.branch
  }

  Push-Location $RepoRoot
  try {
    $sym = git symbolic-ref "refs/remotes/$Remote/HEAD" 2>$null
    if ($LASTEXITCODE -eq 0 -and $sym) {
      $s = ($sym | Out-String).Trim()
      if ($s -match "refs/remotes/$Remote/(?<b>.+)$") {
        return $Matches['b']
      }
    }
  } finally {
    Pop-Location
  }

  return 'main'
}

function Resolve-CodeHome {
  param(
    [Parameter(Mandatory = $true)]
    [string]$RepoRoot
  )

  $cfg = Read-PatchKitConfig -RepoRoot $RepoRoot
  if ($cfg -and $cfg.paths -and $cfg.paths.codeHome) {
    return (Join-Path $RepoRoot ([string]$cfg.paths.codeHome))
  }
  if ($env:USERPROFILE) {
    return (Join-Path $env:USERPROFILE ".codes")
  }
  return (Join-Path $RepoRoot ".codes")
}

function Resolve-I18nLogPath {
  param(
    [Parameter(Mandatory = $true)]
    [string]$RepoRoot
  )

  $cfg = Read-PatchKitConfig -RepoRoot $RepoRoot
  if ($cfg -and $cfg.paths -and $cfg.paths.i18nLog) {
    return (Join-Path $RepoRoot ([string]$cfg.paths.i18nLog))
  }
  $home = Resolve-CodeHome -RepoRoot $RepoRoot
  return (Join-Path $home "i18n-missing.jsonl")
}
