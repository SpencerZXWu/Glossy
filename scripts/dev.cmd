@echo off
rem Runs Glossy from this source tree: a debug binary and the files in src\ as
rem they are, so a change can be tried without building an installer. Double
rem clickable - scripts\make-shortcut.ps1 puts a shortcut on the desktop.
setlocal
title Glossy dev
cd /d "%~dp0.."

where cargo >nul 2>nul
if errorlevel 1 (
  echo cargo is not on PATH. Install the Rust toolchain first - see README.md.
  pause
  exit /b 1
)

if not exist node_modules (
  echo Installing the Tauri CLI...
  call npm.cmd install
  if errorlevel 1 (
    echo npm install failed.
    pause
    exit /b 1
  )
)

rem A second Glossy would install a second mouse hook, so the running one has to
rem go first; otherwise this one shows "already running" and exits.
tasklist /nh /fi "imagename eq glossy.exe" 2>nul | find /i "glossy.exe" >nul
if not errorlevel 1 (
  echo Glossy is already running. Quit it from the notification area first,
  echo then run this again.
  pause
  exit /b 1
)

call npm.cmd run dev

echo.
echo Glossy dev has stopped.
pause
