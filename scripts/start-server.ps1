$ErrorActionPreference = "Stop"
$root = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $root

Get-Process -Name "palworld-breeding-calculator" -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 1

& (Join-Path $PSScriptRoot "prepare-rust.ps1")

$env:BASE_URL = "http://127.0.0.1:3000"
$release = Join-Path $root "target\release\palworld-breeding-calculator.exe"
$debug = Join-Path $root "target\debug\palworld-breeding-calculator.exe"
$exe = $release
if (-not (Test-Path $exe)) {
    Write-Host "Building release (faster)..." -ForegroundColor Yellow
    cargo build --release
    if ($LASTEXITCODE -ne 0) { exit 1 }
}
if (-not (Test-Path $exe)) {
    $exe = $debug
    if (-not (Test-Path $exe)) {
        Write-Host "Building debug..." -ForegroundColor Yellow
        cargo build
        if ($LASTEXITCODE -ne 0) { exit 1 }
        $exe = $debug
    }
}

Write-Host "Starting server (release when available)..." -ForegroundColor Cyan
Start-Process -FilePath $exe -WorkingDirectory $root -WindowStyle Normal

$ready = $false
for ($i = 0; $i -lt 45; $i++) {
    Start-Sleep -Seconds 1
    try {
        $r = Invoke-WebRequest "http://127.0.0.1:3000/api/bootstrap" -UseBasicParsing -TimeoutSec 3
        if ($r.StatusCode -eq 200) { $ready = $true; break }
    } catch { }
}

$url = "http://127.0.0.1:3000/palworld-breeding-calculator"
if ($ready) {
    Write-Host "Server OK. Opening browser..." -ForegroundColor Green
    Start-Process $url
} else {
    Write-Host "Server slow to start (index may still be building). Open manually:" -ForegroundColor Yellow
    Write-Host $url
}
exit 0
