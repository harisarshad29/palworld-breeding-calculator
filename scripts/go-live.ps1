$ErrorActionPreference = "Continue"
$live = "https://palworld-breeding-calculator.onrender.com"
$domain = "https://palworld-breeding-calculator.us"

Write-Host "`n=== Palworld Breeding Calculator - GO LIVE ===" -ForegroundColor Cyan
Write-Host ""

# 1) Render (already deployed if repo connected)
Write-Host "[1] Render live URL (works now):" -ForegroundColor Yellow
Write-Host "    $live/palworld-breeding-calculator" -ForegroundColor Green

$checks = @(
    "/palworld-breeding-calculator",
    "/pal/anubis",
    "/combo/bellanoir/cryolinx-terra",
    "/robots.txt",
    "/sitemap.xml"
)
$ok = 0
foreach ($path in $checks) {
    try {
        $r = Invoke-WebRequest "$live$path" -UseBasicParsing -TimeoutSec 90
        if ($r.StatusCode -eq 200) {
            Write-Host "    [OK] $path" -ForegroundColor Green
            $ok++
        } else {
            Write-Host "    [??] $path -> $($r.StatusCode)" -ForegroundColor Yellow
        }
    } catch {
        Write-Host "    [FAIL] $path - $($_.Exception.Message)" -ForegroundColor Red
    }
}

Write-Host ""
Write-Host "[2] Custom domain (.us) - YOU must set Namecheap DNS once:" -ForegroundColor Yellow
Write-Host "    DELETE Netlify: A @ 75.2.60.5 and CNAME www -> netlify" -ForegroundColor Gray
Write-Host "    ADD Render:   A @ 216.24.57.1" -ForegroundColor Gray
Write-Host "    ADD Render:   CNAME www -> palworld-breeding-calculator.onrender.com" -ForegroundColor Gray
Write-Host "    Then Render -> Settings -> Custom Domains -> add .us + www" -ForegroundColor Gray

try {
    Invoke-WebRequest "$domain/palworld-breeding-calculator" -UseBasicParsing -TimeoutSec 15 | Out-Null
    Write-Host ""
    Write-Host "    [OK] $domain already works!" -ForegroundColor Green
} catch {
    Write-Host ""
    Write-Host "    [--] $domain not on Render yet (expected until DNS)" -ForegroundColor DarkYellow
}

Write-Host ""
Write-Host "Passed $ok / $($checks.Count) Render checks." -ForegroundColor Cyan
Write-Host "Full guide: DEPLOY-ALL.txt`n" -ForegroundColor Gray

if ($env:LIVE_SITE_URL) {
    & (Join-Path $PSScriptRoot "verify-live-seo.ps1")
} else {
    $env:LIVE_SITE_URL = $live
    & (Join-Path $PSScriptRoot "verify-live-seo.ps1")
}
