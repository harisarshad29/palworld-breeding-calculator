# Tum login karo — yeh script baaki guide + auto-check karegi
$ErrorActionPreference = "Continue"
$root = Resolve-Path (Join-Path $PSScriptRoot "..")
$domain = "https://palworld-breeding-calculator.us"
$render = "https://palworld-breeding-calculator.onrender.com"

function Step($n, $title) {
    Write-Host ""
    Write-Host "========== STEP $n : $title ==========" -ForegroundColor Cyan
}

function Open-Url($url) {
    Start-Process $url
    Start-Sleep -Milliseconds 800
}

Step 0 "Pehle Render check (code side done)"
Write-Host "GitHub push ho chuka. Render auto-deploy check..." -ForegroundColor Gray
$renderOk = $false
foreach ($path in @("/palworld-breeding-calculator", "/robots.txt")) {
    try {
        $r = Invoke-WebRequest "$render$path" -UseBasicParsing -TimeoutSec 90
        if ($r.StatusCode -eq 200) {
            Write-Host "  [OK] Render$path" -ForegroundColor Green
            $renderOk = $true
        }
    } catch {
        Write-Host "  [WAIT] Render$path - $($_.Exception.Message)" -ForegroundColor Yellow
        $renderOk = $false
        break
    }
}
if (-not $renderOk) {
    Write-Host "Render abhi deploy ho raha ho sakta hai. 2 min baad dubara chalao." -ForegroundColor Yellow
}

Step 1 "Namecheap DNS (tum login -> paste values)"
Write-Host "Browser khul raha hai Namecheap DNS..." -ForegroundColor Yellow
Open-Url "https://ap.www.namecheap.com/domains/domaincontrolpanel/palworld-breeding-calculator.us/advancedns"
Write-Host @"

COPY PASTE (COPY-PASTE-VALUES.txt bhi kholo):

  DELETE: A @ 75.2.60.5  (Netlify)
  DELETE: CNAME www -> netlify

  ADD:    A @ -> 216.24.57.1
  ADD:    CNAME www -> palworld-breeding-calculator.onrender.com

"@ -ForegroundColor White
Read-Host "Namecheap par records save kar liye? Enter dabao"

Step 2 "Render custom domain + env"
Write-Host "Browser khul raha hai Render dashboard..." -ForegroundColor Yellow
Open-Url "https://dashboard.render.com"
Write-Host @"

Service -> Environment:
  HOST     = 0.0.0.0
  BASE_URL = https://palworld-breeding-calculator.us

Settings -> Custom Domains -> Add:
  palworld-breeding-calculator.us
  www.palworld-breeding-calculator.us

"@ -ForegroundColor White
Read-Host "Render par domain add kar liya? Enter dabao"

Step 3 "DNS wait (auto check — 30 min max)"
Write-Host "Ab .us domain check hoti rahegi..." -ForegroundColor Yellow
$live = $false
for ($i = 1; $i -le 60; $i++) {
    try {
        $r = Invoke-WebRequest "$domain/palworld-breeding-calculator" -UseBasicParsing -TimeoutSec 20
        if ($r.StatusCode -eq 200 -and -not $r.Headers["X-Nf-Request-Id"]) {
            Write-Host "  [OK] .us domain LIVE on Render! (try $i)" -ForegroundColor Green
            $live = $true
            break
        }
        if ($r.Headers["X-Nf-Request-Id"]) {
            Write-Host "  [--] try $i - abhi bhi Netlify" -ForegroundColor DarkYellow
        }
    } catch {
        $msg = $_.Exception.Message
        if ($msg -match "404") {
            Write-Host "  [--] try $i - 404 (DNS ya Netlify abhi purani)" -ForegroundColor DarkYellow
        } else {
            Write-Host "  [--] try $i - $msg" -ForegroundColor DarkYellow
        }
    }
    Start-Sleep -Seconds 30
}
if (-not $live) {
    Write-Host ""
    Write-Host "DNS abhi propagate nahi hui (normal 1-48 hrs)." -ForegroundColor Yellow
    Write-Host "Baad mein chalao: powershell -File scripts\wait-for-dns.ps1" -ForegroundColor Gray
} else {
    $env:LIVE_SITE_URL = $domain
    & (Join-Path $PSScriptRoot "verify-live-seo.ps1")
}

Step 4 "Google Search Console"
Write-Host "Browser khul raha hai Search Console..." -ForegroundColor Yellow
Open-Url "https://search.google.com/search-console"
Write-Host @"

Property add: https://palworld-breeding-calculator.us

Sitemap submit:
  $domain/sitemap.xml

URL Inspection -> Request indexing:
  $domain/
  $domain/palworld-breeding-calculator
  $domain/pal/anubis

"@ -ForegroundColor White
Read-Host "Search Console setup kar liya? Enter dabao"

Step 5 "Netlify conflict band (optional)"
Open-Url "https://app.netlify.com"
Write-Host "Netlify se custom domain hata do YA is repo ko redeploy (netlify.toml redirect)." -ForegroundColor Gray

Write-Host ""
Write-Host "=== SETUP WIZARD DONE ===" -ForegroundColor Green
Write-Host "Live URL: $domain/palworld-breeding-calculator" -ForegroundColor Green
Write-Host "Backup:   $render/palworld-breeding-calculator" -ForegroundColor Gray
Write-Host ""
Write-Host "Har roz off-page: seo\KAM-KARO-ON-OFF-PAGE.md" -ForegroundColor Gray
notepad (Join-Path $root "COPY-PASTE-VALUES.txt") | Out-Null
