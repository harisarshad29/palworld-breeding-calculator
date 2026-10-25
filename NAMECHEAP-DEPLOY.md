# Namecheap par website chalana (Render ki jagah)

Site ab poori static ban jati hai: HTML pages, sitemaps, `api/*.json` aur images.
Namecheap ki shared hosting (cPanel / Apache) par koi server program nahi chalana
parta — Render band kiya ja sakta hai.

## 1. Upload ki file lena

GitHub → **Actions** → sab se naya **Check calculator** run (main branch) →
neeche **Artifacts** → **namecheap-site** download karein. Is ke andar
`namecheap-site.zip` hai (~66 MB).

Ya apne computer par:

```sh
cargo build --release
BASE_URL=https://www.palworld-breeding-calculator.us ./target/release/palworld-breeding-calculator export dist
```

`dist/` folder ka saara maal (`.htaccess` samet) upload karna hai.

## 2. cPanel mein upload

1. Namecheap → Hosting List → **Go to cPanel** → **File Manager** → `public_html`.
2. `public_html` mein purani files hon to pehle backup le lein.
3. **Upload** → `namecheap-site.zip` → wapas File Manager → zip par right-click → **Extract**.
4. Settings mein **Show Hidden Files** on karein aur check karein ke `public_html/.htaccess` maujood hai.
5. Zip file delete kar dein.

## 3. SSL (pehle yeh, phir domain)

cPanel → **SSL/TLS Status** → domain (www aur bina www) ke liye **Run AutoSSL**.
`.htaccess` har request ko `https://www…` par bhejta hai, is liye SSL zaroori hai.

## 4. Domain ko Render se hata kar Namecheap hosting par lana

Namecheap → Domain List → `palworld-breeding-calculator.us` → **Manage**:

- Asaan tareeqa: **Nameservers** → **Namecheap Web Hosting DNS**.
- Ya **Advanced DNS** mein Render wale records hatayein
  (`A @ 216.24.57.1`, `CNAME www → palworld-breeding-calculator.onrender.com`)
  aur cPanel ke home page par dikhne wala **Shared IP Address** lagayein:
  `A @ <hosting IP>` aur `CNAME www → palworld-breeding-calculator.us`.

DNS badalne mein 30 minute se kuch ghante lag sakte hain.

## 5. Check karein

- `https://www.palworld-breeding-calculator.us/` khulta hai
- `http://palworld-breeding-calculator.us/pal/anubis` → `https://www…/pal/anubis` par jata hai
- `/pal/anubis`, `/sitemap.xml`, `/robots.txt`, `/api/bootstrap` sab 200 dete hain
- Calculator, reverse lookup aur chain planner kaam karte hain (sab browser mein chalte hain)

Phir Google Search Console mein `sitemap.xml` dobara submit karein.

## 6. Render band karna

Sab kuch Namecheap par chal jaye to Render dashboard se service **Delete** ya
**Suspend** kar dein, taake kharcha band ho jaye.

## Aage ke updates

- Har `main` push par CI naya `namecheap-site` zip banata hai — wohi upload karein.
- Ya automatic: GitHub → Settings → Secrets and variables → Actions mein
  `NAMECHEAP_FTP_SERVER`, `NAMECHEAP_FTP_USERNAME`, `NAMECHEAP_FTP_PASSWORD`
  daal dein (cPanel → **FTP Accounts**). Phir `Deploy to Namecheap` workflow har
  `main` push par sirf badli hui files upload karega. Pehli dafa ~12,000 files
  jati hain, is mein waqt lagta hai.

## Kya badla aur kya nahi

- Har page, sitemap aur API jawab wahi hai jo Rust server deta tha (608 pages aur
  saari API files Apache par byte-for-byte check ki gayin).
- Chain planner ab browser mein chalta hai (server ke `/api/chain` jaisa hi nateeja).
- ~11,000 parent-pair pages pehle se bane hue hain (jo site par link hain). Baqi
  kisi pair ka URL calculator par us pair ke saath khulta hai.
- Rust server ab bhi local testing ke liye `cargo run` se chalta hai.
