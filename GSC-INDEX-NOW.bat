@echo off
cd /d "%~dp0"
echo.
echo GOOGLE INDEXING - priority URLs (sitemap already submitted)
echo ============================================================
echo.
start "" "https://search.google.com/search-console?resource_id=sc-domain:palworld-breeding-calculator.us"
timeout /t 2 >nul
start notepad "%~dp0INDEX-URLS.txt"
echo.
echo Search Console mein:
echo   1. Property = Domain palworld-breeding-calculator.us (www+apex)
echo   2. URL inspection (top search bar)
echo   3. INDEX-URLS.txt se ek FULL https://www.... URL copy karo
echo   4. Paste - Enter - "Request indexing"
echo   5. Roz 5-10 se zyada mat karo
echo.
echo Note: Apex URLs 301 to www - list uses www (200). Comments mat paste karo.
echo Sitemap: Indexing - Sitemaps - sitemap.xml
echo.
echo Har hafte: Indexing - Pages - indexed count dekho
echo.
pause
