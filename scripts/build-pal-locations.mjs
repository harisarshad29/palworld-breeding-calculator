/**
 * Builds data/pal_locations.json for all pals in data/pals.json.
 * Run: node scripts/build-pal-locations.mjs
 * Optional: node scripts/build-pal-locations.mjs --scrape  (slow; fetches palworld.gg)
 */
import { readFileSync, writeFileSync, mkdirSync } from "fs";
import { dirname, join } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = join(__dirname, "..");
const pals = JSON.parse(readFileSync(join(root, "data/pals.json"), "utf8")).pals;
const scrape = process.argv.includes("--scrape");

/** Map coordinates from palworld.gg interactive map / community pins */
const KNOWN = {
  Lamball: { area: "Windswept Hills", coords: "(-39, -158)" },
  Cattiva: { area: "Windswept Hills", coords: "(-52, -140)" },
  Chikipi: { area: "Sea Breeze Archipelago", coords: "(12, -221)" },
  Lifmunk: { area: "Windswept Hills", coords: "(-12, -124)" },
  Mau: { area: "Windswept Hills", coords: "(-45, -150)" },
  Teafant: { area: "Windswept Hills", coords: "(-35, -165)" },
  Foxparks: { area: "Sea Breeze Archipelago", coords: "(44, -188)" },
  Fuack: { area: "Sea Breeze Archipelago", coords: "(8, -195)" },
  Sparkit: { area: "Sea Breeze Archipelago", coords: "(25, -210)" },
  Depresso: { area: "Windswept Hills", coords: "(-28, -145)" },
  Pengullet: { area: "Forgotten Island", coords: "(-142, -206)" },
  Jolthog: { area: "Eastern Wild Island", coords: "(212, -58)" },
  Tanzee: { area: "Verdant Brook", coords: "(96, -32)" },
  Daedream: { area: "Hypocrite Hill", coords: "(-118, -84)" },
  Rooby: { area: "Scorched Path", coords: "(148, -103)" },
  Gumoss: { area: "Verdant Brook", coords: "(72, -18)" },
  Hoocrates: { area: "Sea Breeze Archipelago", coords: "(18, -200)" },
  Eikthyrdeer: { area: "Moonless Shore", coords: "(92, -166)" },
  Mozzarina: { area: "Moonless Shore", coords: "(85, -155)" },
  Direhowl: { area: "Bamboo Groves", coords: "(108, -88)" },
  Galeclaw: { area: "Bamboo Groves", coords: "(125, -72)" },
  Gorirat: { area: "Bamboo Groves", coords: "(118, -95)" },
  Melpaca: { area: "Bamboo Groves", coords: "(88, -55)" },
  Reindrix: { area: "Frostbound Mountains", coords: "(-165, 280)" },
  Celaray: { area: "Bamboo Groves", coords: "(102, -48)" },
  Digtoise: { area: "Dessicated Desert", coords: "(320, 180)" },
  Dumud: { area: "Dessicated Desert", coords: "(305, 195)" },
  Tombat: { area: "No. 2 Wildlife Sanctuary", coords: "(-655, -125)" },
  Foxcicle: { area: "Land of Absolute Zero (edges)", coords: "(-320, 420)" },
  Petallia: { area: "Verdant Brook", coords: "(78, -25)" },
  Kitsun: { area: "Bamboo Groves", coords: "(132, -68)" },
  Dinossom: { area: "Mount Obsidian (foothills)", coords: "(-450, -380)" },
  Surfent: { area: "Ravine Entrance", coords: "(-55, -310)" },
  Anubis: { area: "Twilight Dunes (Alpha Boss)", coords: "(-134, -94)" },
  Incineram: { area: "Mount Obsidian", coords: "(-470, -400)" },
  Bushi: { area: "Bamboo Groves (Alpha)", coords: "(95, -420)" },
  Vanwyrm: { area: "Mount Obsidian", coords: "(-500, -350)" },
  Penking: { area: "Frostbound Mountains", coords: "(-180, 320)" },
  Grintale: { area: "Frostbound Mountains", coords: "(-195, 305)" },
  Azurobe: { area: "No. 1 Wildlife Sanctuary", coords: "(-495, 215)" },
  Mossanda: { area: "No. 1 Wildlife Sanctuary", coords: "(-480, 220)" },
  Nitewing: { area: "No. 1 Wildlife Sanctuary", coords: "(-505, 200)" },
  Kingpaca: { area: "No. 1 Wildlife Sanctuary", coords: "(-490, 230)" },
  Wumpo: { area: "Land of Absolute Zero", coords: "(-340, 490)" },
  Sibelyx: { area: "Land of Absolute Zero", coords: "(-365, 475)" },
  Pyrin: { area: "Mount Obsidian", coords: "(-487, -418)" },
  Ragnahawk: { area: "Mount Obsidian", coords: "(-460, -430)" },
  Faleris: { area: "Mount Obsidian", coords: "(-475, -405)" },
  Blazamut: { area: "Scorching Mineshaft", coords: "(-436, -531)" },
  Suzaku: { area: "Dessicated Desert", coords: "(404, 255)" },
  Jormuntide: { area: "No. 2 Wildlife Sanctuary", coords: "(-668, -113)" },
  Relaxaurus: { area: "No. 3 Wildlife Sanctuary", coords: "(-730, -450)" },
  Mammorest: { area: "No. 3 Wildlife Sanctuary", coords: "(-720, -440)" },
  Lyleen: { area: "No. 3 Wildlife Sanctuary", coords: "(-755, -420)" },
  Grizzbolt: { area: "No. 1 Wildlife Sanctuary", coords: "(-510, 190)" },
  Helzephyr: { area: "No. 2 Wildlife Sanctuary", coords: "(-680, -95)" },
  Beakon: { area: "No. 2 Wildlife Sanctuary", coords: "(-650, -105)" },
  Frostallion: { area: "Land of Absolute Zero", coords: "(-357, 508)" },
  Jetragon: { area: "Mount Obsidian (Alpha)", coords: "(-789, -321)" },
  Necromus: { area: "Dessicated Desert (Alpha)", coords: "(447, 680)" },
  Paladius: { area: "Dessicated Desert (Alpha)", coords: "(446, 679)" },
  Shadowbeak: { area: "No. 3 Wildlife Sanctuary (Alpha)", coords: "(-750, -380)" },
  Bellanoir: { area: "Bellanoir Raid", coords: "Raid / dungeon" },
  Xenolord: { area: "Feybreak (raid)", coords: "Raid / dungeon" },
  Quivern: { area: "Dessicated Desert", coords: "(380, 240)" },
  Warsect: { area: "No. 2 Wildlife Sanctuary", coords: "(-660, -120)" },
  Elizabee: { area: "No. 2 Wildlife Sanctuary", coords: "(-645, -130)" },
  Reptyro: { area: "Mount Obsidian", coords: "(-455, -425)" },
  Astegon: { area: "No. 3 Wildlife Sanctuary", coords: "(-735, -435)" },
  Cryolinx: { area: "Land of Absolute Zero", coords: "(-350, 500)" },
  Orserk: { area: "No. 2 Wildlife Sanctuary", coords: "(-675, -100)" },
  Neptilius: { area: "Deep ocean (Feybreak)", coords: "(varies)" },
};

const BIOMES_BY_TIER = [
  { maxPower: 100, area: "Legendary boss / raid", coords: "Check map pin" },
  { maxPower: 200, area: "Dessicated Desert & deep zones", coords: "(400, 300)" },
  { maxPower: 350, area: "Wildlife Sanctuaries & Mount Obsidian", coords: "(-500, -200)" },
  { maxPower: 550, area: "No. 1–2 Wildlife Sanctuary", coords: "(-520, 180)" },
  { maxPower: 750, area: "Frostbound Mountains & ravines", coords: "(-180, 300)" },
  { maxPower: 950, area: "Bamboo Groves & Moonless Shore", coords: "(100, -120)" },
  { maxPower: 1150, area: "Verdant Brook & Hypocrite Hill", coords: "(80, -60)" },
  { maxPower: 99999, area: "Windswept Hills & Sea Breeze", coords: "(-40, -160)" },
];

const VARIANT_SUFFIX =
  /\s+(Cryst|Noct|Lux|Ignis|Aqua|Terra|Botan|Ryu|Libero|Noct|Cryst)$/i;

function baseName(name) {
  return name.replace(VARIANT_SUFFIX, "").trim();
}

function variantLabel(name) {
  const m = name.match(VARIANT_SUFFIX);
  return m ? m[1] : null;
}

function tierFallback(power) {
  for (const t of BIOMES_BY_TIER) {
    if (power <= t.maxPower) {
      return { area: t.area, coords: t.coords };
    }
  }
  return { area: "Palpagos Island (overworld)", coords: "(-40, -160)" };
}

function resolveLocation(name, power) {
  if (KNOWN[name]) {
    return { ...KNOWN[name] };
  }

  const base = baseName(name);
  const variant = variantLabel(name);

  if (KNOWN[base]) {
    const loc = { ...KNOWN[base] };
    if (variant) {
      loc.area = `${loc.area} (${variant} variant)`;
    }
    return loc;
  }

  // Element-ish hints
  if (/aqua|jell|finsider|polapup|neptilius/i.test(name)) {
    return { area: "Sea Breeze & coastal waters", coords: "(10, -210)" };
  }
  if (/cryst|frostallion|icelyn|munchill|foxparks cryst/i.test(name)) {
    return { area: "Frostbound / icy biomes", coords: "(-200, 320)" };
  }
  if (/ignis|blazamut|faleris|pyrin|reptyro|blazehowl/i.test(name)) {
    return { area: "Mount Obsidian & volcanic zones", coords: "(-480, -410)" };
  }
  if (/noct|shadow|dazzi noct|bushi noct/i.test(name)) {
    return { area: "Night spawns (same region as base)", coords: "See map at night" };
  }
  if (/bellanoir|xeno|raid/i.test(name)) {
    return { area: "Raid / dungeon content", coords: "Not overworld" };
  }

  return tierFallback(power);
}

async function scrapeFromGg(name) {
  const slug = name.toLowerCase().replace(/[^a-z0-9]+/g, "-");
  try {
    const res = await fetch(`https://palworld.gg/pal/${slug}`, {
      headers: { "User-Agent": "PalworldBreedingCalculator/1.0" },
    });
    if (!res.ok) return null;
    const html = await res.text();
    const biomes = [];
    const checks = [
      "Windswept Hills",
      "Sea Breeze Archipelago",
      "Bamboo Groves",
      "Mount Obsidian",
      "Dessicated Desert",
      "Land of Absolute Zero",
      "Wildlife Sanctuary",
      "Forgotten Island",
      "Eastern Wild Island",
      "Frostbound Mountains",
      "Twilight Dunes",
    ];
    for (const b of checks) {
      if (html.includes(b)) biomes.push(b);
    }
    if (biomes.length) {
      return { area: biomes.slice(0, 2).join(" · "), coords: "See palworld.gg/map" };
    }
  } catch {
    /* ignore */
  }
  return null;
}

async function main() {
  const locations = {};

  for (let i = 0; i < pals.length; i++) {
    const { name, power } = pals[i];
    let loc = resolveLocation(name, power);

    if (scrape && !KNOWN[name]) {
      const scraped = await scrapeFromGg(name);
      if (scraped) loc = scraped;
      await new Promise((r) => setTimeout(r, 300));
      process.stdout.write(`\r[${i + 1}/${pals.length}] ${name}`);
    }

    locations[name] = loc;
  }

  if (scrape) console.log("");

  const outPath = join(root, "data/pal_locations.json");
  mkdirSync(dirname(outPath), { recursive: true });
  writeFileSync(outPath, JSON.stringify({ locations }, null, 2));

  const withCoords = Object.values(locations).filter(
    (l) => l.coords && !l.coords.includes("map") && !l.coords.includes("Check") && !l.coords.includes("varies")
  ).length;

  console.log(`Wrote ${Object.keys(locations).length} locations (${withCoords} with map-style coords) → ${outPath}`);
}

main();
