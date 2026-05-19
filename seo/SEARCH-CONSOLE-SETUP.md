# Google Search Console Setup

## 1. Property add karo

1. Open https://search.google.com/search-console
2. **Add property** → Domain ya URL prefix:
   - Recommended: `https://palworld-breeding-calculator.us` (URL prefix)
3. Verify ownership:
   - **DNS TXT** (domain property), ya
   - **HTML file / meta tag** (URL prefix)

## 2. Sitemap submit

After site is live on Render with correct `BASE_URL`:

```
https://YOUR-DOMAIN.com/sitemap.xml
```

Sitemap index includes:

- `sitemap-pages.xml`
- `sitemap-pals.xml`
- `sitemap-guides.xml`
- `sitemap-combos.xml`
- `sitemap-hubs.xml`

## 3. Important URLs to request indexing

Submit these once after launch:

- `/palworld-breeding-calculator`
- `/guides/best-breeding-combos`
- `/how-to-breed/anubis`
- `/pal/anubis`
- `/pal-pages`

## 4. Weekly reports to check

| Report | Action |
|--------|--------|
| **Performance → Queries** | Low CTR → rewrite title/meta |
| **Pages** | Not indexed → fix links / content |
| **Links → External links** | Backlink growth |
| **Core Web Vitals** | Speed issues → images / hosting |

## 5. Bing (optional, 5 min)

https://www.bing.com/webmasters — import from Google or add sitemap manually.

## 6. Common mistakes

- Sitemap submitted while site still on Netlify static (404 routes)
- `BASE_URL` not set → wrong canonical in HTML
- Only homepage indexed — fix internal links (footer + guides now in project)
