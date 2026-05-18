$ErrorActionPreference = "Stop"
$root = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $root

Write-Host "=== FULL RESET: Rust only ===" -ForegroundColor Cyan
Get-Process -Name "palworld-breeding-calculator" -EA SilentlyContinue | Stop-Process -Force

git reset --hard d1ac04f
foreach ($dir in @("static-site", "wordpress-plugin")) {
  $p = Join-Path $root $dir
  if (Test-Path $p) { Remove-Item $p -Recurse -Force; Write-Host "Removed $dir" }
}

& (Join-Path $PSScriptRoot "apply-rust-only.ps1")
Write-Host "Reset complete. Use START-SERVER.bat" -ForegroundColor Green
