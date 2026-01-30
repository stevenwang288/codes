Set-StrictMode -Version Latest

function Resolve-RepoRoot {
  param(
    [Parameter(Mandatory = $true)]
    [string]$RepoRoot
  )

  $resolved = (Resolve-Path -Path $RepoRoot).Path
  return $resolved
}

function Assert-GitRepo {
  param(
    [Parameter(Mandatory = $true)]
    [string]$RepoRoot
  )

  if (-not (Test-Path -Path (Join-Path $RepoRoot ".git"))) {
    throw "Not a git repository (missing .git): $RepoRoot"
  }
}

function Get-PatchDir {
  param(
    [Parameter(Mandatory = $true)]
    [string]$RepoRoot
  )

  return (Join-Path $RepoRoot "KO/TOOLS/patchkit/code/patches")
}

function Get-PatchFiles {
  param(
    [Parameter(Mandatory = $true)]
    [string]$RepoRoot
  )

  $patchDir = Get-PatchDir -RepoRoot $RepoRoot
  if (-not (Test-Path -Path $patchDir)) {
    return @()
  }

  return @(Get-ChildItem -Path $patchDir -File -Filter "*.patch" | Sort-Object Name)
}

function Invoke-Bash {
  param(
    [Parameter(Mandatory = $true)]
    [string]$RepoRoot,

    [Parameter(Mandatory = $true)]
    [string]$Command
  )

  $bashExe = "C:/Program Files/Git/usr/bin/bash.exe"
  if (-not (Test-Path -Path $bashExe)) {
    $bashExe = "bash.exe"
  }

  # Always normalize paths for Git Bash.
  $escaped = $RepoRoot.Replace('"', '\"')
  $bootstrap = @(
    'repo_posix="$(cygpath -u "' + $escaped + '")"',
    'cd "${repo_posix}"',
    'export CODES_HOME="${HOME}/.codes"',
    'mkdir -p "$CODES_HOME"',
    $Command
  ) -join '; '

  & $bashExe -lc $bootstrap
  if ($LASTEXITCODE -ne 0) {
    throw "bash command failed with exit code $($LASTEXITCODE): $Command"
  }
}

function Write-Section {
  param([string]$Title)
  Write-Host "[codes-patchkit] $Title" -ForegroundColor Cyan
}
