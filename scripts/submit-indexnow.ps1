# Submit priority URLs to IndexNow (Bing/Yandex/others). Google still needs GSC Request indexing.
$ErrorActionPreference = "Stop"
$Key = "22f4f9e126a34930a0810a0d85f0b755"
$HostName = "www.palworld-breeding-calculator.us"
$KeyLocation = "https://$HostName/$Key.txt"
$Urls = @(
    "https://$HostName/",
    "https://$HostName/palworld-breeding-calculator",
    "https://$HostName/palworld-breeding-combinations",
    "https://$HostName/palworld-chain-breeding",
    "https://$HostName/pals",
    "https://$HostName/pal/anubis",
    "https://$HostName/pal/jetragon",
    "https://$HostName/pal/frostallion",
    "https://$HostName/how-to-breed/anubis",
    "https://$HostName/how-to-breed/jetragon",
    "https://$HostName/combos/anubis",
    "https://$HostName/pal-pages",
    "https://$HostName/combo-pages",
    "https://$HostName/guides/best-breeding-combos",
    "https://$HostName/guides/how-to-breed-anubis",
    "https://$HostName/legendary-breeding",
    "https://$HostName/fastest-anubis-breed",
    "https://$HostName/sitemap.xml"
)

Write-Host "Checking key file..." -ForegroundColor Cyan
$keyCheck = curl.exe -s -o NUL -w "%{http_code}" $KeyLocation
if ($keyCheck -ne "200") {
    Write-Host "Key not live yet ($keyCheck). Deploy first, then re-run." -ForegroundColor Yellow
    exit 1
}

$body = @{
    host        = $HostName
    key         = $Key
    keyLocation = $KeyLocation
    urlList     = $Urls
} | ConvertTo-Json -Depth 4

Write-Host "Submitting $($Urls.Count) URLs to IndexNow..." -ForegroundColor Cyan
$tmp = Join-Path $env:TEMP "indexnow-body.json"
Set-Content -Path $tmp -Value $body -Encoding utf8
curl.exe -s -w "`nHTTP %{http_code}`n" -X POST "https://api.indexnow.org/indexnow" -H "Content-Type: application/json; charset=utf-8" --data-binary "@$tmp"

Write-Host "`nPinging sitemaps..." -ForegroundColor Cyan
$sitemap = [uri]::EscapeDataString("https://$HostName/sitemap.xml")
curl.exe -s -o NUL -w "Google ping: %{http_code}`n" "https://www.google.com/ping?sitemap=$sitemap"
curl.exe -s -o NUL -w "Bing ping: %{http_code}`n" "https://www.bing.com/ping?sitemap=$sitemap"
Write-Host "Done." -ForegroundColor Green
