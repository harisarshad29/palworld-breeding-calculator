@echo off
cd /d "%~dp0"
echo.
echo Opening LIVE site + deploy help pages...
echo.
start "" "https://palworld-breeding-calculator.onrender.com/palworld-breeding-calculator"
start "" "https://dashboard.render.com"
start "" "https://ap.www.namecheap.com/domains/domaincontrolpanel/palworld-breeding-calculator.us/advancedns"
echo.
powershell -ExecutionPolicy Bypass -File "%~dp0scripts\go-live.ps1"
echo.
pause
