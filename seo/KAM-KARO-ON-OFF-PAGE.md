# On-Page + Off-Page SEO — Poora Action Plan (Urdu + English)

Yeh file aap ke project ke saath use karo. Code mein on-page SEO zyada tar ready hai; ranking ke liye **live deploy + Google Search Console + backlinks** zaroori hain.

---

## Part A — On-Page SEO (site par)

### Already in project (code)

| Feature | Location |
|---------|----------|
| Unique title, meta, canonical per route | `src/main.rs` + `src/seo_copy.rs` |
| JSON-LD (FAQ, Article, Breadcrumb, HowTo) | `src/main.rs` |
| Sitemap index + split sitemaps | `/sitemap.xml` |
| Pal / combo / guide programmatic pages | Rust routes |
| Internal links (home content + footer) | `index.html`, `seo_copy::SITE_FOOTER_HTML` |
| ~150 word route descriptions | `src/seo_copy.rs` |

### Aap ko karna hai (manual)

1. **Live deploy** — `DEPLOY-ALL.txt` follow karo (Render + `BASE_URL` + DNS).
2. **Google Search Console** — `seo/SEARCH-CONSOLE-SETUP.md`.
3. **Verify** — `powershell -File scripts\verify-live-seo.ps1`
4. **Har hafte** — Search Console → Queries → low CTR pages → title/description update in `src/main.rs` / `GUIDE_PAGES`.

### On-page weekly checklist

- [ ] Koi page `localhost` canonical to nahi dikha raha
- [ ] `/sitemap.xml` sab sitemaps list karta hai
- [ ] Naye guides / pals add kiye to sitemap auto update (Rust)
- [ ] High-impression pages par content 800+ words (guides)

---

## Part B — Off-Page SEO (site ke bahar)

Google backlinks **crawl** se discover karta hai. Reporting **Search Console → Links** mein 1–4 hafte baad aati hai.

### Daily (15–30 min)

1. `seo/backlink-tracker.csv` kholo — 5 outreach rows add/update karo.
2. `seo/outreach-templates.md` se message bhejo (blog, YouTube, Discord).
3. 1 nayi URL Search Console mein “Request indexing”.

### Weekly

1. Search Console: impressions, clicks, indexed pages.
2. 3–5 live backlinks track karo (quality > quantity).
3. 1 community post (helpful, not spam) with link to calculator or guide.

### Files

| File | Use |
|------|-----|
| `seo/backlink-tracker.csv` | Har link / outreach log |
| `seo/outreach-templates.md` | Copy-paste messages |
| `seo/ranking-sprint-30-days.md` | 30-day routine |
| `scripts/daily-seo.ps1` | Daily reminder checklist |

### Backlink rules

- Niche gaming / Palworld sites prefer karo.
- Spam comment / PBN avoid — penalty risk.
- Anchor mix: brand + partial keyword + natural (“this tool”).

---

## Part C — Realistic ranking timeline

| Time | Expect |
|------|--------|
| Week 1–2 | Indexing start (100+ URLs if live) |
| Week 3–6 | Long-tail rankings (“breed anubis palworld”) |
| Month 3+ | Competitive terms slow vs palworld.gg |

Pehle **long-tail** jeeto, phir main keywords.

---

## Quick commands

```powershell
# Local server
cargo run

# Live SEO audit (after deploy)
$env:LIVE_SITE_URL="https://palworld-breeding-calculator.us"
powershell -File scripts\verify-live-seo.ps1

# Daily SEO reminder
powershell -File scripts\daily-seo.ps1
```
