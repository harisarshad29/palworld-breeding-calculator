@echo off
cd /d "%~dp0"
echo.
echo Palworld Breeding Calculator (Rust)
echo ====================================

taskkill /F /IM palworld-breeding-calculator.exe >nul 2>&1

powershell -ExecutionPolicy Bypass -File ".\scripts\prepare-rust.ps1"
if errorlevel 1 (
  echo.
  echo ERROR: Missing files. Run RESET.bat or check data/assets folders.
  pause
  exit /b 1
)

echo Starting server...
start "Palworld Server" cmd /k "cd /d "%~dp0" && cargo run"
echo Waiting for server...
timeout /t 12 /nobreak >nul

powershell -Command "try { (Invoke-WebRequest 'http://127.0.0.1:3000/api/bootstrap' -UseBasicParsing -TimeoutSec 5).StatusCode } catch { exit 1 }"
if errorlevel 1 (
  echo.
  echo Server not ready yet. Wait a few seconds, then open:
  echo http://127.0.0.1:3000/palworld-breeding-calculator
) else (
  start "" "http://127.0.0.1:3000/palworld-breeding-calculator"
  echo Browser opened.
)

echo.
echo Keep the "Palworld Server" window open while using the site.
pause
