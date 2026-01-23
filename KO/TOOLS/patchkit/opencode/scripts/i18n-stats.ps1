# Show i18n collection stats for a target directory.

param(
  # Directory that contains .opencode/i18n-missing.json
  [string]$TargetDir = (Get-Location).Path
)

$ErrorActionPreference = "Stop"

$missingPath = Join-Path $TargetDir ".opencode/i18n-missing.json"
if (!(Test-Path -LiteralPath $missingPath)) {
  Write-Host "[i18n] missing file not found: $missingPath" -ForegroundColor Yellow
  Write-Host "[i18n] 0 pending"
  exit 0
}

try {
  $json = Get-Content -Raw -LiteralPath $missingPath | ConvertFrom-Json
} catch {
  Write-Host "[i18n] invalid json: $missingPath" -ForegroundColor Red
  exit 1
}

$total = 0
foreach ($p in $json.PSObject.Properties) {
  $locale = $p.Name
  $entries = $p.Value
  $count = 0
  if ($entries -and $entries.PSObject) {
    $count = ($entries.PSObject.Properties | Measure-Object).Count
  }
  $total += $count
  Write-Host ("[i18n] " + $locale + ": " + $count)
}

Write-Host ("[i18n] total: " + $total)

