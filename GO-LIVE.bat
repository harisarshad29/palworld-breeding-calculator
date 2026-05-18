@echo off
cd /d "%~dp0"
echo.
echo === Go Live Checklist ===
echo.
powershell -ExecutionPolicy Bypass -File ".\scripts\prepare-rust.ps1"
if errorlevel 1 pause & exit /b 1
echo.
echo [1/3] Building Rust release...
cargo build --release
if errorlevel 1 (
  echo BUILD FAILED
  pause
  exit /b 1
)
echo.
echo [2/3] Local SEO QA (server must be running in another window for full QA)
echo       Or run START-SERVER.bat first, then run this again.
echo.
echo [3/3] Open DEPLOY-ALL.txt for Render + Namecheap steps
start notepad "%~dp0DEPLOY-ALL.txt"
echo.
echo Done. Follow DEPLOY-ALL.txt to point domain to Render.
pause
