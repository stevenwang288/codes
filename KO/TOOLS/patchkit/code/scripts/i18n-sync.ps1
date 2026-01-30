param(
  [Parameter(Mandatory = $true)]
  [string]$RepoRoot,

  [switch]$Watch,

  [string]$Runner = "coder",

  [string]$Model = "code-gpt-5.2-codex",

  [ValidateSet("zh-only", "bilingual-tui")]
  [string]$Style = "zh-only",

  [int]$MaxSeconds = 600
)

Set-StrictMode -Version Latest

. "${PSScriptRoot}/_lib.ps1"
. "${PSScriptRoot}/config.ps1"

$RepoRoot = Resolve-RepoRoot -RepoRoot $RepoRoot

$codeHome = Resolve-CodeHome -RepoRoot $RepoRoot
$env:CODES_HOME = $codeHome
New-Item -ItemType Directory -Force -Path $codeHome | Out-Null

$logPath = Resolve-I18nLogPath -RepoRoot $RepoRoot
$enPath = Join-Path $RepoRoot "code-rs/i18n/assets/en.json"
$zhPath = Join-Path $RepoRoot "code-rs/i18n/assets/zh-CN.json"

if (-not (Test-Path -Path $enPath)) { throw "Missing: $enPath" }
if (-not (Test-Path -Path $zhPath)) { throw "Missing: $zhPath" }

$args = @(
  "KO/TOOLS/i18n-collector/cli.mjs",
  "sync",
  "--log", $logPath,
  "--en", $enPath,
  "--zh", $zhPath,
  "--runner", $Runner,
  "--model", $Model,
  "--style", $Style,
  "--max-seconds", "$MaxSeconds"
)

if ($Watch) {
  $args += "--watch"
} else {
  $args += "--once"
}

Write-Section "i18n sync ($($Watch ? 'watch' : 'once'))"
Write-Host "[codes-patchkit] Home=$codeHome" -ForegroundColor DarkGray
Write-Host "[codes-patchkit] log=$logPath" -ForegroundColor DarkGray

Push-Location $RepoRoot
try {
  $out = node @args 2>&1
  if ($LASTEXITCODE -ne 0) {
    throw "i18n sync failed (exit $LASTEXITCODE)"
  }

  $outText = ($out | Out-String)
  Write-Host $outText

  $m = [regex]::Match($outText, "apply: updated (?<n>\\d+) keys")
  if ($m.Success) {
    $n = [int]$m.Groups['n'].Value
    if ($n -gt 0) {
      $notifyScript = Join-Path $RepoRoot "KO/TOOLS/patchkit/code/notify/notify.ps1"
      if (Test-Path -Path $notifyScript) {
        $payload = @{
          type = "i18n-sync"
          input_messages = @("Updated $n zh-CN keys")
          'last-assistant-message' = "i18n 回写完成"
        } | ConvertTo-Json -Compress
        pwsh -NoProfile -ExecutionPolicy Bypass -File $notifyScript $payload | Out-Null
      }
    }
  }
} finally {
  Pop-Location
}
