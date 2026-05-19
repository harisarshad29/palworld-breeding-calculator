# Palworld Breeding Calculator (Rust)

Rust server + web UI. **No Netlify static site.**

## Run

Double-click **START-SERVER.bat** or:

```bash
cargo run
```

http://127.0.0.1:3000/palworld-breeding-calculator

## Deploy

See **DEPLOY-RUST.txt** and **render.yaml**.

## Reset to Rust-only

Double-click **RESET.bat** (removes static-site / wordpress folders).

## Layout

- `src/` — Rust server + SEO
- `index.html`, `app.js`, `styles.css` — frontend
- `data/`, `assets/pals/` — game data + images
- `seo/` — off-page playbook, outreach, backlink tracker

## SEO (on-page + off-page)

### On-page (built into Rust)

- Per-route titles, meta descriptions (~150 words), canonicals, Open Graph
- JSON-LD: FAQ, Article, Breadcrumb, HowTo, WebSite
- Sitemap index: `/sitemap.xml` (+ pals, guides, combos, hubs)
- Programmatic pages: `/pal/:name`, `/combo/:a/:b`, guides, how-to-breed hubs
- Sitewide footer with internal links (`src/seo_copy.rs`)

### Off-page (you run daily)

1. Deploy live with `BASE_URL` — see `DEPLOY-ALL.txt`
2. Google Search Console — see `seo/SEARCH-CONSOLE-SETUP.md`
3. Daily routine — `powershell -File scripts\daily-seo.ps1`
4. Full plan (Urdu/English) — `seo/KAM-KARO-ON-OFF-PAGE.md`
5. Log backlinks — `seo/backlink-tracker.csv`

### Verify live SEO

```powershell
$env:LIVE_SITE_URL="https://palworld-breeding-calculator.us"
powershell -File scripts\verify-live-seo.ps1
```
