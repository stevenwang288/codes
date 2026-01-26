@echo off
setlocal EnableExtensions EnableDelayedExpansion

REM codes.cmd - Windows launcher for this repo (Git Bash required).
REM Goal: `codes` should work from anywhere.

if "%CODES_TRACE%"=="1" (
  echo [codes] CMDLINE: %cmdcmdline%
  echo [codes] ARGS  : %*
  echo [codes] ARG1  : %1
)

set "BASH_EXE=C:\Program Files\Git\usr\bin\bash.exe"
if not exist "%BASH_EXE%" (
  echo [codes] ERROR: Git Bash not found at "%BASH_EXE%" 1>&2
  endlocal
  exit /b 1
)

set "CODES_STATE_DIR=%LOCALAPPDATA%\codes"
if "%LOCALAPPDATA%"=="" set "CODES_STATE_DIR=%USERPROFILE%\AppData\Local\codes"
set "CODES_LAST_REPO_FILE=%CODES_STATE_DIR%\repo-root.txt"

call :resolve_repo_root
if errorlevel 1 (
  endlocal
  exit /b 1
)

pushd "%REPO_WIN%" >nul
if errorlevel 1 (
  echo [codes] ERROR: Unable to enter repo root: "%REPO_WIN%" 1>&2
  endlocal
  exit /b 1
)

set "CODE_HOME=%CD%\.codes-home"
set "CODEX_HOME=%CD%\.codes-home"
if "%CODEX_LANG%"=="" (
  if exist "%CODE_HOME%\ui-language.txt" (
    for /f "usebackq delims=" %%I in ("%CODE_HOME%\ui-language.txt") do (
      set "CODEX_LANG=%%I"
      goto :codes_lang_loaded
    )
  )
)
:codes_lang_loaded
if "%CODEX_LANG%"=="" set "CODEX_LANG=zh-CN"
set "CODE_AUTO_TRUST=1"

REM Lightweight diagnostics that don't enter the TUI.
REM Usage: codes which
if /I "%~1"=="which" (
  shift
  echo [codes] repo-root: "%CD%"
  echo [codes] CODE_HOME: "%CODE_HOME%"
  echo [codes] CODEX_HOME: "%CODEX_HOME%"
  echo [codes] CODEX_LANG: "%CODEX_LANG%"
  if exist "%CODE_HOME%\ui-language.txt" (
    for /f "usebackq delims=" %%I in ("%CODE_HOME%\ui-language.txt") do echo [codes] ui-language.txt: %%I
  ) else (
    echo [codes] ui-language.txt: (missing)
  )
  if exist "%CODE_HOME%\last-built-bin.txt" (
    for /f "usebackq delims=" %%I in ("%CODE_HOME%\last-built-bin.txt") do echo [codes] last-built-bin.txt: %%I
  ) else (
    echo [codes] last-built-bin.txt: (missing)
  )
  popd >nul
  endlocal
  exit /b 0
)

REM -z means: enable i18n missing-key collection for this run.
set "CODE_I18N_COLLECT_MISSING="
set "FORCE_REBUILD="
set "CODE_ARGS="

REM Arg parsing via SHIFT is more robust than FOR %%A in (%*),
REM which can break on certain characters (notably parentheses).
:parse_args
if "%~1"=="" goto :parse_args_done
if /I "%~1"=="-z" set "CODE_I18N_COLLECT_MISSING=1" & shift & goto :parse_args
if /I "%~1"=="--rebuild" set "FORCE_REBUILD=1" & shift & goto :parse_args
if /I "%~1"=="build" set "FORCE_REBUILD=1" & shift & goto :parse_args
set "CODE_ARGS=%CODE_ARGS% %~1"
shift
goto :parse_args
:parse_args_done

if not exist "%CODE_HOME%" mkdir "%CODE_HOME%" >nul 2>nul

if not exist "%CODES_STATE_DIR%" mkdir "%CODES_STATE_DIR%" >nul 2>nul
(
  echo %CD%
) > "%CODES_LAST_REPO_FILE%" 2>nul

REM Fast path: if a built binary exists, run it directly to avoid recompiling.
REM Use build-fast.sh only when missing or when forced via `codes --rebuild` / `codes build`.
set "FAST_BIN=%CD%\code-rs\target\dev-fast\code"
set "FAST_BIN_EXE=%CD%\code-rs\target\dev-fast\code.exe"
if not "%FORCE_REBUILD%"=="1" if exist "%FAST_BIN_EXE%" (
  "%FAST_BIN_EXE%"%CODE_ARGS%
  set "EXIT_CODE=%ERRORLEVEL%"
  popd >nul
  endlocal & exit /b %EXIT_CODE%
)

REM Preferred fast path: use the last-built bin recorded by build-fast.sh.
set "LAST_BUILT_BIN="
if not "%FORCE_REBUILD%"=="1" if exist "%CODE_HOME%\last-built-bin.txt" (
  for /f "usebackq delims=" %%I in ("%CODE_HOME%\last-built-bin.txt") do set "LAST_BUILT_BIN=%%I"
  if not "%LAST_BUILT_BIN%"=="" (
    REM Prefer running a Windows path directly to avoid cmd/PowerShell quoting edge cases.
    set "LAST_BUILT_WIN="
    if "%LAST_BUILT_BIN:~0,3%"=="/c/" set "LAST_BUILT_WIN=C:\%LAST_BUILT_BIN:~3%"
    if "%LAST_BUILT_BIN:~0,3%"=="/d/" set "LAST_BUILT_WIN=D:\%LAST_BUILT_BIN:~3%"
    if "%LAST_BUILT_BIN:~0,3%"=="/e/" set "LAST_BUILT_WIN=E:\%LAST_BUILT_BIN:~3%"
    if "%LAST_BUILT_BIN:~0,3%"=="/f/" set "LAST_BUILT_WIN=F:\%LAST_BUILT_BIN:~3%"
    if not "%LAST_BUILT_WIN%"=="" set "LAST_BUILT_WIN=%LAST_BUILT_WIN:/=\%"

    set "LAST_BUILT_WIN_EXE=%LAST_BUILT_WIN%"
    if not "%LAST_BUILT_WIN_EXE%"=="" (
      if not exist "%LAST_BUILT_WIN_EXE%" if exist "%LAST_BUILT_WIN_EXE%.exe" set "LAST_BUILT_WIN_EXE=%LAST_BUILT_WIN_EXE%.exe"
    )

    if not "%LAST_BUILT_WIN_EXE%"=="" if exist "%LAST_BUILT_WIN_EXE%" (
      set "CACHED_EXE=%CODE_HOME%\code.exe"
      copy /Y "%LAST_BUILT_WIN_EXE%" "%CACHED_EXE%" >nul 2>nul
      if "%ERRORLEVEL%"=="0" (
        "%CACHED_EXE%"%CODE_ARGS%
      ) else (
        REM If the cached exe is locked by another running instance, run the fresh binary directly.
        "%LAST_BUILT_WIN_EXE%"%CODE_ARGS%
      )
    ) else (
      REM Fallback: run via Git Bash if the path couldn't be converted.
      "%BASH_EXE%" -lc "cd \"`cygpath -u '%CD%'`\"; %LAST_BUILT_BIN%%CODE_ARGS%"
    )
    set "EXIT_CODE=%ERRORLEVEL%"
    popd >nul
    endlocal & exit /b %EXIT_CODE%
  )
)
if not "%FORCE_REBUILD%"=="1" if exist "%FAST_BIN%" (
  REM Run via Git Bash so the non-.exe binary works reliably.
  "%BASH_EXE%" -lc "cd \"`cygpath -u '%CD%'`\"; ./code-rs/target/dev-fast/code%CODE_ARGS%"
  set "EXIT_CODE=%ERRORLEVEL%"
  popd >nul
  endlocal & exit /b %EXIT_CODE%
)

REM No cached binary: build (and optionally run) via build-fast.sh.
if "%CODE_ARGS%"=="" (
  "%BASH_EXE%" -lc "cd \"`cygpath -u '%CD%'`\"; ./build-fast.sh run"
) else (
  "%BASH_EXE%" -lc "cd \"`cygpath -u '%CD%'`\"; ./build-fast.sh"
  if errorlevel 1 (
    set "EXIT_CODE=%ERRORLEVEL%"
    popd >nul
    endlocal & exit /b %EXIT_CODE%
  )
  if exist "%FAST_BIN_EXE%" (
    "%FAST_BIN_EXE%"%CODE_ARGS%
  ) else (
    "%BASH_EXE%" -lc "cd \"`cygpath -u '%CD%'`\"; ./code-rs/target/dev-fast/code%CODE_ARGS%"
  )
)

set "EXIT_CODE=%ERRORLEVEL%"
popd >nul
endlocal & exit /b %EXIT_CODE%

:resolve_repo_root
set "REPO_WIN="

if not "%CODES_REPO_ROOT%"=="" if exist "%CODES_REPO_ROOT%\build-fast.sh" set "REPO_WIN=%CODES_REPO_ROOT%"
if "%REPO_WIN%"=="" if not "%CODE_REPO_ROOT%"=="" if exist "%CODE_REPO_ROOT%\build-fast.sh" set "REPO_WIN=%CODE_REPO_ROOT%"

if "%REPO_WIN%"=="" (
  for /f "usebackq delims=" %%I in (`git -C "%CD%" rev-parse --show-toplevel 2^>nul`) do (
    if exist "%%I\build-fast.sh" set "REPO_WIN=%%I"
  )
)

if not "%REPO_WIN%"=="" exit /b 0

set "SEARCH_WIN=%CD%"
:search_up
if exist "%SEARCH_WIN%\build-fast.sh" (
  set "REPO_WIN=%SEARCH_WIN%"
  exit /b 0
)

for %%I in ("%SEARCH_WIN%\..") do set "PARENT_WIN=%%~fI"
if /I "%PARENT_WIN%"=="%SEARCH_WIN%" goto :search_done
set "SEARCH_WIN=%PARENT_WIN%"
goto :search_up
:search_done

if "%REPO_WIN%"=="" (
  set "SCRIPT_DIR=%~dp0"
  if "%SCRIPT_DIR:~-1%"=="\" set "SCRIPT_DIR=%SCRIPT_DIR:~0,-1%"
  if exist "%SCRIPT_DIR%\build-fast.sh" set "REPO_WIN=%SCRIPT_DIR%"
)

if "%REPO_WIN%"=="" if exist "%CODES_LAST_REPO_FILE%" (
  for /f "usebackq delims=" %%I in ("%CODES_LAST_REPO_FILE%") do (
    if exist "%%I\build-fast.sh" set "REPO_WIN=%%I"
    goto :last_repo_done
  )
  :last_repo_done
)

if "%REPO_WIN%"=="" (
  echo [codes] ERROR: Repo root not found from "%CD%". 1>&2
  echo [codes] Hint: run from inside the repo (contains build-fast.sh). 1>&2
  echo [codes]       Or set CODES_REPO_ROOT to the repo root path. 1>&2
  exit /b 1
)

exit /b 0
