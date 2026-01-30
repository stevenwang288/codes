@echo off
setlocal EnableExtensions

REM Thin wrapper that delegates to PowerShell for robust quoting and logic.
REM Usage: codes [args...]

set "SCRIPT_DIR=%~dp0"
REM `%~dp0` always ends with a trailing backslash; trim it unconditionally.
set "SCRIPT_DIR=%SCRIPT_DIR:~0,-1%"
set "PS1=%SCRIPT_DIR%\scripts\codes.ps1"

if exist "%PS1%" (
  REM Prefer pwsh when available; fall back to Windows PowerShell.
  where pwsh >nul 2>nul
  if "%ERRORLEVEL%"=="0" (
    pwsh -NoProfile -ExecutionPolicy Bypass -File "%PS1%" -- %*
    exit /b %ERRORLEVEL%
  )
  powershell -NoProfile -ExecutionPolicy Bypass -File "%PS1%" %*
  exit /b %ERRORLEVEL%
)

echo [codes] ERROR: missing "%PS1%" 1>&2
exit /b 1
