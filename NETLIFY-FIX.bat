@echo off
cd /d "%~dp0"
powershell -ExecutionPolicy Bypass -File "%~dp0scripts\rebuild-netlify-zip.ps1"
echo.
echo NETLIFY FIX v2 - ALL paths redirect (/_redirects added)
echo ========================================================
echo.
echo STEP 1 - Netlify site khulegi (spiffy-khapse = .us domain)
start "" "https://app.netlify.com/projects/spiffy-khapse-94803b/overview"
timeout /t 2 >nul
echo.
echo STEP 2 - Drag this zip onto "Production deploys" area:
echo   %~dp0NETLIFY-REDIRECT-DEPLOY.zip
echo.
echo STEP 3 - Wait Published, then test opens automatically...
timeout /t 3 >nul
start "" "https://palworld-breeding-calculator.us/palworld-breeding-calculator"
explorer /select,"%~dp0NETLIFY-REDIRECT-DEPLOY.zip"
pause
