@echo off
cd /d "%~dp0"
echo.
echo HTML TAG FIX (if you prefer meta tag over DNS)
echo ===============================================
echo.
echo Search Console HTML tag section se EXACT content copy karo.
echo Example: content="abc123xyz"  - sirf abc123xyz paste karo
echo.
set /p TOKEN="Google HTML tag content paste karo: "
if "%TOKEN%"=="" (
  echo Khali - cancel
  pause
  exit /b 1
)
echo.
echo Render dashboard khul raha hai - Environment mein add karo:
echo   Key:   GOOGLE_SITE_VERIFICATION
echo   Value: %TOKEN%
echo.
start "" "https://dashboard.render.com"
echo Token clipboard mein copy ho gaya:
echo %TOKEN% | clip
echo.
echo Render - Environment - Add GOOGLE_SITE_VERIFICATION = pasted value
echo Save - Manual Deploy - phir Search Console VERIFY dabao
pause
