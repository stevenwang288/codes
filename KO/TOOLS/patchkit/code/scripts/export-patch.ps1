param(
  [Parameter(Mandatory = $true)]
  [string]$RepoRoot,

  [string]$Name = "local",

  [switch]$IncludeStaged
)

Set-StrictMode -Version Latest

. "${PSScriptRoot}/_lib.ps1"

$RepoRoot = Resolve-RepoRoot -RepoRoot $RepoRoot
Assert-GitRepo -RepoRoot $RepoRoot

$patchDir = Get-PatchDir -RepoRoot $RepoRoot
New-Item -ItemType Directory -Path $patchDir -Force | Out-Null

$patchPath = Join-Path $patchDir ("{0}.patch" -f $Name)

Write-Section "Export patch -> $patchPath"

Push-Location $RepoRoot
try {
  $pathspec = @(
    "--",
    ":(exclude)KO/TOOLS/patchkit/code/patches",
    ":(exclude)KO/TOOLS/patchkit/code/.state"
  )

  function Write-GitDiffToStream {
    param(
      [Parameter(Mandatory = $true)]
      [string[]]$Args,
      [Parameter(Mandatory = $true)]
      [System.IO.Stream]$OutStream
    )

    $psi = [System.Diagnostics.ProcessStartInfo]::new()
    $psi.FileName = "git"
    foreach ($a in $Args) { [void]$psi.ArgumentList.Add($a) }
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    $psi.UseShellExecute = $false

    $p = [System.Diagnostics.Process]::new()
    $p.StartInfo = $psi
    if (-not $p.Start()) {
      throw "Failed to start git process"
    }

    try {
      $p.StandardOutput.BaseStream.CopyTo($OutStream)
      $p.WaitForExit()
      if ($p.ExitCode -ne 0) {
        $stderr = $p.StandardError.ReadToEnd()
        throw "git diff failed (exit $($p.ExitCode)): $stderr"
      }
    } finally {
      $p.Dispose()
    }
  }

  $patchDir = Split-Path -Parent $patchPath
  New-Item -ItemType Directory -Path $patchDir -Force | Out-Null
  $fs = [System.IO.File]::Open($patchPath, [System.IO.FileMode]::Create, [System.IO.FileAccess]::Write, [System.IO.FileShare]::Read)
  try {
    Write-GitDiffToStream -Args (@("diff", "--binary") + $pathspec) -OutStream $fs
    if ($IncludeStaged) {
      $nl = [System.Text.Encoding]::UTF8.GetBytes("`n")
      $fs.Write($nl, 0, $nl.Length)
      Write-GitDiffToStream -Args (@("diff", "--binary", "--staged") + $pathspec) -OutStream $fs
    }
  } finally {
    $fs.Dispose()
  }

  $len = (Get-Item -LiteralPath $patchPath).Length
  if ($len -eq 0) {
    Write-Host "[codes-patchkit] No changes to export." -ForegroundColor Yellow
    Remove-Item -Force $patchPath -ErrorAction SilentlyContinue
    return
  }

  Write-Host "[codes-patchkit] Wrote patch: $patchPath" -ForegroundColor Green
} finally {
  Pop-Location
}
