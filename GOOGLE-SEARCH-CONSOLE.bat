@echo off
cd /d "%~dp0"
echo.
echo GOOGLE SEARCH CONSOLE - sirf 1 click VERIFY (baqi main ne code mein kar diya)
echo ==============================================================================
echo.
echo Render par Google verification tag add ho chuka hai.
echo Tumhein sirf Search Console mein VERIFY dabana hai.
echo.
start "" "https://search.google.com/search-console"
timeout /t 2 >nul
echo.
echo Search Console mein:
echo   1. Add property - URL prefix (RIGHT side)
echo   2. URL: https://palworld-breeding-calculator.us
echo   3. Verification: HTML tag (recommended)
echo   4. VERIFY dabao - file upload ki zaroorat NAHI
echo.
echo Deploy wait: Render 2-3 min redeploy ho raha ho to thori der baad VERIFY karo.
echo.
pause
start "" "https://palworld-breeding-calculator.us/"
pause
