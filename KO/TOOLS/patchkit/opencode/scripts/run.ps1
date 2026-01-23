# Purpose:
# - Apply PatchKit customizations to an *external* opencode checkout.
# - Designed to live in code-main under tools/.
#
# Usage:
#   pwsh -ExecutionPolicy Bypass -File "./tools/opencode-patchkit/scripts/run.ps1" -RepoRoot "D:/path/to/opencode"
#
# Notes:
# - If -RepoRoot is omitted, PatchKit will try OPENCODE_REPO_ROOT, patchkit.json, or auto-discovery.

$ErrorActionPreference = "Stop"

param(
  [string]$RepoRoot
)

$ScriptRoot = Split-Path -Parent $PSCommandPath
. (Join-Path $ScriptRoot "_lib.ps1")

$RepoRoot = Resolve-OpencodeRepoRoot -RepoRoot $RepoRoot
$ApplyScript = Join-Path $ScriptRoot "apply-patches.ps1"

& $ApplyScript -RepoRoot $RepoRoot
