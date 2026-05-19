# Daily SEO routine reminder — run each morning
$Base = if ($env:LIVE_SITE_URL) { $env:LIVE_SITE_URL.TrimEnd("/") } else { "https://palworld-breeding-calculator.us" }

Write-Host ""
Write-Host "=== Daily SEO (15-30 min) ===" -ForegroundColor Cyan
Write-Host ""
Write-Host "[ ] 1. Open seo/backlink-tracker.csv — log 5 outreach (sent/replied/live)" -ForegroundColor Yellow
Write-Host "[ ] 2. Send messages from seo/outreach-templates.md" -ForegroundColor Yellow
Write-Host "[ ] 3. Search Console — Request indexing for 1 new/updated URL" -ForegroundColor Yellow
Write-Host "[ ] 4. Share 1 helpful link (Discord/Reddit) — no spam" -ForegroundColor Yellow
Write-Host "[ ] 5. Check live site: $Base/palworld-breeding-calculator" -ForegroundColor Yellow
Write-Host ""
Write-Host "Weekly: run scripts\verify-live-seo.ps1 + review GSC Performance" -ForegroundColor Green
Write-Host "Full plan: seo\KAM-KARO-ON-OFF-PAGE.md" -ForegroundColor Green
Write-Host ""
