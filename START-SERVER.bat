@echo off
cd /d "%~dp0"
echo.
powershell -ExecutionPolicy Bypass -File "%~dp0scripts\start-server.ps1"
if errorlevel 1 (
  echo.
  echo FAILED - see FIX-ERRORS.txt
  pause
  exit /b 1
)
echo.
echo Server window must stay OPEN while you use the site.
pause
