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
