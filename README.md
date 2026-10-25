# Palworld Breeding Calculator

Rust server and browser tools for species prediction, reverse lookup, and breeding
routes from an owned roster. No account or game-save upload is required.

## Run

```sh
cargo run --locked
```

Open http://127.0.0.1:3000/palworld-breeding-calculator.

## Data and calculation rules

The pinned PalCalc community snapshot contains 299 species, 44,850 unordered
parent pairs, and 44,851 outcomes. Browser and server calculations use the same
compact source table. Same-species pairs are supported. Katress + Wixen requires
both parent genders because the outcome changes with orientation.

See [data/SOURCES.md](data/SOURCES.md), [data/provenance.json](data/provenance.json),
and the retained [MIT notice](licenses/PalCalc-MIT.txt). The snapshot does not
declare a game build itself, but it was cross-checked against PalSphere's
Palworld 1.0.5 extract with no differences (details in SOURCES.md). After each
game patch, bump `GAME_VERSION` / `DATA_UPDATED` in `src/main.rs` only once the
comparison has been re-run. Refresh with
`python scripts/import_palcalc.py` after reviewing the pinned source commit.

The route planner uses the selected owned species, assumes suitable genders are
available, and searches up to 15 steps and 8,000 inventories. A failed bounded
search is not proof that no route exists. Every returned step requires both
parents to be available first. The capture score is a breeding-power heuristic,
not an actual capture probability.

## Validate

```sh
cargo test --locked
cargo build --release --locked
node --check app.js
node tests/frontend-calculator.cjs
npm ci --ignore-scripts
npx playwright install chromium
# Run the Rust server in another terminal, then:
npm run test:browser
```

Browser checks exercise desktop and 390px mobile layouts, same-species results,
gender-specific outcomes, swapping, reverse lookup, owned-roster chains, theme
switching, and horizontal overflow. Screenshots are saved to `test-results/` and
uploaded by CI. `BROWSER_BASE_URL` overrides the test server origin.

## Hosting on Namecheap (static export)

The site is published as static files on Namecheap shared hosting (Apache), so no
always-on server is needed:

```sh
cargo build --release --locked
BASE_URL=https://www.palworld-breeding-calculator.us ./target/release/palworld-breeding-calculator export dist
```

`export` renders every sitemap page and every internally linked page through the
same router the server uses, writes `api/*.json` for the calculator, copies assets,
and generates `.htaccess` (https + www, clean URLs, old redirects, 404 page). The
chain planner runs in the browser. CI uploads the result as the `namecheap-site`
artifact; step-by-step upload and DNS instructions are in
[NAMECHEAP-DEPLOY.md](NAMECHEAP-DEPLOY.md).

### Netlify (free alternative)

`export` also writes `_redirects` and `_headers`, so the same output runs on Netlify.
`.github/workflows/netlify-site.yml` force-pushes the export to the `netlify-site`
branch on every `main` push; link the Netlify project to that branch (no build
command, publish directory `.`).

## Hosting and health (Render, legacy)

`render.yaml` builds the locked release, declares the preferred `www` domain,
waits for CI checks, and uses `/health` for readiness. Apply or sync the Blueprint
in the existing Render service; repository settings alone do not prove that the
service configuration was applied. Keep `BASE_URL` equal to the public HTTPS
origin for canonical and sitemap consistency.

`GET /health` reports readiness, species count, the source commit, and the Render
application commit when available. `GET /api/bootstrap` includes data provenance.
The legacy Netlify redirect configuration is retained until the existing hosting
and domain bindings can be checked; do not attach the production domain to both
providers or enable a redirect loop.

On 25 October 2026 the public site displayed "This service has been suspended by
its owner." Resume the existing Render service and deploy the tested `main`
commit before treating repository changes as live. Confirm `/health`, both
custom-domain forms, canonical tags, sitemap URLs, and primary tool flows after
deployment. No paid plan change is required or authorized by this repository.
