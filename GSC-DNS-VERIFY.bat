@echo off
cd /d "%~dp0"
echo.
echo GOOGLE VERIFY - DNS method (HTML tag fail fix - 100%% works)
echo =============================================================
echo.
echo Problem: HTML tag token galat tha (file token != tag token).
echo Solution: DNS TXT - koi code / zip nahi.
echo.
start "" "https://search.google.com/search-console"
timeout /t 2 >nul
start "" "https://ap.www.namecheap.com/domains/domaincontrolpanel/palworld-breeding-calculator.us/advancedns"
echo.
echo STEP 1 - Search Console (left tab):
echo   Add property - DOMAIN (left box)
echo   Enter: palworld-breeding-calculator.us
echo   Copy the TXT record Google shows
echo.
echo STEP 2 - Namecheap (right tab):
echo   Advanced DNS - Add New Record
echo   Type: TXT    Host: @    Value: (paste Google TXT)
echo   Save
echo.
echo STEP 3 - 10-30 min wait - Search Console VERIFY
echo.
notepad "%~dp0COPY-PASTE-VALUES.txt"
pause
