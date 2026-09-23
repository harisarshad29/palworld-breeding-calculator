@echo off
cd /d "%~dp0"
echo.
echo GOOGLE SEARCH CONSOLE - verify + sitemap
echo ========================================
echo.
start "" "https://search.google.com/search-console?resource_id=sc-domain:palworld-breeding-calculator.us"
timeout /t 2 >nul
echo.
echo Search Console mein:
echo   1. Property = Domain palworld-breeding-calculator.us
echo   2. Indexing - Sitemaps
echo   3. Remove galat entries (/, /pal/anubis, /pals) agar hon
echo   4. Add sitemap: sitemap.xml
echo      Full URL: https://www.palworld-breeding-calculator.us/sitemap.xml
echo   5. Page URLs Request indexing se INDEX-URLS.txt (GSC-INDEX-NOW.bat)
echo.
pause
start "" "https://www.palworld-breeding-calculator.us/sitemap.xml"
pause
