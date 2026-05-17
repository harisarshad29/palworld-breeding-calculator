import { calculateChild, combinationsForTarget, findPal } from "./logic.js";
import { slugToTitle } from "./seo-routes.js";

function escapeHtml(text) {
  return String(text)
    .replace(/&/g, "&amp;")
    .replace(/"/g, "&quot;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}

function palSlug(name) {
  return name.toLowerCase().trim().replace(/[^a-z0-9]+/g, "-").replace(/^-+|-+$/g, "");
}

function tierNote(power) {
  if (power <= 50) return "a high-tier legendary target";
  if (power <= 100) return "a mid-to-late game target";
  return "an accessible early-to-mid target";
}

export function findPalBySlug(pals, slug) {
  const q = String(slug || "").toLowerCase();
  return pals.find((p) => palSlug(p.name) === q) || null;
}

export function buildPalRouteContent(pal, locations, combos) {
  const slug = palSlug(pal.name);
  const loc = locations[pal.name];
  const area = loc?.area || "Palpagos Island (search in-game map)";
  const coords = loc?.coords || "Open Map tab";
  const rows = combos
    .slice(0, 20)
    .map(
      (pair) =>
        `<tr><td><a href="/combo/${palSlug(pair.a)}/${palSlug(pair.b)}">${escapeHtml(pair.a)}</a></td><td><a href="/combo/${palSlug(pair.a)}/${palSlug(pair.b)}">${escapeHtml(pair.b)}</a></td><td>${escapeHtml(pair.method)}</td></tr>`
    )
    .join("");
  const more =
    combos.length > 20 ? `<p class="muted">Showing 20 of ${combos.length} combinations. Use reverse calculator for full list.</p>` : "";

  return {
    title: `${pal.name} Breeding Guide – Best Combos, Parents & Location | Palworld`,
    h1: `${pal.name} Breeding Guide`,
    badge: pal.name,
    h1Intro: `Complete ${pal.name} breeding reference with ${combos.length} parent pair routes, breeding power ${pal.power}, map location, and calculator shortcuts. ${pal.name} is ${tierNote(pal.power)}.`,
    pageDesc: `This page lists ${combos.length} parent pair routes that can produce ${pal.name} using live breeding logic. With breeding power ${pal.power}, ${pal.name} is ${tierNote(pal.power)} to plan in long chains. Spawn area: ${area}. Coordinates: ${coords}. Use the calculator with target pre-selected or browse combo links below.`,
    aboutHeading: `About ${pal.name}`,
    aboutHtml: `
      <p><strong>Area:</strong> ${escapeHtml(area)} · <strong>Coordinates:</strong> ${escapeHtml(coords)}</p>
      <p><a href="/palworld-breeding-calculator?target=${encodeURIComponent(pal.name)}">Open calculator with ${escapeHtml(pal.name)} as target</a> · <a href="/combos/${slug}">Combo hub</a> · <a href="/how-to-breed/${slug}">How to breed guide</a></p>
      <table class="database-table"><thead><tr><th>Parent A</th><th>Parent B</th><th>Method</th></tr></thead><tbody>${rows}</tbody></table>
      ${more}`,
    prefillTarget: pal.name,
  };
}

export function buildCombosHubContent(pal, combos) {
  const slug = palSlug(pal.name);
  const links = combos
    .slice(0, 12)
    .map(
      (pair) =>
        `<a href="/combo/${palSlug(pair.a)}/${palSlug(pair.b)}">${escapeHtml(pair.a)} + ${escapeHtml(pair.b)}</a>`
    )
    .join(" · ");

  return {
    title: `${pal.name} Breeding Combinations – All Parent Pairs | Palworld`,
    h1: `${pal.name} Breeding Combinations`,
    badge: `${pal.name} Combos`,
    h1Intro: `Every parent pair that can produce ${pal.name} in Palworld, with special combination and power-average methods listed for quick comparison before you commit eggs.`,
    pageDesc: `Complete list of Palworld breeding combinations to get ${pal.name}. ${combos.length} parent pairs with special combos and power-average routes linked to our calculator.`,
    aboutHeading: `${pal.name} combinations`,
    aboutHtml: `
      <p>Found <strong>${combos.length}</strong> routes to hatch <strong>${escapeHtml(pal.name)}</strong>.</p>
      <p>${links}</p>
      <p><a href="/palworld-breeding-calculator?target=${encodeURIComponent(pal.name)}">Reverse calculator</a> · <a href="/pal/${slug}">Pal profile</a></p>`,
    prefillTarget: pal.name,
  };
}

export function buildHowToBreedContent(pal, combos) {
  const slug = palSlug(pal.name);
  return {
    title: `How to Breed ${pal.name} in Palworld - Parent Paths`,
    h1: `How to Breed ${pal.name}`,
    badge: `Breed ${pal.name}`,
    h1Intro: `Step-by-step ${pal.name} breeding guide: unlock structures, use reverse lookup for ${combos.length} valid parent pairs, farm easier parents first, then run the egg loop.`,
    pageDesc: `Learn how to breed ${pal.name} in Palworld with parent path strategy, breeding power logic, and links to every valid combination in this database.`,
    aboutHeading: `How to breed ${pal.name}`,
    aboutHtml: `
      <ol>
        <li>Unlock <strong>Breeding Farm</strong> and <strong>Egg Incubator</strong>; keep cake in the feed box.</li>
        <li>Open the <a href="/palworld-breeding-calculator?target=${encodeURIComponent(pal.name)}">calculator</a> with ${escapeHtml(pal.name)} as target — <strong>${combos.length}</strong> pairs available.</li>
        <li>Farm easier parents first (higher breeding power = usually easier captures).</li>
        <li>Place parents, incubate, repeat until you hatch ${escapeHtml(pal.name)}.</li>
      </ol>
      <p><a href="/pal/${slug}">Pal page</a> · <a href="/combos/${slug}">All combos</a></p>`,
    prefillTarget: pal.name,
  };
}

export function buildComboPairContent(parentA, parentB, calc) {
  return {
    title: `${parentA.name} + ${parentB.name} Breeding Result | Palworld`,
    h1: `${parentA.name} + ${parentB.name}`,
    badge: "Parent Pair",
    h1Intro: `Breeding ${parentA.name} and ${parentB.name} predicts child ${calc.child.name} via ${calc.method}. Breeding powers: ${parentA.power} and ${parentB.power}.`,
    pageDesc: `Palworld breeding outcome for ${parentA.name} + ${parentB.name}: child ${calc.child.name} (${calc.method}). Test alternatives in the calculator below.`,
    aboutHeading: "Pair result",
    aboutHtml: `
      <p><strong>Child:</strong> ${escapeHtml(calc.child.name)} (${calc.child.power} power)</p>
      <p><strong>Method:</strong> ${escapeHtml(calc.method)}</p>
      <p><a href="/pal/${palSlug(calc.child.name)}">${escapeHtml(calc.child.name)} profile</a></p>`,
    prefillParents: [parentA.name, parentB.name],
  };
}

export function resolvePathContent(pathname, appData, specialCombos) {
  const path = pathname.toLowerCase().replace(/\/$/, "") || "/";
  const pals = appData.pals;

  const palMatch = path.match(/^\/pal\/([^/]+)$/);
  if (palMatch) {
    const pal = findPalBySlug(pals, palMatch[1]);
    if (!pal) return null;
    const combos = combinationsForTarget(pals, specialCombos, pal.name);
    return { type: "pal", ...buildPalRouteContent(pal, appData.pal_locations, combos) };
  }

  const combosMatch = path.match(/^\/combos\/([^/]+)$/);
  if (combosMatch) {
    const pal = findPalBySlug(pals, combosMatch[1]);
    if (!pal) return null;
    const combos = combinationsForTarget(pals, specialCombos, pal.name);
    return { type: "combos-hub", ...buildCombosHubContent(pal, combos) };
  }

  const howMatch = path.match(/^\/how-to-breed\/([^/]+)$/);
  if (howMatch) {
    const pal = findPalBySlug(pals, howMatch[1]);
    if (!pal) return null;
    const combos = combinationsForTarget(pals, specialCombos, pal.name);
    return { type: "how-to", ...buildHowToBreedContent(pal, combos) };
  }

  const pairMatch = path.match(/^\/combo\/([^/]+)\/([^/]+)$/);
  if (pairMatch) {
    const a = findPalBySlug(pals, pairMatch[1]) || findPal(pals, slugToTitle(pairMatch[1]));
    const b = findPalBySlug(pals, pairMatch[2]) || findPal(pals, slugToTitle(pairMatch[2]));
    if (!a || !b) return null;
    const calc = calculateChild(pals, specialCombos, a.name, b.name);
    if (!calc) return null;
    return { type: "combo-pair", ...buildComboPairContent(a, b, calc) };
  }

  return null;
}
