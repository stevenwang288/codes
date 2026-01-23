param(
  [Parameter(Position = 0)]
  [ValidateSet(
    "help",
    "status",
    "test",
    "clean",
    "bootstrap",
    "update",
    "apply",
    "build",
    "restart",
    "start",
    "config",
    "i18n-stats",
    "i18n-sync",
    "i18n-wizard",
    "watchdog"
  )]
  [string]$Cmd = "help",

  [string]$RepoRoot = (Get-Location).Path,

  [ValidateSet("ask", "yes", "no")]
  [string]$Confirm = "ask",

  [string]$Runner = "coder",
  [string]$Model = "code-gpt-5.2-codex",
  [ValidateSet("zh-only", "bilingual-tui")]
  [string]$Style = "zh-only",
  [int]$MaxSeconds = 600,

  [switch]$Fetch,
  [switch]$ApplyPatches,
  [switch]$Build,
  [switch]$StartWatchdog,
  [switch]$I18nWatch,

  [int]$WatchdogIntervalSeconds = 60,
  [int]$WatchdogStallSeconds = 180
)

Set-StrictMode -Version Latest

$root = (Resolve-Path -Path $PSScriptRoot).Path
. (Join-Path $root "scripts/_lib.ps1")
. (Join-Path $root "scripts/config.ps1")

$RepoRoot = Resolve-RepoRoot -RepoRoot $RepoRoot
Assert-GitRepo -RepoRoot $RepoRoot

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

function Invoke-I18nStats {
  $logPath = Resolve-I18nLogPath -RepoRoot $RepoRoot
  $enPath = Join-Path $RepoRoot "code-rs/i18n/assets/en.json"
  $zhPath = Join-Path $RepoRoot "code-rs/i18n/assets/zh-CN.json"
  if (-not (Test-Path -Path $enPath)) { throw "Missing: $enPath" }
  if (-not (Test-Path -Path $zhPath)) { throw "Missing: $zhPath" }
  Push-Location $RepoRoot
  try {
    node "KO/TOOLS/i18n-collector/cli.mjs" stats --log "$logPath" --en "$enPath" --zh "$zhPath" --group-by type
    if ($LASTEXITCODE -ne 0) { throw "i18n stats failed (exit $LASTEXITCODE)" }
  } finally {
    Pop-Location
  }
}

function Show-Status {
  $remote = Resolve-UpstreamRemote -RepoRoot $RepoRoot
  $branch = Resolve-UpstreamBranch -RepoRoot $RepoRoot -Remote $remote

  Push-Location $RepoRoot
  try {
    Write-Section "PatchKit status"
    Write-Host "[code-patchkit] repo: $RepoRoot"
    Write-Host "[code-patchkit] upstream: $remote/$branch"

    $head = (git rev-parse --short HEAD)
    $dirty = (git status --porcelain=v1)
    Write-Host "[code-patchkit] head: $head"
    if ($dirty) {
      Write-Host "[code-patchkit] dirty: yes" -ForegroundColor Yellow
    } else {
      Write-Host "[code-patchkit] dirty: no" -ForegroundColor Green
    }

    if ($Fetch) {
      Write-Section "Fetch remotes"
      git fetch $remote --prune
      if ($LASTEXITCODE -ne 0) { throw "git fetch failed (exit $LASTEXITCODE)" }
    }

    $remoteHead = (git rev-parse --short "refs/remotes/$remote/$branch" 2>$null)
    if ($LASTEXITCODE -eq 0 -and $remoteHead) {
      Write-Host "[code-patchkit] upstream head: $remoteHead"
      $behind = (git rev-list --count "HEAD..refs/remotes/$remote/$branch" 2>$null)
      $ahead = (git rev-list --count "refs/remotes/$remote/$branch..HEAD" 2>$null)
      if ($LASTEXITCODE -eq 0) {
        Write-Host "[code-patchkit] ahead=$ahead behind=$behind"
      }
    }

    Write-Section "Patches"
    $patches = @(Get-PatchFiles -RepoRoot $RepoRoot)
    if ($patches.Length -eq 0) {
      Write-Host "[code-patchkit] patches: none" -ForegroundColor Yellow
    } else {
      Write-Host "[code-patchkit] patches: $($patches.Length)"
      foreach ($p in $patches) {
        git apply --reverse --check --whitespace=nowarn "$($p.FullName)" 2>$null
        if ($LASTEXITCODE -eq 0) {
          Write-Host "  - $($p.Name): applied" -ForegroundColor Green
        } else {
          Write-Host "  - $($p.Name): not-applied" -ForegroundColor Yellow
        }
      }
    }
  } finally {
    Pop-Location
  }

  Write-Section "i18n"
  Invoke-I18nStats
}

function Print-TestInstructions {
  Write-Section "Test"
  $home = "$env:USERPROFILE/.codes"
  $log = "$env:USERPROFILE/.codes/patchkit/i18n-missing.jsonl"
  Write-Host "[code-patchkit] Home: $home"
  Write-Host "[code-patchkit] Log:  $log"
  Write-Host "[code-patchkit] Start: run ./codes (or patchkit start), then open Help/Popular commands." 
  Write-Host "[code-patchkit] After a few actions, run:"
  Write-Host "  node KO/TOOLS/i18n-collector/cli.mjs stats --log \"$log\" --en \"code-rs/i18n/assets/en.json\" --zh \"code-rs/i18n/assets/zh-CN.json\" --group-by type"
}

function Do-UpdateWorkflow {
  $remote = Resolve-UpstreamRemote -RepoRoot $RepoRoot
  $branch = Resolve-UpstreamBranch -RepoRoot $RepoRoot -Remote $remote

  Push-Location $RepoRoot
  try {
    if ($Fetch) {
      Write-Section "Fetch remotes"
      git fetch $remote --prune
      if ($LASTEXITCODE -ne 0) { throw "git fetch failed (exit $LASTEXITCODE)" }
    }

    $behind = (git rev-list --count "HEAD..refs/remotes/$remote/$branch" 2>$null)
    if ($LASTEXITCODE -ne 0) { $behind = 0 }

    Write-Section "Upstream check"
    Write-Host "[code-patchkit] upstream: $remote/$branch"
    Write-Host "[code-patchkit] behind: $behind"

    if ([int]$behind -gt 0) {
      if (-not (Confirm-Next -Prompt "发现上游更新（behind=$behind），是否执行 git pull --ff-only？")) {
        return
      }

      $dirty = (git status --porcelain=v1)
      if ($dirty) {
        throw "Refusing to pull with a dirty working tree. Export a patch first or commit/stash."
      }

      Write-Section "Pull (ff-only)"
      git pull --ff-only
      if ($LASTEXITCODE -ne 0) { throw "git pull --ff-only failed (exit $LASTEXITCODE)" }
    } else {
      Write-Host "[code-patchkit] upstream already up to date." -ForegroundColor Green
    }
  } finally {
    Pop-Location
  }

  if ($ApplyPatches -or (Confirm-Next -Prompt "是否应用 PatchKit 补丁？")) {
    & (Join-Path $root "scripts/apply-patches.ps1") -RepoRoot $RepoRoot
  }

  if (Confirm-Next -Prompt "是否检查 i18n 待翻译并（可选）回写？") {
    Invoke-I18nStats
    if (Confirm-Next -Prompt "是否执行 i18n 向导（翻译→回写→可选编译/重启）？") {
      & (Join-Path $root "scripts/i18n-wizard.ps1") -RepoRoot $RepoRoot -Runner $Runner -Model $Model -Style $Style -MaxSeconds $MaxSeconds -Confirm $Confirm
    }
  }

  if ($Build -or (Confirm-Next -Prompt "是否编译（./build-fast.sh）？")) {
    & (Join-Path $root "scripts/run.ps1") -RepoRoot $RepoRoot -Configure -Apply -Build
  }

  if (Confirm-Next -Prompt "是否启动新 codes 窗口（重启到新版本）？") {
    & (Join-Path $root "scripts/start.ps1") -RepoRoot $RepoRoot -StartCodeWindow
    Write-Host "[code-patchkit] 已启动新窗口；旧窗口请手动关闭。" -ForegroundColor Yellow
  }
}

switch ($Cmd) {
  'help' {
    @(
      'code-patchkit (single entry point)',
      '',
      'Commands:',
      '  status       Show repo/upstream/patch/i18n status (use -Fetch to refresh remotes)',
      '  test         Print quick test instructions for i18n collection',
      '  update       Wizard: check upstream -> pull -> apply -> i18n -> build -> restart',
      '  config       Ensure .codes-home/config.toml hooks (notify/tui.notifications)',
      '  apply        Apply patch files under KO/TOOLS/patchkit/code/patches/',
      '  build        Run ./build-fast.sh (bash) via PatchKit',
      '  start        Start codes + optional watchdog + i18n watch',
      '  restart      Start a new codes window',
      '  i18n-stats   Show pending i18n counts and type grouping',
      '  i18n-sync    Run i18n sync once',
      '  i18n-wizard  Interactive i18n workflow wizard',
      '  watchdog     Start watchdog (monitors .codes-home activity)',
      '',
      'Examples:',
      '  pwsh -ExecutionPolicy Bypass -File ./KO/TOOLS/patchkit/code/patchkit.ps1 status -Fetch',
      '  pwsh -ExecutionPolicy Bypass -File ./KO/TOOLS/patchkit/code/patchkit.ps1 update -Fetch',
      '  pwsh -ExecutionPolicy Bypass -File ./KO/TOOLS/patchkit/code/patchkit.ps1 start -I18nWatch -StartWatchdog',
      ''
    ) -join "`n" | Write-Host
  }
  'status' { Show-Status }
  'test' { Print-TestInstructions }
  'clean' { & (Join-Path $root "scripts/clean.ps1") -RepoRoot $RepoRoot }
  'bootstrap' { & (Join-Path $root "scripts/bootstrap-codes-home.ps1") -RepoRoot $RepoRoot }
  'update' { Do-UpdateWorkflow }
  'config' { & (Join-Path $root "scripts/ensure-config.ps1") -RepoRoot $RepoRoot }
  'apply' { & (Join-Path $root "scripts/apply-patches.ps1") -RepoRoot $RepoRoot }
  'build' { & (Join-Path $root "scripts/run.ps1") -RepoRoot $RepoRoot -Build }
  'restart' { & (Join-Path $root "scripts/start.ps1") -RepoRoot $RepoRoot -StartCodeWindow }
  'start' {
    & (Join-Path $root "scripts/start.ps1") -RepoRoot $RepoRoot -Watchdog:$StartWatchdog -I18nWatch:$I18nWatch -StartCodeWindow
  }
  'i18n-stats' { Invoke-I18nStats }
  'i18n-sync' {
    & (Join-Path $root "scripts/i18n-sync.ps1") -RepoRoot $RepoRoot -Runner $Runner -Model $Model -Style $Style -MaxSeconds $MaxSeconds
  }
  'i18n-wizard' {
    & (Join-Path $root "scripts/i18n-wizard.ps1") -RepoRoot $RepoRoot -Runner $Runner -Model $Model -Style $Style -MaxSeconds $MaxSeconds -Confirm $Confirm
  }
  'watchdog' {
    & (Join-Path $root "scripts/watchdog.ps1") -RepoRoot $RepoRoot -IntervalSeconds $WatchdogIntervalSeconds -StallSeconds $WatchdogStallSeconds
  }
}
