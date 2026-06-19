# Audit LIVE site on-page SEO (after Render + domain)
$ErrorActionPreference = "Continue"
$RenderUrl = "https://palworld-breeding-calculator.onrender.com"
$Base = if ($env:LIVE_SITE_URL) { $env:LIVE_SITE_URL.TrimEnd("/") } else { "https://palworld-breeding-calculator.us" }
$Failed = 0

function Fail($msg) { Write-Host "[FAIL] $msg" -ForegroundColor Red; $script:Failed++ }
function Pass($msg) { Write-Host "[PASS] $msg" -ForegroundColor Green }
function Get-Page($path, [int]$TimeoutSec = 20) {
    try { return Invoke-WebRequest -Uri "$Base$path" -UseBasicParsing -TimeoutSec $TimeoutSec }
    catch { Fail "GET $path - $($_.Exception.Message)"; return $null }
}

function Test-NetlifyOldSite {
    try {
        $r = Invoke-WebRequest -Uri "$Base/palworld-breeding-calculator" -UseBasicParsing -TimeoutSec 25
        if ($r.Headers["X-Nf-Request-Id"] -or ($r.Headers["Server"] -match "Netlify")) {
            Fail "Domain still on NETLIFY old static site. Fix DNS or redeploy Netlify with netlify.toml redirect."
            Write-Host "      Working URL: $RenderUrl/palworld-breeding-calculator" -ForegroundColor Yellow
            return $true
        }
    } catch {
        if ($_.Exception.Message -match "404") {
            try {
                $rootPage = Invoke-WebRequest -Uri "$Base/" -UseBasicParsing -TimeoutSec 15
                if ($rootPage.Headers["X-Nf-Request-Id"]) {
                    Fail ".us returns 404 for calculator - Netlify old site. Use Render URL or fix DNS."
                    Write-Host "      Working URL: $RenderUrl/palworld-breeding-calculator" -ForegroundColor Yellow
                    return $true
                }
            } catch { }
        }
    }
    return $false
}

Write-Host ""
Write-Host "Live SEO audit: $Base" -ForegroundColor Cyan
Write-Host ""

if (Test-NetlifyOldSite) {
    Write-Host ""
    Write-Host "Stopped: fix domain first, then re-run." -ForegroundColor Yellow
    exit 1
}

$r = Get-Page "/" -TimeoutSec 90
if ($r) {
    $h = $r.Content
    if ($h -match "<title>") { Pass "title tag present" } else { Fail "missing title" }
    $meta = [regex]::Match($h, 'name="description"\s+content="([^"]+)"').Groups[1].Value
    if ($meta) {
        $w = ($meta -split "\s+").Count
        if ($w -ge 120) { Pass "meta description ~$w words" } else { Fail "meta description only $w words (want ~150)" }
    } else { Fail "missing meta description" }
    if ($h -match "application/ld\+json") { Pass "JSON-LD present" } else { Fail "no JSON-LD" }
    if ($h -match "127\.0\.0\.1|localhost") { Fail "page has localhost URLs" } else { Pass "no localhost in HTML" }
    if ($h -match 'rel="canonical"') { Pass "canonical present" } else { Fail "missing canonical" }
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
