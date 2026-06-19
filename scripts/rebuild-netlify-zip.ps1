$root = Resolve-Path (Join-Path $PSScriptRoot "..")
$zip = Join-Path $root "NETLIFY-REDIRECT-DEPLOY.zip"
if (Test-Path $zip) { Remove-Item $zip -Force }
Compress-Archive -Path (Join-Path $root "netlify.toml"), (Join-Path $root "netlify-redirect-site") -DestinationPath $zip -Force
Write-Host "Created: $zip ($((Get-Item $zip).Length) bytes)" -ForegroundColor Green
