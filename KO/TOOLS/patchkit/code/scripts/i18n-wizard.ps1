param(
  [Parameter(Mandatory = $true)]
  [string]$RepoRoot,

  [string]$Runner = "coder",
  [string]$Model = "code-gpt-5.2-codex",
  [ValidateSet("zh-only", "bilingual-tui")]
  [string]$Style = "zh-only",
  [int]$MaxSeconds = 600,

  [switch]$Build,
  [switch]$Restart,

  [ValidateSet("ask", "yes", "no")]
  [string]$Confirm = "ask"
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

function Confirm-Next {
  param([string]$Prompt)
  if ($Confirm -eq 'yes') { return $true }
  if ($Confirm -eq 'no') { return $false }

  while ($true) {
    $ans = Read-Host "$Prompt (y/n)"
    if ($null -eq $ans) { return $false }
    switch ($ans.Trim().ToLowerInvariant()) {
      'y' { return $true }
      'yes' { return $true }
      'n' { return $false }
      'no' { return $false }
      default { Write-Host "请输入 y 或 n" -ForegroundColor Yellow }
    }
  }
}

Write-Section "i18n 向导"
Write-Host "[codes-patchkit] Home=$codeHome" -ForegroundColor DarkGray
Write-Host "[codes-patchkit] log=$logPath" -ForegroundColor DarkGray

if (-not (Test-Path -Path $enPath)) { throw "Missing: $enPath" }
if (-not (Test-Path -Path $zhPath)) { throw "Missing: $zhPath" }

Push-Location $RepoRoot
try {
  Write-Section "Step 1/4: 统计待翻译数量"
  node "KO/TOOLS/i18n-collector/cli.mjs" stats --log "$logPath" --en "$enPath" --zh "$zhPath"
  if ($LASTEXITCODE -ne 0) { throw "stats failed (exit $LASTEXITCODE)" }

  if (-not (Confirm-Next -Prompt "继续：执行翻译并回写（sync --once）？")) {
    Write-Host "[codes-patchkit] 已退出向导。" -ForegroundColor Yellow
    exit 0
  }

  Write-Section "Step 2/4: 翻译并回写"
  & (Join-Path $PSScriptRoot "i18n-sync.ps1") -RepoRoot $RepoRoot -Runner $Runner -Model $Model -Style $Style -MaxSeconds $MaxSeconds
  if ($LASTEXITCODE -ne 0) { throw "i18n-sync failed (exit $LASTEXITCODE)" }

  if ($Build -or (Confirm-Next -Prompt "继续：编译（./build-fast.sh）？")) {
    Write-Section "Step 3/4: 编译"
    & (Join-Path $PSScriptRoot "run.ps1") -RepoRoot $RepoRoot -Build
    if ($LASTEXITCODE -ne 0) { throw "build failed (exit $LASTEXITCODE)" }
  }

  if ($Restart -or (Confirm-Next -Prompt "继续：启动新 codes 窗口（重启到新版本）？")) {
    Write-Section "Step 4/4: 重启"
    & (Join-Path $PSScriptRoot "start.ps1") -RepoRoot $RepoRoot -StartCodeWindow
    if ($LASTEXITCODE -ne 0) { throw "start failed (exit $LASTEXITCODE)" }
    Write-Host "[codes-patchkit] 已启动新窗口；旧窗口请手动关闭。" -ForegroundColor Yellow
  }
} finally {
  Pop-Location
}

Write-Host "[codes-patchkit] 向导完成。" -ForegroundColor Green
