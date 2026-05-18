$ErrorActionPreference = "Stop"
$root = Resolve-Path (Join-Path $PSScriptRoot "..")

$appPath = Join-Path $root "app.js"
$app = Get-Content $appPath -Raw
if ($app -match 'bg-pal-sticker' -and $app -notmatch 'kidBg\.innerHTML = ""') {
  Write-Host "Stickers already enabled in app.js"
} elseif ($app -match 'function renderKidBackground\(\) \{\s+kidBg\.innerHTML = ""') {
  $app = $app -replace '(?s)function renderKidBackground\(\) \{\s+kidBg\.innerHTML = "";\s+\}', @'
function renderKidBackground() {
  if (!kidBg) return;
  const pals = ["Lamball","Cattiva","Chikipi","Foxparks","Pengullet","Anubis","Jetragon","Frostallion","Blazamut","Suzaku","Necromus","Paladius","Relaxaurus","Penking","Elizabee","Grizzbolt","Lyleen","Mossanda","Azurobe","Incineram","Beakon","Sibelyx","Astegon","Shadowbeak","Bellanoir","Kitsun","Rooby","Daedream"];
  const slots = [
    {l:1,t:4,s:76,r:-12},{l:3,t:28,s:58,r:6},{l:2,t:52,s:64,r:-8},{l:4,t:76,s:54,r:10},{l:1,t:90,s:62,r:-15},
    {l:88,t:3,s:72,r:14},{l:91,t:26,s:56,r:-9},{l:89,t:48,s:68,r:11},{l:92,t:70,s:60,r:-7},{l:87,t:88,s:66,r:16},
    {l:14,t:2,s:50,r:8},{l:78,t:2,s:48,r:-11},{l:8,t:42,s:44,r:-5},{l:84,t:38,s:46,r:7},{l:6,t:64,s:42,r:12},
    {l:86,t:58,s:44,r:-13},{l:18,t:86,s:40,r:6},{l:76,t:84,s:42,r:-8},{l:22,t:14,s:38,r:-4},{l:72,t:16,s:38,r:5},
    {l:10,t:18,s:36,r:9},{l:82,t:20,s:36,r:-6}
  ];
  const frag = document.createDocumentFragment();
  slots.forEach((sl, i) => {
    const img = document.createElement("img");
    img.className = "bg-pal-sticker";
    img.src = getPalImageUrl(pals[i % pals.length]);
    img.alt = "";
    img.loading = "lazy";
    img.style.cssText = `left:${sl.l}%;top:${sl.t}%;width:${sl.s}px;height:${sl.s}px;transform:rotate(${sl.r}deg);animation-delay:${(i%8)*0.22}s;animation-duration:${7+(i%6)}s;`;
    img.onerror = () => { img.onerror = null; img.src = PAL_PLACEHOLDER; };
    frag.appendChild(img);
  });
  kidBg.replaceChildren(frag);
}
'@
  if ($app -notmatch 'renderKidBackground\(\);\s*\r?\n\s*renderPalGrid') {
    $app = $app -replace '(renderStatsBar\(\);)', "`$1`n  renderKidBackground();"
  }
  Set-Content $appPath $app -NoNewline
  Write-Host "Stickers enabled in app.js"
}

$cssPath = Join-Path $root "styles.css"
(Get-Content $cssPath -Raw) -replace 'opacity: 0\.18', 'opacity: 0.28' | Set-Content $cssPath -NoNewline

$gi = Join-Path $root ".gitignore"
$giText = Get-Content $gi -Raw
foreach ($line in @("static-site/", "wordpress-plugin/")) {
  if ($giText -notlike "*$line*") { Add-Content $gi $line }
}

$idx = Join-Path $root "index.html"
$html = Get-Content $idx -Raw
if ($html -notlike "*favicon*") {
  $html = $html -replace '(<meta name="viewport"[^>]+>)', "`$1`n    <link rel=`"icon`" href=`"/favicon.svg`" type=`"image/svg+xml`" />"
  Set-Content $idx $html -NoNewline
}

Write-Host "Rust-only UI patches done."
