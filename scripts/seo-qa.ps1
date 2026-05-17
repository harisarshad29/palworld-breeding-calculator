# SEO / QC checks for Palworld Breeding Calculator (run while server is up: cargo run)
$ErrorActionPreference = "Stop"
$Base = if ($env:SEO_QA_BASE) { $env:SEO_QA_BASE } else { "http://127.0.0.1:3000" }
$Failed = 0

function Fail($msg) {
    Write-Host "[FAIL] $msg" -ForegroundColor Red
    $script:Failed++
}

function Pass($msg) {
    Write-Host "[PASS] $msg" -ForegroundColor Green
}

function Get-Page($path) {
    try {
        return Invoke-WebRequest -Uri "$Base$path" -UseBasicParsing
    } catch {
        Fail "Could not fetch $path - is the server running? (cargo run)"
        return $null
    }
}

function Get-RedirectStatus($path) {
    try {
        $r = Invoke-WebRequest -Uri "$Base$path" -MaximumRedirection 0 -UseBasicParsing
        return [int]$r.StatusCode
    } catch {
        if ($_.Exception.Response) {
            return [int]$_.Exception.Response.StatusCode
        }
        Fail "Could not fetch $path redirect - is the server running? (cargo run)"
        return $null
    }
}

function Count-Matches($html, $pattern) {
    return ([regex]::Matches($html, $pattern)).Count
}

Write-Host "`nSEO QA against $Base`n" -ForegroundColor Cyan

# 1) robots.txt
$r = Get-Page "/robots.txt"
if ($r) {
    if ($r.Content -match "Sitemap:") { Pass "robots.txt includes Sitemap" } else { Fail "robots.txt missing Sitemap" }
}

# 2) sitemap index + sub-sitemaps
$s = Get-Page "/sitemap.xml"
if ($s) {
    if ($s.Content -match "sitemapindex") { Pass "sitemap.xml is sitemap index" } else { Fail "sitemap.xml should be sitemap index" }
}
$palMap = Get-Page "/sitemap-pals.xml"
if ($palMap) {
    $palCount = Count-Matches $palMap.Content "<loc>"
    if ($palCount -ge 150) { Pass "sitemap-pals.xml has $palCount URLs" } else { Fail "sitemap-pals.xml expected 150+ URLs, got $palCount" }
}
$guideMap = Get-Page "/sitemap-guides.xml"
if ($guideMap) {
    $g = Count-Matches $guideMap.Content "<loc>"
    if ($g -ge 10) { Pass "sitemap-guides.xml has $g guide URLs" } else { Fail "sitemap-guides.xml expected 10+ URLs, got $g" }
}

# 3) Home page SEO
$homePage = Get-Page "/"
if ($homePage) {
    $h = $homePage.Content
    $titles = Count-Matches $h "<title>"
    if ($titles -eq 1) { Pass "home has exactly one <title>" } else { Fail "home has $titles title tags (expected 1)" }
    if ($h -match 'rel="canonical"') { Pass "home has canonical" } else { Fail "home missing canonical" }
    if ($h -match 'application/ld\+json') { Pass "home has JSON-LD" } else { Fail "home missing JSON-LD" }
    if ($h -match "About This Tool") { Pass "home has About section" } else { Fail "home missing About This Tool section" }
    if ($h -match 'id="routeSubtitle"') {
        $sub = [regex]::Match($h, 'id="routeSubtitle"[^>]*>([^<]+)').Groups[1].Value
        $subWords = ($sub -split '\s+').Count
        if ($subWords -ge 50 -and $subWords -le 60) { Pass "home H1 characteristics: $subWords words" } else { Fail "home H1 characteristics: $subWords words (want 50-60)" }
    }
    if ($h -match 'id="aboutSite"') {
        $about = [regex]::Match($h, 'id="aboutSite"[\s\S]*?<p>([\s\S]*?)</p>').Groups[1].Value
        $aboutWords = ($about -split '\s+').Count
        if ($aboutWords -ge 145 -and $aboutWords -le 155) { Pass "home description: $aboutWords words" } else { Fail "home description: $aboutWords words (want ~150)" }
    }
    if ($h -match "How do I breed Anubis\?") { Pass "home FAQ visible" } else { Fail "home FAQ missing" }
    if ($h -match '"name": "How do I breed Anubis\?"') { Pass "FAQ schema matches visible FAQ" } else { Fail "FAQ schema out of sync" }
}

# 4) Canonical breeding calculator route (old URL should redirect)
$redirectStatus = Get-RedirectStatus "/breeding-calculator"
if ($redirectStatus -ge 300 -and $redirectStatus -lt 400) {
    Pass "/breeding-calculator redirects (status $redirectStatus)"
} elseif ($null -ne $redirectStatus) {
    Fail "/breeding-calculator should redirect, got status $redirectStatus"
}
$breed = Get-Page "/palworld-breeding-calculator"
if ($breed) {
    if ($breed.Content -match "<h1[^>]*>Palworld Breeding Calculator Online</h1>") {
        Pass "palworld-breeding-calculator has unique H1"
    } else {
        Fail "palworld-breeding-calculator missing unique H1"
    }
    if ($breed.Content -match "About This Page") { Pass "palworld-breeding-calculator has route blurb" } else { Fail "palworld-breeding-calculator missing route blurb" }
    if ($breed.Content -match 'data-route-intro="') { Pass "breeding route preserves route intro attribute" } else { Fail "breeding route missing data-route-intro" }
}

# 5) Pals route
$pals = Get-Page "/pals"
if ($pals -and $pals.Content -match "<h1[^>]*>Palworld Pals Database</h1>") {
    Pass "pals route has unique H1"
} elseif ($pals) {
    Fail "pals route missing unique H1"
}

# 6) Pal detail page
$pal = Get-Page "/pal/anubis"
if ($pal) {
    if ($pal.Content -match "Anubis in Palworld") { Pass "pal detail page renders" } else { Fail "pal detail page content missing" }
}

# 7) Intent page + pal count
$intent = Get-Page "/fastest-anubis-breed"
if ($intent -and $intent.Content -match "Top parent pairs for Anubis") { Pass "intent page with dynamic combos" } elseif ($intent) { Fail "intent page missing dynamic combos" }
$boot = Get-Page "/api/bootstrap"
if ($boot) {
    $json = $boot.Content | ConvertFrom-Json
    if ($json.pals.Count -ge 150) { Pass "API has $($json.pals.Count) pals" } else { Fail "API has only $($json.pals.Count) pals (expected 150+)" }
}

# 8) API health
$api = Get-Page "/api/bootstrap"
if ($api -and $api.StatusCode -eq 200) { Pass "API bootstrap OK" } elseif ($api) { Fail "API bootstrap status $($api.StatusCode)" }

# 9) All SEO copy constants within word limits
$countScript = Join-Path $PSScriptRoot "count-seo-words.mjs"
if (Test-Path $countScript) {
    $lines = node $countScript 2>&1
    $bad = @()
    foreach ($line in $lines) {
        if ($line -match '^(\S+) (H1|DESC) (\d+)$') {
            $n = [int]$Matches[3]
            if ($Matches[2] -eq "H1" -and ($n -lt 50 -or $n -gt 60)) { $bad += $line }
            if ($Matches[2] -eq "DESC" -and ($n -lt 145 -or $n -gt 155)) { $bad += $line }
        }
    }
    if ($bad.Count -eq 0) { Pass "all seo_copy.rs blocks within 50-60 / ~150 word targets" } else { Fail "seo_copy out of range: $($bad -join '; ')" }
}

Write-Host ""
if ($Failed -eq 0) {
    Write-Host "All SEO QA checks passed." -ForegroundColor Green
    exit 0
} else {
    Write-Host "$Failed check(s) failed." -ForegroundColor Red
    exit 1
}
