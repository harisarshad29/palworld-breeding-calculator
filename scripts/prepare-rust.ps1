$ErrorActionPreference = "Stop"
$root = Resolve-Path (Join-Path $PSScriptRoot "..")

$required = @(
  "index.html", "app.js", "styles.css", "Cargo.toml", "favicon.svg",
  "data\pals.json", "data\special_combos.json", "data\pal_locations.json",
  "assets\pals\placeholder.svg", "src\main.rs"
)
foreach ($rel in $required) {
  if (-not (Test-Path (Join-Path $root $rel))) {
    Write-Error "Missing required file: $rel"
  }
}

$webp = (Get-ChildItem (Join-Path $root "assets\pals\*.webp") -EA SilentlyContinue).Count
if ($webp -lt 100) {
  Write-Warning "Only $webp pal images in assets/pals (expected ~195). Run: node scripts/download-pal-icons.js"
}

Write-Host "Project OK ($webp pal images)." -ForegroundColor Green
exit 0
