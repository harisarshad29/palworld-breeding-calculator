@echo off
cd /d "%~dp0"
echo.
echo ========================================
echo   PALWORLD - TUM LOGIN, BAQI AUTO GUIDE
echo ========================================
echo.
echo 1. Login tabs khulengi (Namecheap, Render, Google)
echo 2. COPY-PASTE-VALUES.txt mein sab values hain
echo 3. Script DNS check karegi jab tum save kar lo
echo.
pause
powershell -ExecutionPolicy Bypass -File "%~dp0scripts\complete-setup.ps1"
pause
