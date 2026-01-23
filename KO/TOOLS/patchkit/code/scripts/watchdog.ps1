param(
  [Parameter(Mandatory = $true)]
  [string]$RepoRoot,

  [int]$IntervalSeconds = 60,
  [int]$StallSeconds = 180,

  [switch]$Once
)

Set-StrictMode -Version Latest

. "${PSScriptRoot}/_lib.ps1"
. "${PSScriptRoot}/config.ps1"

$RepoRoot = Resolve-RepoRoot -RepoRoot $RepoRoot

if ($IntervalSeconds -le 0) { throw "IntervalSeconds must be > 0" }
if ($StallSeconds -le 0) { throw "StallSeconds must be > 0" }

$codeHome = Resolve-CodeHome -RepoRoot $RepoRoot
$historyPath = Join-Path $codeHome "history.jsonl"
$criticalLog = Join-Path $codeHome "debug_logs/critical.log"
$notifyScript = Join-Path $RepoRoot "KO/TOOLS/patchkit/code/notify/notify.ps1"

function Get-ProgressStamp {
  $stamps = @()
  foreach ($p in @($historyPath, $criticalLog)) {
    if (Test-Path -Path $p) {
      $item = Get-Item -LiteralPath $p
      $stamps += [PSCustomObject]@{
        Path = $p
        LastWriteTimeUtc = $item.LastWriteTimeUtc
        Length = $item.Length
      }
    }
  }
  return $stamps
}

function Notify-Stall {
  param([string]$Details)
  if (-not (Test-Path -Path $notifyScript)) {
    Write-Host "[code-patchkit] notify script missing: $notifyScript" -ForegroundColor Yellow
    return
  }

  $payload = @{
    type = "watchdog-stall"
    input_messages = @($Details)
    'last-assistant-message' = "可能卡住了"
  } | ConvertTo-Json -Compress

  pwsh -NoProfile -ExecutionPolicy Bypass -File $notifyScript $payload | Out-Null
}

Write-Section "Watchdog (interval=${IntervalSeconds}s, stall=${StallSeconds}s)"
Write-Host "[code-patchkit] watching: $historyPath" -ForegroundColor DarkGray
Write-Host "[code-patchkit] watching: $criticalLog" -ForegroundColor DarkGray

$baseline = @{}
$lastProgressUtc = [DateTime]::UtcNow
$lastNotifyUtc = [DateTime]::UtcNow.AddYears(-1)

function Update-Baseline {
  $stamps = Get-ProgressStamp
  foreach ($s in $stamps) {
    $baseline[$s.Path] = $s
  }
}

Update-Baseline

do {
  Start-Sleep -Seconds $IntervalSeconds

  $nowUtc = [DateTime]::UtcNow
  $stamps = Get-ProgressStamp

  $progressed = $false
  foreach ($s in $stamps) {
    $prev = $baseline[$s.Path]
    if ($null -eq $prev) {
      $baseline[$s.Path] = $s
      $progressed = $true
      continue
    }
    if ($s.LastWriteTimeUtc -gt $prev.LastWriteTimeUtc -or $s.Length -ne $prev.Length) {
      $baseline[$s.Path] = $s
      $progressed = $true
    }
  }

  if ($progressed) {
    $lastProgressUtc = $nowUtc
    continue
  }

  $stallFor = ($nowUtc - $lastProgressUtc).TotalSeconds
  if ($stallFor -ge $StallSeconds) {
    # Throttle: don't spam more than once per stall window.
    $sinceNotify = ($nowUtc - $lastNotifyUtc).TotalSeconds
    if ($sinceNotify -ge $StallSeconds) {
      $details = "No new activity for ${stallFor}s. Consider checking the TUI: subagents may be stuck or awaiting auth/approval."
      Notify-Stall -Details $details
      $lastNotifyUtc = $nowUtc
    }
  }
} while (-not $Once)
