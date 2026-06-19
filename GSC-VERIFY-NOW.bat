@echo off
cd /d "%~dp0"
echo.
echo GOOGLE VERIFY - DNS TXT already live on Namecheap
echo ==================================================
echo.
powershell -NoProfile -Command "Resolve-DnsName palworld-breeding-calculator.us -Type TXT | Where-Object { $_.Strings -match 'google-site-verification' } | ForEach-Object { Write-Host 'DNS TXT OK:' $_.Strings[0] -ForegroundColor Green }"
echo.
echo Search Console khul raha hai - sirf VERIFY dabao:
start "" "https://search.google.com/search-console/welcome"
timeout /t 2 >nul
echo.
echo Steps:
echo   1. Property: palworld-breeding-calculator.us (Domain)
echo   2. VERIFY click
echo   3. Success ke baad Sitemaps - submit: sitemap.xml
echo.
pause
