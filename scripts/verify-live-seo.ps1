# Audit LIVE site on-page SEO (after Render + domain)
$ErrorActionPreference = "Continue"
$Base = if ($env:LIVE_SITE_URL) { $env:LIVE_SITE_URL.TrimEnd("/") } else { "https://palworld-breeding-calculator.us" }
$Failed = 0

function Fail($msg) { Write-Host "[FAIL] $msg" -ForegroundColor Red; $script:Failed++ }
function Pass($msg) { Write-Host "[PASS] $msg" -ForegroundColor Green }
function Get-Page($path) {
    try { return Invoke-WebRequest -Uri "$Base$path" -UseBasicParsing -TimeoutSec 20 }
    catch { Fail "GET $path - $($_.Exception.Message)"; return $null }
}

Write-Host ""
Write-Host "Live SEO audit: $Base" -ForegroundColor Cyan
Write-Host ""

$r = Get-Page "/"
if ($r) {
    $h = $r.Content
    if ($h -match "<title>([^<]+)</title>") { Pass "title: $($matches[1])" } else { Fail "missing title" }
    if ($h -match 'name="description"\s+content="([^"]+)"' -or $h -match 'name="description" content="([^"]+)"') {
        $w = ($matches[1] -split "\s+").Count
        if ($w -ge 120) { Pass "meta description ~$w words" } else { Fail "meta description only $w words (want ~150)" }
    } else { Fail "missing meta description" }
    if ($h -match 'application/ld\+json') { Pass "JSON-LD present" } else { Fail "no JSON-LD (FAQ schema)" }
    if ($h -match '127\.0\.0\.1|localhost') { Fail "page still has localhost URLs in HTML" } else { Pass "no localhost in HTML" }
    if ($h -match 'rel="canonical" href="([^"]+)"') {
        if ($matches[1] -like "$Base*") { Pass "canonical OK" } else { Fail "canonical wrong: $($matches[1])" }
    } else { Fail "missing canonical" }
}

foreach ($path in @("/palworld-breeding-calculator", "/pal/anubis", "/robots.txt", "/sitemap.xml")) {
    $p = Get-Page $path
    if ($p -and $p.StatusCode -eq 200) { Pass "$path -> 200" }
    elseif ($p) { Fail "$path -> $($p.StatusCode)" }
}

if ($Failed -eq 0) {
    Write-Host ""
    Write-Host "Live site looks good for SEO." -ForegroundColor Green
} else {
    Write-Host ""
    Write-Host "$Failed issue(s). See DEPLOY-ALL.txt (Render + BASE_URL + DNS)." -ForegroundColor Yellow
}
Write-Host ""
exit $Failed
