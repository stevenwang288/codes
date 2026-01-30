param(
  [Parameter(Mandatory = $true)]
  [string]$RepoRoot
)

Set-StrictMode -Version Latest

. "${PSScriptRoot}/_lib.ps1"

$RepoRoot = Resolve-RepoRoot -RepoRoot $RepoRoot
Assert-GitRepo -RepoRoot $RepoRoot

$patchFiles = @(Get-PatchFiles -RepoRoot $RepoRoot)
if ($patchFiles.Length -eq 0) {
  Write-Host "[codes-patchkit] No patch files found." -ForegroundColor Yellow
  exit 0
}

Push-Location $RepoRoot
try {
  foreach ($patch in $patchFiles) {
    Write-Section "Applying patch: $($patch.Name)"

    git apply --check --ignore-space-change --ignore-whitespace --whitespace=nowarn "$($patch.FullName)" 2>$null
    $canApply = ($LASTEXITCODE -eq 0)

    if (-not $canApply) {
      git apply --reverse --check --ignore-space-change --ignore-whitespace --whitespace=nowarn "$($patch.FullName)" 2>$null
      $alreadyApplied = ($LASTEXITCODE -eq 0)
      if ($alreadyApplied) {
        Write-Host "[codes-patchkit] Patch already applied, skip." -ForegroundColor Yellow
        continue
      }
      throw "Patch does not apply cleanly and does not look already applied: $($patch.FullName)"
    }

    git apply --whitespace=nowarn "$($patch.FullName)"
    if ($LASTEXITCODE -ne 0) {
      throw "git apply failed (exit $LASTEXITCODE): $($patch.FullName)"
    }
  }
} finally {
  Pop-Location
}

Write-Host "[codes-patchkit] Done." -ForegroundColor Green
