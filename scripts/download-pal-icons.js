const fs = require("fs");
const path = require("path");

function palSlug(name) {
  return name
    .toLowerCase()
    .trim()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
}

const palsFile = path.join(process.cwd(), "data", "pals.json");
const pals = JSON.parse(fs.readFileSync(palsFile, "utf8")).pals;

const outDir = path.join(process.cwd(), "assets", "pals");
fs.mkdirSync(outDir, { recursive: true });

async function fetchText(url) {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`Failed to fetch ${url} (${response.status})`);
  }
  return response.text();
}

async function fetchBuffer(url) {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`Failed to fetch image ${url} (${response.status})`);
  }
  return Buffer.from(await response.arrayBuffer());
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

const SLUG_ALIASES = {
  xenolord: ["xenolord", "xeno-lord"],
  warsect: ["warsect", "war-sect"],
  nyafia: ["nyafia", "nyafia-pal"],
};

async function tryDownloadFromWikily(slug) {
  const pageUrl = `https://wikily.gg/palworld/pals/${slug}/`;
  const pageContent = await fetchText(pageUrl);
  const match = pageContent.match(/"image":\["(https:\/\/r2\.wikily\.gg[^"]+)"\]/);
  if (!match) {
    return null;
  }
  return fetchBuffer(match[1]);
}

async function downloadPal(name) {
  const slug = palSlug(name);
  const outPath = path.join(outDir, `${slug}.webp`);
  if (fs.existsSync(outPath)) {
    return { name, slug, status: "skipped" };
  }

  const slugCandidates = SLUG_ALIASES[slug] || [slug];
  for (const candidate of slugCandidates) {
    try {
      const data = await tryDownloadFromWikily(candidate);
      if (data) {
        fs.writeFileSync(outPath, data);
        return { name, slug, status: "ok", usedSlug: candidate };
      }
    } catch {
      // try next alias
    }
  }

  return { name, slug, status: "no-image-url" };
}

async function downloadAll() {
  let ok = 0;
  let skipped = 0;
  let failed = 0;
  const failures = [];

  for (let i = 0; i < pals.length; i++) {
    const { name } = pals[i];
    try {
      const result = await downloadPal(name);
      if (result.status === "ok") {
        ok++;
        console.log(`[ok] ${name} -> ${result.slug}.webp`);
      } else if (result.status === "skipped") {
        skipped++;
      } else {
        failed++;
        failures.push(name);
        console.warn(`[miss] ${name} (${result.slug})`);
      }
    } catch (error) {
      failed++;
      failures.push(name);
      console.warn(`[fail] ${name}: ${error.message}`);
    }
    await sleep(120);
  }

  console.log(`\nDone: ${ok} downloaded, ${skipped} skipped, ${failed} missing/failed.`);
  if (failures.length) {
    fs.writeFileSync(
      path.join(outDir, "missing-icons.json"),
      JSON.stringify(failures, null, 2),
      "utf8"
    );
    console.log(`Missing list: assets/pals/missing-icons.json`);
  }
}

downloadAll().catch((error) => {
  console.error(error);
  process.exit(1);
});
