param(
  [Parameter(Mandatory = $true)]
  [string]$RepoRoot,

  [switch]$Force
)

Set-StrictMode -Version Latest

. "${PSScriptRoot}/_lib.ps1"
. "${PSScriptRoot}/config.ps1"

$RepoRoot = Resolve-RepoRoot -RepoRoot $RepoRoot

$codeHome = Resolve-CodeHome -RepoRoot $RepoRoot
New-Item -ItemType Directory -Force -Path $codeHome | Out-Null

$configPath = Join-Path $codeHome "config.toml"
$notifyScript = Join-Path $RepoRoot "KO/TOOLS/patchkit/code/notify/notify.ps1"

if (-not (Test-Path -Path $notifyScript)) {
  throw "Missing notify script: $notifyScript"
}

if (-not (Test-Path -Path $configPath)) {
  New-Item -ItemType File -Force -Path $configPath | Out-Null
}

$raw = Get-Content -Raw -Path $configPath

function Ensure-Line {
  param(
    [Parameter(Mandatory = $true)][string]$Key,
    [Parameter(Mandatory = $true)][string]$Line
  )

  if ($raw -match "(?m)^\s*${Key}\s*=") {
    if ($Force) {
      $script:raw = ($raw -replace "(?m)^\s*${Key}\s*=.*$", $Line)
    }
  } else {
    if (-not $raw.EndsWith("`n") -and $raw.Length -gt 0) { $script:raw += "`n" }
    $script:raw += $Line + "`n"
  }
}

function Ensure-SectionLine {
  param(
    [Parameter(Mandatory = $true)][string]$Section,
    [Parameter(Mandatory = $true)][string]$Key,
    [Parameter(Mandatory = $true)][string]$Line
  )

  $sectionHeader = "\\[${([regex]::Escape($Section))}\\]"
  $sectionPattern = "(?m)^\\s*${sectionHeader}\\s*(?:#.*)?$"

  if ($raw -notmatch $sectionPattern) {
    if (-not $raw.EndsWith("`n") -and $raw.Length -gt 0) { $script:raw += "`n" }
    $script:raw += "[${Section}]`n"
    $script:raw += $Line + "`n"
    return
  }

  $blockPattern = "(?ms)^(\\s*${sectionHeader}\\s*(?:#.*)?$)(?<body>.*?)(?=^\\s*\\[|\\z)"
  $blockMatch = [regex]::Match($raw, $blockPattern)
  if (-not $blockMatch.Success) {
    return
  }

  $body = $blockMatch.Groups["body"].Value
  if ($body -match "(?m)^\\s*${Key}\\s*=") {
    if (-not $Force) {
      return
    }
    $newBody = ($body -replace "(?m)^\\s*${Key}\\s*=.*$", $Line)
  } else {
    $newBody = $body
    if (-not $newBody.EndsWith("`n") -and $newBody.Length -gt 0) { $newBody += "`n" }
    $newBody += $Line + "`n"
  }

  $script:raw = $raw.Substring(0, $blockMatch.Index) + $blockMatch.Groups[1].Value + $newBody + $raw.Substring($blockMatch.Index + $blockMatch.Length)
}

Write-Section "Ensure config: $configPath"

# 1) Unify HOME roots for this workspace usage.
Ensure-Line -Key "auto_drive_observer_cadence" -Line "auto_drive_observer_cadence = 3"

# 2) OS-level notify hook (independent of TUI).
$notifyLine = 'notify = ["pwsh", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", "' + ($notifyScript -replace "\\", "/") + '"]'
Ensure-Line -Key "notify" -Line $notifyLine

# 3) Prefer built-in TUI notifications when the terminal supports it.
Ensure-SectionLine -Section "tui" -Key "notifications" -Line 'notifications = ["agent-turn-complete", "approval-requested"]'

[System.IO.File]::WriteAllText(
  $configPath,
  $raw,
  [System.Text.UTF8Encoding]::new($false)
)

Write-Host "[codes-patchkit] Updated config." -ForegroundColor Green
