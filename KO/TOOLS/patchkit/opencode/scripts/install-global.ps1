# Purpose:
# - Install "op" / "opencode" launchers globally (user scope).
# - The launchers always run the latest locally built binary via PatchKit env injection.
#
# What it does:
# - Copies the locally built binary to XDG data bin: ~/.local/share/opencode/bin/opencode.real.exe
# - Writes launchers: op.cmd, OP.cmd, opencode.cmd, op.ps1 (and optional aliases)
# - Adds that bin dir to the *User* PATH (prepended) if missing.

param(
  [Parameter(Mandatory = $true)]
  [string]$RepoRoot
)

$ErrorActionPreference = "Stop"

function Get-XdgDataHome {
  if ($env:XDG_DATA_HOME) {
    return $env:XDG_DATA_HOME
  }
  return (Join-Path $env:USERPROFILE ".local/share")
}

function Get-XdgConfigHome {
  if ($env:XDG_CONFIG_HOME) {
    return $env:XDG_CONFIG_HOME
  }
  return (Join-Path $env:USERPROFILE ".config")
}

$xdgDataHome = Get-XdgDataHome
$xdgConfigHome = Get-XdgConfigHome

$binDir = Join-Path $xdgDataHome "opencode/bin"
$npmBin = Join-Path $env:APPDATA "npm"
$exeBuilt = Join-Path $RepoRoot "packages/opencode/dist/opencode-windows-x64/bin/opencode.exe"
$exeReal = Join-Path $binDir "opencode.real.exe"

if (!(Test-Path -LiteralPath $exeBuilt)) {
  throw "Built binary not found: $exeBuilt (run PatchKit build first)"
}

New-Item -ItemType Directory -Force -Path $binDir | Out-Null
if ($npmBin) { New-Item -ItemType Directory -Force -Path $npmBin | Out-Null }
Copy-Item -Force -LiteralPath $exeBuilt -Destination $exeReal

$opPs1 = Join-Path $binDir "op.ps1"
$opCmd = Join-Path $binDir "op.cmd"
$opCmdUpper = Join-Path $binDir "OP.cmd"
$opencodeCmd = Join-Path $binDir "opencode.cmd"

$agentsPath = Join-Path $xdgConfigHome "opencode/AGENTS.md"
$configDir = Join-Path $xdgConfigHome "opencode"
$dataDir = Join-Path $xdgDataHome "opencode"

$opPs1Template = @'
param(
  [Parameter(ValueFromRemainingArguments = $true)]
  [string[]]$Args
)

$ErrorActionPreference = "Stop"

$env:OPENCODE_DISABLE_CLAUDE_CODE = "1"
$env:OPENCODE_DISABLE_CLAUDE_CODE_PROMPT = "1"
$env:OPENCODE_DISABLE_CLAUDE_CODE_SKILLS = "1"
$env:OPENCODE_LANGUAGE = "zh-CN"

Write-Host "[op] 注入摘要" -ForegroundColor Cyan
Write-Host ("  - cwd: " + (Get-Location).Path)
Write-Host "  - OPENCODE_LANGUAGE=zh-CN"
Write-Host "  - OPENCODE_DISABLE_CLAUDE_CODE=1 (prompt+skills)"
Write-Host ("  - config: __CONFIG_DIR__")
Write-Host ("  - data:   __DATA_DIR__")

$agentsPath = "__AGENTS_PATH__"
$agentsStatus = " (missing)"
if (Test-Path -LiteralPath $agentsPath) { $agentsStatus = " (found)" }
Write-Host ("  - AGENTS.md: " + $agentsPath + $agentsStatus)

& "__EXE_REAL__" @Args
'@

$opPs1Content = $opPs1Template
$opPs1Content = $opPs1Content.Replace("__CONFIG_DIR__", $configDir)
$opPs1Content = $opPs1Content.Replace("__DATA_DIR__", $dataDir)
$opPs1Content = $opPs1Content.Replace("__AGENTS_PATH__", $agentsPath)
$opPs1Content = $opPs1Content.Replace("__EXE_REAL__", $exeReal)

Set-Content -LiteralPath $opPs1 -Value $opPs1Content -Encoding UTF8

# Also install launchers into npm bin, since it's commonly already on PATH in the current session.
if ($npmBin) {
  Set-Content -LiteralPath (Join-Path $npmBin "op.ps1") -Value $opPs1Content -Encoding UTF8
}

$cmdBody = @'
@echo off
set OPENCODE_DISABLE_CLAUDE_CODE=1
set OPENCODE_DISABLE_CLAUDE_CODE_PROMPT=1
set OPENCODE_DISABLE_CLAUDE_CODE_SKILLS=1
set OPENCODE_LANGUAGE=zh-CN
pwsh -NoProfile -ExecutionPolicy Bypass -File "%~dp0op.ps1" %*
'@
Set-Content -LiteralPath $opCmd -Value $cmdBody -Encoding ASCII
Set-Content -LiteralPath $opCmdUpper -Value $cmdBody -Encoding ASCII
Set-Content -LiteralPath $opencodeCmd -Value $cmdBody -Encoding ASCII

if ($npmBin) {
  Set-Content -LiteralPath (Join-Path $npmBin "op.cmd") -Value $cmdBody -Encoding ASCII
  Set-Content -LiteralPath (Join-Path $npmBin "OP.cmd") -Value $cmdBody -Encoding ASCII
  Set-Content -LiteralPath (Join-Path $npmBin "opencode.cmd") -Value $cmdBody -Encoding ASCII
}

# Ensure user PATH contains binDir (prepend)
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if (!$userPath) { $userPath = "" }
if ($userPath -notmatch [Regex]::Escape($binDir)) {
  $newUserPath = ($binDir + ";" + $userPath).TrimEnd(';')
  [Environment]::SetEnvironmentVariable("Path", $newUserPath, "User")
}

# Update current process PATH too
if ($env:Path -notmatch [Regex]::Escape($binDir)) {
  $env:Path = $binDir + ";" + $env:Path
}

Write-Host "[opencode-patchkit] 已安装到全局（用户级 PATH）" -ForegroundColor Green
Write-Host ("  - bin: " + $binDir)
Write-Host "  - 命令: op / OP / opencode"
