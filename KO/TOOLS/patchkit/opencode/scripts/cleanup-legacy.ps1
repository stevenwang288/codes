# Purpose:
# - Remove legacy/conflicting launchers from previous experiments so `op`/`opencode`
#   always resolve to PatchKit-managed versions.
#
# Notes:
# - This script is intentionally conservative: it only removes `op*`/`opencode*`
#   shims from the user npm bin directory.
# - npm global checks are optional and off by default (they can be slow).

param(
  [switch]$WithNpmUninstall
)

$ErrorActionPreference = "Stop"

$npmBin = Join-Path $env:APPDATA "npm"
if (!(Test-Path -LiteralPath $npmBin)) {
  return
}

$targets = @(
  (Join-Path $npmBin "op"),
  (Join-Path $npmBin "op.cmd"),
  (Join-Path $npmBin "OP.cmd"),
  (Join-Path $npmBin "op.ps1"),
  (Join-Path $npmBin "opencode"),
  (Join-Path $npmBin "opencode.cmd"),
  (Join-Path $npmBin "opencode.ps1")
)

$removed = @()
foreach ($t in $targets) {
  if (Test-Path -LiteralPath $t) {
    Remove-Item -LiteralPath $t -Force
    $removed += $t
  }
}

if ($removed.Count -gt 0) {
  Write-Host "[opencode-patchkit] 已清理旧版/冲突启动器（npm bin）" -ForegroundColor Yellow
  foreach ($r in $removed) { Write-Host ("  - removed: " + $r) }
}

if ($WithNpmUninstall) {
  # Optional: remove old npm global package to prevent it from recreating shims.
  try {
    $npm = Get-Command npm -ErrorAction SilentlyContinue
    if ($npm) {
      $out = & npm ls -g --depth=0 opencode-ai 2>$null
      if ($LASTEXITCODE -eq 0 -and $out -match 'opencode-ai@') {
        Write-Host "[opencode-patchkit] 卸载旧版 npm 全局包：opencode-ai" -ForegroundColor Yellow
        & npm uninstall -g opencode-ai | Out-Null
      }
    }
  } catch {
    # ignore
  }
}

# Remove legacy PATH entry if present
try {
  $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
  if ($userPath -and $userPath -match "\\.opencode\\\\bin") {
    $parts = $userPath.Split(';') | Where-Object { $_ -and ($_ -notmatch "\\.opencode\\\\bin") }
    [Environment]::SetEnvironmentVariable("Path", ($parts -join ';'), "User")
  }
} catch {
  # ignore
}
