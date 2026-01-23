# Shared helpers for PatchKit scripts

$ErrorActionPreference = "Stop"

function Resolve-PatchKitRoot {
  # scripts/_lib.ps1 -> scripts -> patchkit root
  return (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
}

function Resolve-CodeMainRoot {
  $patchKitRoot = Resolve-PatchKitRoot
  return (Resolve-Path (Join-Path $patchKitRoot "../..")).Path
}

function Test-IsOpencodeRepoRoot([string]$Path) {
  if (!$Path) { return $false }
  if (!(Test-Path -LiteralPath $Path)) { return $false }
  $pkg = Join-Path $Path "packages/opencode/package.json"
  return (Test-Path -LiteralPath $pkg)
}

function Resolve-OpencodeRepoRoot {
  param(
    [string]$RepoRoot
  )

  if ($RepoRoot) {
    $resolved = (Resolve-Path $RepoRoot).Path
    if (!(Test-IsOpencodeRepoRoot $resolved)) {
      throw "Not an opencode repo root: $resolved (expected packages/opencode/package.json)"
    }
    return $resolved
  }

  if ($env:OPENCODE_REPO_ROOT) {
    $resolved = (Resolve-Path $env:OPENCODE_REPO_ROOT).Path
    if (Test-IsOpencodeRepoRoot $resolved) { return $resolved }
  }

  $patchKitRoot = Resolve-PatchKitRoot
  $configPath = Join-Path $patchKitRoot "patchkit.json"
  if (Test-Path -LiteralPath $configPath) {
    try {
      $cfg = Get-Content -Raw -LiteralPath $configPath | ConvertFrom-Json
      if ($cfg.opencodeRepoRoot) {
        $resolved = (Resolve-Path $cfg.opencodeRepoRoot).Path
        if (Test-IsOpencodeRepoRoot $resolved) { return $resolved }
      }
    } catch {
      # ignore malformed config, fall through to auto-discovery
    }
  }

  # Auto-discovery: sibling folder next to code-main
  $codeMainRoot = Resolve-CodeMainRoot
  $parent = Split-Path -Parent $codeMainRoot

  $candidates = @(
    (Join-Path $parent "opencode/opencode-dev"),
    (Join-Path $parent "opencode/opencode-dev-1.1.32"),
    (Join-Path $parent "opencode"),
    (Join-Path $codeMainRoot "third_party/opencode"),
    (Join-Path $codeMainRoot "third_party/opencode-dev")
  )

  foreach ($c in $candidates) {
    if (Test-IsOpencodeRepoRoot $c) {
      return (Resolve-Path $c).Path
    }
  }

  throw (
    "Unable to locate opencode repo root." +
      " Set OPENCODE_REPO_ROOT or create tools/opencode-patchkit/patchkit.json with { opencodeRepoRoot: '...'}"
  )
}

function Get-OpencodeBuiltExePath([string]$RepoRoot) {
  return (Join-Path $RepoRoot "packages/opencode/dist/opencode-windows-x64/bin/opencode.exe")
}

function Get-StateDir {
  $patchKitRoot = Resolve-PatchKitRoot
  $stateDir = Join-Path $patchKitRoot ".state"
  New-Item -ItemType Directory -Force -Path $stateDir | Out-Null
  return $stateDir
}

function Get-StampDbPath {
  return (Join-Path (Get-StateDir) "build-stamps.json")
}

function Get-UpstreamDbPath {
  return (Join-Path (Get-StateDir) "upstreams.json")
}

function Read-UpstreamDb {
  $path = Get-UpstreamDbPath
  if (!(Test-Path -LiteralPath $path)) { return @{} }
  try {
    $obj = Get-Content -Raw -LiteralPath $path | ConvertFrom-Json
    $ht = @{}
    foreach ($p in $obj.PSObject.Properties) { $ht[$p.Name] = $p.Value }
    return $ht
  } catch {
    return @{}
  }
}

function Write-UpstreamDb([hashtable]$Db) {
  $path = Get-UpstreamDbPath
  $Db | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath $path -Encoding UTF8
}

function Get-GitOriginUrl([string]$RepoRoot) {
  try {
    if (!(Test-Path -LiteralPath (Join-Path $RepoRoot '.git'))) { return $null }
    $url = (& git -C $RepoRoot remote get-url origin 2>$null)
    if ($LASTEXITCODE -ne 0) { return $null }
    $u = ($url | Select-Object -First 1).Trim()
    if ($u) { return $u }
  } catch {}
  return $null
}

function Remember-Upstream([string]$RepoRoot) {
  $origin = Get-GitOriginUrl $RepoRoot
  if (!$origin) { return $null }
  $db = Read-UpstreamDb
  $db[$RepoRoot] = $origin
  Write-UpstreamDb $db
  return $origin
}

function Read-StampDb {
  $path = Get-StampDbPath
  if (!(Test-Path -LiteralPath $path)) { return @{} }
  try {
    $obj = Get-Content -Raw -LiteralPath $path | ConvertFrom-Json
    if ($obj -is [hashtable]) { return $obj }
    # Convert PSCustomObject -> hashtable
    $ht = @{}
    foreach ($p in $obj.PSObject.Properties) { $ht[$p.Name] = $p.Value }
    return $ht
  } catch {
    return @{}
  }
}

function Write-StampDb([hashtable]$Db) {
  $path = Get-StampDbPath
  $Db | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath $path -Encoding UTF8
}

function Get-FileHashHex([string]$Path) {
  return (Get-FileHash -Algorithm SHA256 -LiteralPath $Path).Hash.ToLowerInvariant()
}

function Get-StringSha256Hex([string]$Text) {
  $bytes = [System.Text.Encoding]::UTF8.GetBytes($Text)
  $sha = [System.Security.Cryptography.SHA256]::Create()
  try {
    $hash = $sha.ComputeHash($bytes)
    return ([BitConverter]::ToString($hash).Replace("-", "").ToLowerInvariant())
  } finally {
    $sha.Dispose()
  }
}

function Get-OpencodeBuildStamp {
  param(
    [Parameter(Mandatory = $true)]
    [string]$RepoRoot
  )

  $files = @(
    "bun.lock",
    "packages/opencode/package.json",
    "packages/opencode/src/cli/cmd/tui/app.tsx",
    "packages/opencode/src/cli/cmd/tui/util/exit-confirm.ts",
    "packages/opencode/src/cli/cmd/tui/component/prompt/index.tsx",
    "packages/opencode/src/cli/cmd/tui/routes/session/index.tsx",
    "packages/opencode/src/cli/cmd/tui/routes/session/permission.tsx",
    "packages/opencode/src/cli/cmd/tui/routes/session/question.tsx",
    "packages/opencode/src/cli/cmd/tui/ui/dialog-confirm.tsx",
    "packages/opencode/src/project/project.ts",
    "packages/script/src/index.ts"
  )

  $rows = @()
  foreach ($rel in $files) {
    $abs = Join-Path $RepoRoot $rel
    if (Test-Path -LiteralPath $abs) {
      $rows += ($rel + ":" + (Get-FileHashHex $abs))
    } else {
      $rows += ($rel + ":missing")
    }
  }

  # Include PatchKit rules template so changing it triggers rebuild when it affects behavior.
  $patchKitRoot = Resolve-PatchKitRoot
  $agents = Join-Path $patchKitRoot "templates/AGENTS.md"
  if (Test-Path -LiteralPath $agents) {
    $rows += ("tools/opencode-patchkit/templates/AGENTS.md:" + (Get-FileHashHex $agents))
  }

  $text = ($rows | Sort-Object) -join "`n"
  return (Get-StringSha256Hex $text)
}
