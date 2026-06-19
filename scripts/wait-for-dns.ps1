# Poll until .us points to Render (run anytime after Namecheap DNS save)
$ErrorActionPreference = "Continue"
$domain = "https://palworld-breeding-calculator.us"
$max = if ($env:DNS_WAIT_TRIES) { [int]$env:DNS_WAIT_TRIES } else { 120 }

Write-Host "Waiting for $domain -> Render (every 30s, max $max tries)..." -ForegroundColor Cyan
for ($i = 1; $i -le $max; $i++) {
    try {
        $r = Invoke-WebRequest "$domain/palworld-breeding-calculator" -UseBasicParsing -TimeoutSec 25
        if ($r.StatusCode -eq 200 -and -not $r.Headers["X-Nf-Request-Id"]) {
            Write-Host "[OK] Domain live on Render (try $i)" -ForegroundColor Green
            $env:LIVE_SITE_URL = $domain
            & (Join-Path $PSScriptRoot "verify-live-seo.ps1")
            exit 0
        }
        Write-Host "[--] try $i - still Netlify or wrong host" -ForegroundColor Yellow
    } catch {
        Write-Host "[--] try $i - $($_.Exception.Message)" -ForegroundColor Yellow
    }
    Start-Sleep -Seconds 30
}
Write-Host "Not ready yet. DNS can take up to 48 hours. Re-run this script later." -ForegroundColor Yellow
exit 1
