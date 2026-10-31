const parentASelect = document.getElementById("parentA");
const parentAGenderSelect = document.getElementById("parentAGender");
const parentBGenderSelect = document.getElementById("parentBGender");
const parentBSelect = document.getElementById("parentB");
const targetChildSelect = document.getElementById("targetChild");
const calculateBtn = document.getElementById("calculateBtn");
const swapBtn = document.getElementById("swapBtn");
const pickForSelect = document.getElementById("pickFor");
const findCombosBtn = document.getElementById("findCombosBtn");
const resultDiv = document.getElementById("result");
const combosDiv = document.getElementById("combos");
const statsBar = document.getElementById("statsBar");
const themeToggleBtn = document.getElementById("themeToggleBtn");
const palSearch = document.getElementById("palSearch");
const palGrid = document.getElementById("palGrid");
const kidBg = document.getElementById("kidBg");
const confettiLayer = document.getElementById("confettiLayer");
const databasePanelTitle = document.getElementById("databasePanelTitle");
const databasePanelBody = document.getElementById("databasePanelBody");
const navButtons = document.querySelectorAll("[data-view]");
const routeBadge = document.getElementById("routeBadge");
const routeSubtitle = document.getElementById("routeSubtitle");
const databaseCard = document.getElementById("databaseCard");
const arenaCard = document.getElementById("arenaCard");
const reverseCard = document.getElementById("reverseCard");
const chainCard = document.getElementById("chainCard");
const chainOwnedSelect = document.getElementById("chainOwned");
const chainAdditionalSearch = document.getElementById("chainAdditionalSearch");
const chainAdditionalChips = document.getElementById("chainAdditionalChips");
const chainAdditionalOptions = document.getElementById("chainAdditionalOptions");
const chainGoalSelect = document.getElementById("chainGoal");
const findChainBtn = document.getElementById("findChainBtn");
const chainResultDiv = document.getElementById("chainResult");
const palBoxCard = document.getElementById("palBoxCard");
const heroStrip = document.getElementById("heroStrip");
const palGridStatus = document.getElementById("palGridStatus");
const showAllPalsBtn = document.getElementById("showAllPalsBtn");
// Featured cards are rendered by the server; the full list is built on demand.
const featuredPalCards = Array.from(palGrid?.children || []);

const pathToView = {
  "/": "breeding",
  "/breeding-calculator": "breeding",
  "/palworld-breeding-calculator": "breeding",
  "/pals": "pals",
  "/map": "map",
  "/maps": "map",
  "/items": "items",
  "/technology": "technology",
  "/capture-rate": "capture",
  "/palworld-breeding-combinations": "breeding",
  "/palworld-capture-rate-calculator": "capture",
  "/palworld-chain-breeding": "chain"
};

const viewMeta = {
  breeding: {
    badge: "Breeding View",
    subtitle: "Use parent pair logic and reverse lookup to discover strong child outcomes.",
    focusCardId: "arenaCard"
  },
  pals: {
    badge: "Pals View",
    subtitle: "Browse pal power references and compare top options for breeding routes.",
    focusCardId: "databaseCard"
  },
  map: {
    badge: "Map View",
    subtitle: "See location-focused data to find pals faster and speed up breeding setup.",
    focusCardId: "databaseCard"
  },
  items: {
    badge: "Items View",
    subtitle: "Check item sources and farming notes to support your breeding progression.",
    focusCardId: "databaseCard"
  },
  technology: {
    badge: "Technology View",
    subtitle: "Track key unlock milestones needed for incubation, breeding, and production.",
    focusCardId: "databaseCard"
  },
  capture: {
    badge: "Capture View",
    subtitle: "Compare a breeding-power difficulty heuristic. This is not an in-game capture probability.",
    focusCardId: "reverseCard"
  },
  chain: {
    badge: "Chain Breeder",
    subtitle: "Check routes using your selected owned roster. Missing partner species must be acquired first.",
    focusCardId: "chainCard"
  }
};

const routeIconSeeds = {
  breeding: ["anubis", "jetragon", "frostallion", "blazamut", "suzaku", "jormuntide", "necromus"],
  pals: ["lamball", "cattiva", "chikipi", "lifmunk", "tanzee", "foxparks", "rooby"],
  map: ["eikthyrdeer", "pengullet", "daedream", "pyrin", "anubis", "jetragon", "suzaku"],
  items: ["lamball", "foxparks", "rooby", "frostallion", "jolthog", "pengullet", "anubis"],
  technology: ["lifmunk", "tanzee", "eikthyrdeer", "jormuntide", "paladius", "necromus", "jetragon"],
  capture: ["jetragon", "frostallion", "necromus", "paladius", "jormuntide", "anubis", "suzaku"],
  chain: ["lamball", "cattiva", "anubis", "jetragon", "frostallion", "necromus", "paladius"]
};

let appData = {
  pals: [],
  pal_locations: {},
  items: [],
  technologies: [],
  special_combos_count: 0,
  special_combos: {}
};

const PAL_GRID_CHUNK = 48;
const COMBO_LIST_WITH_IMAGES = 24;
const COMBO_LIST_MAX = 80;
const CHAIN_SUGGESTIONS_MAX = 8;

let palByNameLower = new Map();
let palBySlugMap = new Map();

const combinationsCache = new Map();
let locationsLoaded = false;
let locationsLoading = null;
let lastHeroView = "";
let combinationsRequestId = 0;
let chainRequestId = 0;
const MAP_ROWS_INITIAL = 9999;

const serverRouteIntro =
  document.body.dataset.routeIntro?.trim() ||
  document.getElementById("routeSubtitle")?.textContent?.trim() ||
  "";

// Same wording as `pal_icon_alt` in src/main.rs.
function getPalAltText(palName) {
  return `${palName} Palworld Pal icon`;
}

function debounce(fn, ms) {
  let timer = null;
  return (...args) => {
    clearTimeout(timer);
    timer = setTimeout(() => fn(...args), ms);
  };
}

// The first toLocaleString() call loads ICU locale data: ~200ms on a throttled phone CPU.
function formatCount(value) {
  return String(value).replace(/\B(?=(\d{3})+(?!\d))/g, ",");
}

function yieldToMain() {
  if (globalThis.scheduler?.yield) return globalThis.scheduler.yield();
  return new Promise((resolve) => setTimeout(resolve, 0));
}

function whenIdle(fn, timeout = 200) {
  if (typeof requestIdleCallback === "function") {
    requestIdleCallback(fn, { timeout });
  } else {
    setTimeout(fn, 16);
  }
}

function palSlug(palName) {
  return palName
    .toLowerCase()
    .trim()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
}

// Root-absolute paths. Relative "assets/..." breaks on /pal/, /combo/, and /how-to-breed/.
function getPalImageUrl(palName) {
  const slug = palSlug(resolvePalDisplayName(palName));
  return appData.available_icons?.includes(slug) ? `/assets/pals/thumbs/${slug}.webp` : PAL_PLACEHOLDER;
}

function getPalFullImageUrl(palName) {
  const slug = palSlug(resolvePalDisplayName(palName));
  return appData.available_icons?.includes(slug) ? `/assets/pals/${slug}.webp` : PAL_PLACEHOLDER;
}

function buildPalIndexes() {
  palByNameLower = new Map();
  palBySlugMap = new Map();
  for (const pal of appData.pals) {
    palByNameLower.set(pal.name.toLowerCase(), pal);
    palBySlugMap.set(palSlug(pal.name), pal);
  }
  appData.pals.forEach((pal, index) => pal.tableIndex = index);
}

function resolvePalDisplayName(palName) {
  if (!palName) return palName;
  const exact = palByNameLower.get(palName) || palByNameLower.get(palName.toLowerCase());
  if (exact) return exact.name;
  const bySlug = palBySlugMap.get(palSlug(palName));
  return bySlug?.name ?? palName;
}

function comboKey(a, b) {
  return a <= b ? `${a}|${b}` : `${b}|${a}`;
}

function calculateChildLocal(parentA, parentB, genderA = null, genderB = null) {
  const first = palByNameLower.get(parentA?.toLowerCase());
  const second = palByNameLower.get(parentB?.toLowerCase());
  const table = appData.breeding_table;
  if (!first || !second || !table) return null;
  let a = first.tableIndex, b = second.tableIndex;
  if (a > b) [a, b, genderA, genderB] = [b, a, genderB, genderA];
  const offset = a * appData.pals.length - a * (a - 1) / 2 + b - a;
  let childIndex = table.children[offset];
  let method = first.name === second.name ? "Same species" : "PalCalc source table";
  if (childIndex < 0) {
    const row = table.gender_outcomes.find((row) => row.a === a && row.b === b &&
      row.a_gender === genderA && row.b_gender === genderB);
    if (!row) return { error: "This pair has gender-specific outcomes. Choose one male and one female parent." };
    childIndex = row.child;
    method = `Gender-specific: ${appData.pals[a].name} ${genderA} + ${appData.pals[b].name} ${genderB}`;
  } else if (appData.special_combos?.[comboKey(first.name, second.name)]) {
    method = "Special combination";
  }
  const child = appData.pals[childIndex];
  return child ? { child, method, distance: null } : null;
}

// ---- Chain planner (runs in the browser; mirrors src/chain.rs exactly) ----
const CHAIN_MAX_STEPS = 15;
const CHAIN_MAX_STATES = 8000;
let reverseIndexCache = null;

function tableOutcomes(i, j) {
  const table = appData.breeding_table;
  const n = appData.pals.length;
  const offset = i * n - i * (i - 1) / 2 + j - i;
  const child = table.children[offset];
  if (child >= 0) return [{ a: i, b: j, child, a_gender: "wildcard", b_gender: "wildcard" }];
  return table.gender_outcomes.filter((row) => row.a === i && row.b === j);
}

function outcomeMethod(row) {
  const pals = appData.pals;
  if (row.a_gender !== "wildcard") {
    return `Gender-specific: ${pals[row.a].name} ${row.a_gender} + ${pals[row.b].name} ${row.b_gender}`;
  }
  if (row.a === row.b) return "Same species";
  if (appData.special_combos?.[comboKey(pals[row.a].name, pals[row.b].name)]) return "Special combination";
  return "PalCalc source table";
}

// child name -> [{a, b, method}] in the same order as the server's reverse index.
function reverseIndex() {
  if (reverseIndexCache) return reverseIndexCache;
  const pals = appData.pals;
  const index = new Map();
  for (let i = 0; i < pals.length; i++) {
    for (let j = i; j < pals.length; j++) {
      for (const row of tableOutcomes(i, j)) {
        const name = pals[row.child].name;
        if (!index.has(name)) index.set(name, []);
        index.get(name).push({ a: pals[i].name, b: pals[j].name, method: outcomeMethod(row) });
      }
    }
  }
  reverseIndexCache = index;
  return index;
}

function byChildThenParents(x, y) {
  const kx = [x.child, x.parent_a, x.parent_b];
  const ky = [y.child, y.parent_a, y.parent_b];
  for (let k = 0; k < 3; k++) {
    if (kx[k] < ky[k]) return -1;
    if (kx[k] > ky[k]) return 1;
  }
  return 0;
}

function searchChainLocal(owned, goal, alsoOwned) {
  if (owned === goal) return [];
  const startHave = new Set([owned, ...alsoOwned]);
  if (startHave.has(goal)) return [];
  const indexOf = new Map(appData.pals.map((pal) => [pal.name, pal.tableIndex]));
  const keyOf = (have) => [...have].sort().join("|");
  const seen = new Set([keyOf(startHave)]);
  const queue = [{ steps: [], have: startHave }];
  for (let head = 0; head < queue.length; head++) {
    const node = queue[head];
    if (node.steps.length >= CHAIN_MAX_STEPS) continue;
    // Every recipe whose parents are both owned, in the server's (child, a, b) order.
    const owned_ = [...node.have].map((name) => indexOf.get(name)).sort((x, y) => x - y);
    const recipes = [];
    for (let x = 0; x < owned_.length; x++) {
      for (let y = x; y < owned_.length; y++) {
        for (const row of tableOutcomes(owned_[x], owned_[y])) {
          recipes.push({
            parent_a: appData.pals[owned_[x]].name,
            parent_b: appData.pals[owned_[y]].name,
            child: appData.pals[row.child].name,
            method: outcomeMethod(row)
          });
        }
      }
    }
    recipes.sort(byChildThenParents);
    for (const recipe of recipes) {
      if (node.have.has(recipe.child)) continue;
      const steps = [...node.steps, recipe];
      if (recipe.child === goal) return steps;
      const have = new Set(node.have);
      have.add(recipe.child);
      const key = keyOf(have);
      if (seen.has(key)) continue;
      if (seen.size >= CHAIN_MAX_STATES) return null;
      seen.add(key);
      queue.push({ steps, have });
    }
  }
  return null;
}

function directParentComboForOwned(owned, goal) {
  const lower = owned.toLowerCase();
  const pairs = (reverseIndex().get(goal) || []).filter(
    (pair) => pair.a.toLowerCase() === lower || pair.b.toLowerCase() === lower
  );
  const partnerOf = (pair) => (pair.a.toLowerCase() === lower ? pair.b : pair.a);
  const power = (name) => palByNameLower.get(name.toLowerCase())?.power ?? 0;
  pairs.sort((x, y) => {
    const px = partnerOf(x), py = partnerOf(y);
    if (power(px) !== power(py)) return power(py) - power(px);
    return px < py ? -1 : px > py ? 1 : 0;
  });
  return pairs[0] || null;
}

function chainFailureHint(owned, goal) {
  const ownedPal = palByNameLower.get(owned.toLowerCase());
  const goalPal = palByNameLower.get(goal.toLowerCase());
  if (!ownedPal || !goalPal) return null;
  const pair = directParentComboForOwned(ownedPal.name, goalPal.name);
  if (pair) {
    const partner = pair.a.toLowerCase() === owned.toLowerCase() ? pair.b : pair.a;
    const partnerPal = palByNameLower.get(partner.toLowerCase());
    if (!partnerPal) return null;
    return `${owned} can breed ${goal} with ${partner}, but the planner could not reach ${partner} (breeding power ${partnerPal.power}) starting from only ${owned} (power ${ownedPal.power}). Capture or breed ${partner} first, then use that pair in the calculator.`;
  }
  if (goalPal.power - ownedPal.power > 200) {
    return `${owned} (breeding power ${ownedPal.power}) is far below ${goal} (power ${goalPal.power}). Pick an owned Pal closer to your goal, or use reverse lookup for parent pairs.`;
  }
  return null;
}

/** Same JSON shape as the old /api/chain endpoint, computed in the browser. */
function planChainLocal(ownedInput, goalInput, alsoOwnedInput = []) {
  const owned = ownedInput.trim();
  const goal = goalInput.trim();
  const ownedPal = palByNameLower.get(owned.toLowerCase());
  const goalPal = palByNameLower.get(goal.toLowerCase());
  if (!owned || !goal || !ownedPal || !goalPal) throw new Error("Unknown owned or goal Pal");
  const roster = [];
  for (const name of alsoOwnedInput.map((n) => n.trim()).filter(Boolean)) {
    const pal = palByNameLower.get(name.toLowerCase());
    if (!pal) throw new Error(`Unknown owned Pal: ${name}`);
    if (!roster.includes(pal.name)) roster.push(pal.name);
  }
  const steps = searchChainLocal(ownedPal.name, goalPal.name, roster);
  if (steps) {
    return { owned, also_owned: roster, goal, found: true, steps, step_count: steps.length };
  }
  const hint = roster.length ? null : chainFailureHint(owned, goal);
  const pair = directParentComboForOwned(owned, goalPal.name);
  const partial = pair
    ? {
        parent_a: pair.a,
        parent_b: pair.b,
        child: goal,
        method: pair.method,
        missing_partner: pair.a.toLowerCase() === owned.toLowerCase() ? pair.b : pair.a
      }
    : null;
  return {
    owned, also_owned: roster, goal, found: false, steps: [], step_count: 0, hint, partial,
    message: hint || `No executable breeding chain found from ${owned} to ${goal} within planner limits (15 steps and 8,000 inventories). Only selected species are treated as owned. Add owned partners or acquire a partner from reverse lookup first.`
  };
}

function getPalCdnUrl(palName) {
  const name = resolvePalDisplayName(palName);
  return `https://ggservers.com/images/palworld/${encodeURIComponent(name)}.webp`;
}

const PAL_PLACEHOLDER = "/assets/pals/placeholder.svg";

globalThis.__palImgFallback = (img) => {
  if (!img?.dataset) return;
  if (img.dataset.fallback !== "full") {
    img.dataset.fallback = "full";
    img.src = img.dataset.full;
    return;
  }
  if (img.dataset.fallback !== "cdn" && img.dataset.cdn) {
    img.dataset.fallback = "cdn";
    img.src = img.dataset.cdn;
    return;
  }
  if (img.dataset.fallback !== "ph" && img.dataset.placeholder) {
    img.dataset.fallback = "ph";
    img.onerror = null;
    img.src = img.dataset.placeholder;
  }
};

function buildPalImage(palName, className = "pal-image") {
  const localThumb = getPalImageUrl(palName);
  const localFull = getPalFullImageUrl(palName);
  const cdn = "";
  const size = className.includes("grid") ? 100 : className.includes("hero") ? 42 : 36;
  
  return `<img class="${escapeHtml(className)}" src="${localThumb}" alt="${escapeHtml(getPalAltText(palName))}" width="${size}" height="${size}" loading="lazy" decoding="async"
    data-full="${localFull}" data-cdn="${cdn}" data-placeholder="${PAL_PLACEHOLDER}"
    onerror="window.__palImgFallback&&window.__palImgFallback(this)" />`;
}

const HERO_ICON_EAGER = 6;
const HERO_STRIP_MAX_PARALLEL = 8;
const heroStripLoadQueue = [];
let heroStripLoadsActive = 0;
let heroStripObserver = null;

function drainHeroStripQueue() {
  while (heroStripLoadsActive < HERO_STRIP_MAX_PARALLEL && heroStripLoadQueue.length) {
    const img = heroStripLoadQueue.shift();
    const url = img.dataset.src;
    if (!url || img.dataset.loaded === "1") continue;

    heroStripLoadsActive += 1;
    const probe = new Image();
    probe.decoding = "async";
    probe.onload = () => {
      img.src = url;
      img.classList.add("is-loaded");
      img.dataset.loaded = "1";
      heroStripLoadsActive -= 1;
      drainHeroStripQueue();
    };
    probe.onerror = () => {
      img.src = PAL_PLACEHOLDER;
      img.classList.add("is-loaded", "is-placeholder");
      img.dataset.loaded = "1";
      heroStripLoadsActive -= 1;
      drainHeroStripQueue();
    };
    probe.src = url;
  }
}

function queueHeroStripImage(img) {
  if (!img || img.dataset.loaded === "1" || img.dataset.queued === "1") return;
  img.dataset.queued = "1";
  heroStripLoadQueue.push(img);
  drainHeroStripQueue();
}

function buildHeroIcon(palName) {
  const local = getPalImageUrl(palName);
  const alt = getPalAltText(palName);
  return `<img class="hero-icon" alt="${escapeHtml(alt)}" width="42" height="42" decoding="async"
    src="${PAL_PLACEHOLDER}" data-src="${local}" />`;
}

function teardownHeroStripLoader() {
  if (heroStripObserver) {
    heroStripObserver.disconnect();
    heroStripObserver = null;
  }
  heroStripLoadQueue.length = 0;
  heroStripLoadsActive = 0;
}

function prefetchHeroIconsNearScroll() {
  if (!heroStrip) return;
  const stripRect = heroStrip.getBoundingClientRect();
  const pad = 320;
  for (const img of heroStrip.querySelectorAll(".hero-icon[data-src]")) {
    if (img.dataset.loaded === "1") continue;
    const r = img.getBoundingClientRect();
    if (r.right >= stripRect.left - pad && r.left <= stripRect.right + pad) {
      queueHeroStripImage(img);
      heroStripObserver?.unobserve(img);
    }
  }
}

let heroStripScrollTimer = null;
function onHeroStripScroll() {
  clearTimeout(heroStripScrollTimer);
  heroStripScrollTimer = setTimeout(prefetchHeroIconsNearScroll, 50);
}

function setupHeroStripLoader() {
  if (!heroStrip) return;
  teardownHeroStripLoader();
  heroStripObserver = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        if (entry.isIntersecting) {
          queueHeroStripImage(entry.target);
          heroStripObserver.unobserve(entry.target);
        }
      }
    },
    { root: heroStrip, rootMargin: "400px 0px", threshold: 0.01 }
  );

  const imgs = heroStrip.querySelectorAll(".hero-icon[data-src]");
  imgs.forEach((img, index) => {
    if (index < HERO_ICON_EAGER) {
      queueHeroStripImage(img);
    } else {
      heroStripObserver.observe(img);
    }
  });

  if (!heroStrip.dataset.scrollBound) {
    heroStrip.dataset.scrollBound = "1";
    heroStrip.addEventListener("scroll", onHeroStripScroll, { passive: true });
  }
}

let calculateRequestId = 0;
let parentCalcTimer = null;

function escapeHtml(text) {
  return String(text)
    .replaceAll("&", "&amp;")
    .replaceAll('"', "&quot;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;");
}

async function apiFetch(url, options) {
  const response = await fetch(url, options);
  if (!response.ok) {
    const message = await response.text();
    throw new Error(message || `API failed: ${response.status}`);
  }
  return response.json();
}

function getParentNames() {
  const parentA = parentASelect.value?.trim();
  const parentB = parentBSelect.value?.trim();
  return { parentA, parentB };
}

function showResultError(message) {
  resultDiv.innerHTML = `<div class="result-error">${escapeHtml(message)}</div>`;
}

function scheduleRenderResult(options = {}) {
  clearTimeout(parentCalcTimer);
  parentCalcTimer = setTimeout(() => {
    renderResult(options);
  }, 50);
}

// Built once, then updated in place so a recalculation only swaps text and image sources.
let resultView = null;

function setArenaCard(card, palName) {
  const img = card.querySelector("img");
  delete img.dataset.fallback;
  img.src = getPalImageUrl(palName);
  img.dataset.full = getPalFullImageUrl(palName);
  img.alt = getPalAltText(palName);
  card.querySelector(".arena-name").textContent = palName;
}

function paintResult(calc, parentA, parentB, celebrate) {
  if (!resultView) {
    const root = document.createElement("div");
    root.innerHTML = `
      <div class="arena">
        ${buildArenaCard(parentA, "parent")}
        <span class="flow-symbol">+</span>
        ${buildArenaCard(parentB, "parent")}
        <span class="flow-symbol">=</span>
        ${buildArenaCard(calc.child.name, "child")}
      </div>
      <div class="muted result-method"></div>
      <div class="muted result-distance" hidden></div>`;
    resultView = { root, cards: root.querySelectorAll(".arena-card"), method: root.querySelector(".result-method"), distance: root.querySelector(".result-distance") };
  }
  const [cardA, cardB, cardChild] = resultView.cards;
  setArenaCard(cardA, parentA);
  setArenaCard(cardB, parentB);
  setArenaCard(cardChild, calc.child.name);
  resultView.method.textContent = calc.method;
  const hasDistance = typeof calc.distance === "number";
  resultView.distance.hidden = !hasDistance;
  resultView.distance.textContent = hasDistance ? `Closest power distance: ${calc.distance}` : "";
  if (resultView.root.parentNode !== resultDiv) resultDiv.replaceChildren(resultView.root);
  if (celebrate) {
    celebrateConfetti();
  }
}

function buildPalChip(palName) {
  return `
    <div class="pal-chip">
      ${buildPalImage(palName, "pal-image")}
      <span>${escapeHtml(palName)}</span>
    </div>
  `;
}

function buildArenaCard(palName, role) {
  const roleClass = role === "child" ? " child" : "";
  return `
    <div class="arena-card${roleClass}">
      ${buildPalImage(palName, "arena-pal-img")}
      <div class="arena-name">${escapeHtml(palName)}</div>
    </div>
  `;
}

function populateSelect(selectEl) {
  const fragment = document.createDocumentFragment();
  for (const pal of appData.pals) {
    const option = document.createElement("option");
    option.value = pal.name;
    option.textContent = pal.name;
    fragment.appendChild(option);
  }
  selectEl.replaceChildren(fragment);
}

function setTheme(themeName) {
  document.body.dataset.theme = themeName;
  localStorage.setItem("palworldTheme", themeName);
  themeToggleBtn.textContent = themeName === "light" ? "Dark Mode" : "Light Mode";
}

function renderStatsBar() {
  const totalPairs = (appData.pals.length * (appData.pals.length + 1)) / 2;
  statsBar.innerHTML = `
    <span class="stat-pill">Pals: <strong>${appData.pals.length}</strong></span>
    <span class="stat-pill">Fixed cross-species pairs: <strong>${appData.special_combos_count}</strong></span>
    <span class="stat-pill">Parent pairs: <strong>${totalPairs}</strong></span>
  `;
}

function celebrateConfetti() {
  if (globalThis.matchMedia?.("(prefers-reduced-motion: reduce)").matches) return;
  const colors = ["#ff4fd8", "#53e8ff", "#fff15f", "#7dff7a", "#ff8c42"];
  const pieces = 36;
  const rect = resultDiv.getBoundingClientRect();
  const startX = rect.left + rect.width / 2;
  const startY = rect.top + 28;

  for (let i = 0; i < pieces; i += 1) {
    const piece = document.createElement("span");
    piece.className = "confetti-piece";
    piece.style.left = `${startX - 160 + Math.random() * 320}px`;
    piece.style.top = `${startY}px`;
    piece.style.background = colors[i % colors.length];
    piece.style.width = `${7 + Math.random() * 7}px`;
    piece.style.height = `${10 + Math.random() * 9}px`;
    piece.style.animationDuration = `${700 + Math.random() * 700}ms`;
    piece.style.animationDelay = `${Math.random() * 120}ms`;
    confettiLayer.appendChild(piece);
    setTimeout(() => piece.remove(), 1700);
  }
}

async function ensureLocations() {
  if (locationsLoaded) return;
  if (locationsLoading) {
    await locationsLoading;
    return;
  }
  locationsLoading = apiFetch("/api/locations").then((data) => {
    appData.pal_locations = data;
    locationsLoaded = true;
  });
  await locationsLoading;
  locationsLoading = null;
}

function restoreRouteIntro() {
  if (serverRouteIntro && routeSubtitle) {
    routeSubtitle.textContent = serverRouteIntro;
  }
}

async function renderDatabasePanel(view) {
  const meta = viewMeta[view] || viewMeta.breeding;
  routeBadge.textContent = meta.badge;
  restoreRouteIntro();
  document.body.dataset.routeView = view;
  if (view !== lastHeroView) {
    renderHeroStrip(view, true);
    lastHeroView = view;
  }
  navButtons.forEach((button) => {
    button.classList.toggle("is-active", button.dataset.view === view);
  });

  if (view === "pals") {
    const highest = [...appData.pals].sort((a, b) => b.power - a.power).slice(0, 3);
    databasePanelTitle.textContent = "Pals";
    databasePanelBody.innerHTML = `Total pals in current dataset: <strong>${appData.pals.length}</strong>.<br />Highest breeding-power pals: ${highest
      .map((pal) => `<strong>${pal.name}</strong> (${pal.power})`)
      .join(", ")}.`;
    return;
  }

  if (view === "map") {
    databasePanelTitle.textContent = "Map";
    databasePanelBody.innerHTML = `<div class="muted">Loading map data…</div>`;
    await ensureLocations();
    const entries = appData.pals
      .map((pal) => {
        const location = appData.pal_locations[pal.name];
        return {
          name: pal.name,
          area: location?.area || "Palpagos Island (overworld)",
          coords: location?.coords || "—"
        };
      })
      .sort((a, b) => a.name.localeCompare(b.name));
    const visible = entries.slice(0, MAP_ROWS_INITIAL);
    const rows = visible
      .map(
        (entry) =>
          `<tr><td>${escapeHtml(entry.name)}</td><td>${escapeHtml(entry.area)}</td><td>${escapeHtml(entry.coords)}</td></tr>`
      )
      .join("");
    const more =
      entries.length > visible.length
        ? `<p class="muted">Showing ${visible.length} of ${entries.length} Pals. Use Pal Box search for one name.</p>`
        : `<p class="muted">All <strong>${entries.length}</strong> Pals listed.</p>`;
    databasePanelBody.innerHTML = `
      <div class="database-caption">Spawn regions and map coordinates.</div>
      <table class="database-table">
        <thead><tr><th>Pal</th><th>Area</th><th>Coordinates</th></tr></thead>
        <tbody>${rows}</tbody>
      </table>
      ${more}
    `;
    return;
  }

  if (view === "items") {
    databasePanelTitle.textContent = "Items";
    const rows = appData.items
      .map((item) => `<tr><td>${item.item}</td><td>${item.source}</td><td>${item.notes}</td></tr>`)
      .join("");
    databasePanelBody.innerHTML = `
      <div class="database-caption">Common crafting and breeding-related materials.</div>
      <table class="database-table">
        <thead><tr><th>Item</th><th>Source</th><th>Notes</th></tr></thead>
        <tbody>${rows}</tbody>
      </table>
    `;
    return;
  }

  if (view === "technology") {
    databasePanelTitle.textContent = "Technology";
    const rows = [...appData.technologies]
      .sort((a, b) => a.level - b.level)
      .map((tech) => `<tr><td>${tech.level}</td><td>${tech.name}</td><td>${tech.cost}</td></tr>`)
      .join("");
    databasePanelBody.innerHTML = `
      <div class="database-caption">Breeding, egg and sphere unlocks for Palworld 1.0.5 (level cap 80).</div>
      <table class="database-table">
        <thead><tr><th>Level</th><th>Technology</th><th>Cost</th></tr></thead>
        <tbody>${rows}</tbody>
      </table>
    `;
    return;
  }

  if (view === "chain") {
    databasePanelTitle.textContent = "Chain Breeding";
    databasePanelBody.innerHTML = `Only the species selected in your owned roster are treated as available. Every step requires both parent species to be available. Use reverse lookup to find partners to acquire first; the bounded planner does not guarantee a route.`;
    return;
  }

  if (view === "capture") {
    const targetName = targetChildSelect.value || appData.pals[0].name;
    const capture = await apiFetch(`/api/capture/${encodeURIComponent(targetName)}`);
    const list = capture.easier_targets.map((pal) => `<li>${pal.name} (${pal.power})</li>`).join("");
    databasePanelTitle.textContent = "Capture Difficulty Heuristic";
    databasePanelBody.innerHTML = `
      <div class="database-caption">
        Breeding-power difficulty score for <strong>${escapeHtml(capture.target)}</strong>: <strong>${capture.difficulty_score}/100</strong> (higher means harder in this heuristic).
      </div>
      <p class="muted">This score uses breeding power only. It is not a capture probability and does not account for spheres, health, level, or settings.</p>
      <div class="database-caption">Lower-score species in this roster (not verified capture rates):</div>
      <ul class="database-list">${list}</ul>
    `;
    return;
  }

  databasePanelTitle.textContent = "Breeding Calculator";
  databasePanelBody.innerHTML = `Breeding calculator is active. Current special combinations: <strong>${appData.special_combos_count}</strong>.`;
}

const HERO_STRIP_MAX_ICONS = 24;

function heroStripPalNames(view) {
  const seeds = routeIconSeeds[view] || routeIconSeeds.breeding;
  const seen = new Set();
  const names = [];
  for (const seed of seeds) {
    const name = resolvePalDisplayName(seed);
    const key = name.toLowerCase();
    if (!seen.has(key)) {
      seen.add(key);
      names.push(name);
    }
  }
  for (const pal of appData.pals) {
    if (names.length >= HERO_STRIP_MAX_ICONS) break;
    const key = pal.name.toLowerCase();
    if (!seen.has(key)) {
      seen.add(key);
      names.push(pal.name);
    }
  }
  return names;
}

function renderHeroStrip(view, force = false) {
  if (!heroStrip) return;
  if (!force && view === lastHeroView && heroStrip.childElementCount > 5) return;
  
  const icons = heroStripPalNames(view);
  teardownHeroStripLoader();
  heroStrip.replaceChildren();
  const frag = document.createDocumentFragment();
  for (const name of icons) {
    const wrap = document.createElement("div");
    wrap.innerHTML = buildHeroIcon(name);
    frag.appendChild(wrap.firstElementChild);
  }
  heroStrip.appendChild(frag);
  setupHeroStripLoader();
}

function normalizePath(pathname) {
  const lower = String(pathname || "/").toLowerCase();
  if (lower.length > 1 && lower.endsWith("/")) {
    return lower.slice(0, -1);
  }
  return lower;
}

function resolveViewFromPath(pathname) {
  const normalized = normalizePath(pathname);
  if (pathToView[normalized]) return pathToView[normalized];
  if (normalized.startsWith("/pal/")) return "pals";
  if (normalized.startsWith("/combo/")) return "breeding";
  if (normalized.startsWith("/guides/")) return "breeding";
  return "breeding";
}

function focusViewCard(view) {
  const meta = viewMeta[view] || viewMeta.breeding;
  const target = document.getElementById(meta.focusCardId);
  if (!target) return;
  target.scrollIntoView({ behavior: "smooth", block: "start" });
}

function renderResult(options = {}) {
  const { celebrate = false } = options;
  const { parentA, parentB } = getParentNames();

  if (!parentA || !parentB) {
    showResultError("Choose both Parent A and Parent B.");
    return;
  }

  const genderA = parentAGenderSelect?.value || null;
  const genderB = parentBGenderSelect?.value || null;
  if (genderA && genderA === genderB) {
    showResultError("Choose one male and one female parent.");
    return;
  }

  const requestId = ++calculateRequestId;
  const calc = calculateChildLocal(parentA, parentB, genderA, genderB);
  if (!calc) {
    showResultError("Could not find those Pals. Refresh the page or pick names from the list.");
    return;
  }
  if (calc.error) { showResultError(calc.error); return; }
  if (requestId !== calculateRequestId) return;
  paintResult(calc, parentA, parentB, celebrate);
}

async function renderChainBreeding() {
  const requestId = ++chainRequestId;
  const owned = chainOwnedSelect?.value?.trim();
  const goal = chainGoalSelect?.value?.trim();
  if (!chainResultDiv || !owned || !goal) return;
  
  if (owned.toLowerCase() === goal.toLowerCase()) {
    chainResultDiv.innerHTML = `<span class="muted">You already own <strong>${escapeHtml(goal)}</strong> — no breeding steps needed.</span>`;
    return;
  }

  chainResultDiv.innerHTML = `<span class="muted">Finding chain from ${escapeHtml(owned)} to ${escapeHtml(goal)}…</span>`;
  if (findChainBtn) findChainBtn.disabled = true;

  const alsoOwned = [...chainRoster];
  const params = new URLSearchParams({ owned, goal });
  if (alsoOwned.length) params.set("also_owned", alsoOwned.join(","));
  const shareUrl = `${globalThis.location.pathname}?${params}`;
  if (globalThis.location.search !== `?${params}`) {
    globalThis.history.replaceState({}, "", shareUrl);
  }

  try {
    // Computed in the browser so the site also works as static files (no server API).
    const data = planChainLocal(owned, goal, alsoOwned);
    if (requestId !== chainRequestId) return;
    const steps = data.steps || [];
    if (data.found === false || (!steps.length && data.found !== true)) {
      const msg =
        data.hint ||
        data.message ||
        `No executable route found within planner limits. Add other owned species or use reverse lookup to acquire a partner.`;
      let extra = "";
      if (data.partial?.missing_partner) {
        const p = data.partial;
        const goalSlug = palSlug(goal);
        extra = `
          <p class="chain-partial"><strong>Direct combo found:</strong>
            ${buildPalChip(p.parent_a)}
            <span class="flow-symbol">+</span>
            ${buildPalChip(p.parent_b)}
            <span class="flow-symbol">→</span>
            <strong>${escapeHtml(goal)}</strong>
            <span class="muted">(${escapeHtml(p.method || "")})</span>
          </p>
          <p class="muted">You still need <strong>${escapeHtml(p.missing_partner)}</strong> before this egg works.</p>
          <p><a href="/?target=${encodeURIComponent(goal)}">Reverse lookup for ${escapeHtml(goal)}</a>
            · <a href="/combos/${goalSlug}">All ${escapeHtml(goal)} combos</a></p>`;
      } else {
        extra = `<p><a href="/?target=${encodeURIComponent(goal)}">Reverse lookup for ${escapeHtml(goal)}</a></p>`;
      }
      chainResultDiv.innerHTML = `<div class="chain-not-found"><p>${escapeHtml(msg)}</p>${extra}</div>`;
      return;
    }
    if (!steps.length) {
      chainResultDiv.innerHTML = `<span class="muted">Your selected roster already includes <strong>${escapeHtml(goal)}</strong> — no breeding steps needed.</span>`;
      return;
    }

    const items = steps
      .map((step, index) => {
        const childSlug = palSlug(step.child);
        return `
        <li class="chain-step">
          <div class="chain-step-head">Step ${index + 1}: Breed ${escapeHtml(step.child)}</div>
          <div class="chain-step-pair">
            ${buildPalChip(step.parent_a)}
            <span class="flow-symbol">+</span>
            ${buildPalChip(step.parent_b)}
            <span class="flow-symbol">→</span>
            <a class="pal-chip pal-chip-link" href="/pal/${childSlug}">${buildPalImage(step.child, "pal-image")}<span>${escapeHtml(step.child)}</span></a>
          </div>
          <span class="muted">(${escapeHtml(step.method)})</span>
        </li>
      `;
      })
      .join("");

    chainResultDiv.innerHTML = `
      <p><strong>${steps.length}</strong> breeding step${steps.length === 1 ? "" : "s"} from <strong>${escapeHtml(owned)}</strong> to <strong>${escapeHtml(goal)}</strong>.</p>
      <ol class="chain-steps">${items}</ol>
      <p class="muted"><a href="/?target=${encodeURIComponent(goal)}">Reverse lookup for ${escapeHtml(goal)}</a></p>
    `;
  } catch (error) {
    if (requestId === chainRequestId) {
      const msg = String(error.message || "Chain lookup failed");
      const hint = msg.includes("timed out") || msg.includes("15 steps")
        ? " Try a closer goal Pal or use the reverse calculator for parent pairs."
        : "";
      chainResultDiv.innerHTML = `<span class="muted">${escapeHtml(msg)}${escapeHtml(hint)}</span>`;
    }
  } finally {
    if (requestId === chainRequestId && findChainBtn) {
      findChainBtn.disabled = false;
    }
  }
}

async function renderCombinations() {
  const requestId = ++combinationsRequestId;
  const targetName = targetChildSelect.value;
  if (!targetName) {
    combosDiv.innerHTML = `<span class="muted">Select a target Pal first.</span>`;
    return;
  }
  combosDiv.innerHTML = `<span class="muted">Finding combinations for ${escapeHtml(targetName)}…</span>`;
  findCombosBtn.disabled = true;
  try {
    let pairs = combinationsCache.get(targetName);
    if (!pairs) {
      pairs = await apiFetch(`/api/combinations/${encodeURIComponent(targetName)}`);
      combinationsCache.set(targetName, pairs);
    }
    if (requestId !== combinationsRequestId) return;
    if (!pairs.length) {
      combosDiv.innerHTML = `<span class="muted">No combinations found for ${escapeHtml(targetName)}.</span>`;
      return;
    }

    const visible = pairs.slice(0, COMBO_LIST_MAX);
    const items = visible
      .map((pair, index) => {
        const withImages = index < COMBO_LIST_WITH_IMAGES;
        const pairBody = withImages
          ? `<div class="combo-pair">${buildPalChip(pair.a)}<span class="flow-symbol">+</span>${buildPalChip(pair.b)}</div>`
          : `<div class="combo-pair"><strong>${escapeHtml(pair.a)}</strong><span class="flow-symbol">+</span><strong>${escapeHtml(pair.b)}</strong></div>`;
        return `<li class="combo-item">${pairBody}<span class="muted">(${escapeHtml(pair.method)})</span></li>`;
      })
      .join("");

    const more =
      pairs.length > visible.length
        ? `<p class="muted">Showing ${visible.length} of ${pairs.length} combinations${visible.length > COMBO_LIST_WITH_IMAGES ? " (pics on first " + COMBO_LIST_WITH_IMAGES + ")" : ""}.</p>`
        : "";

    combosDiv.innerHTML = `
    <div class="target-row"><span>Found <strong>${pairs.length}</strong> combinations for <strong>${escapeHtml(targetName)}</strong></span></div>
    <ul class="combo-list">${items}</ul>
    ${more}
  `;
  } catch (error) {
    if (requestId === combinationsRequestId) {
      combosDiv.innerHTML = `<span class="muted">Could not load combinations: ${escapeHtml(error.message)}</span>`;
    }
  } finally {
    if (requestId === combinationsRequestId) {
      findCombosBtn.disabled = false;
    }
  }
}

// Cards are created once per Pal and reused, so searching only moves existing nodes.
const palCardCache = new Map();
let palGridExpanded = false;
let palGridToken = 0;

function palCard(pal) {
  let card = palCardCache.get(pal.name);
  if (!card) {
    const template = document.createElement("template");
    template.innerHTML = `<button class="pal-grid-card" type="button" data-pal-name="${escapeHtml(pal.name)}">${buildPalImage(pal.name, "pal-grid-img")}<div class="pal-grid-name">${escapeHtml(pal.name)}</div><div class="pal-power">Power ${pal.power}</div></button>`;
    card = template.content.firstElementChild;
    palCardCache.set(pal.name, card);
  }
  return card;
}

// Appends cards in idle-time chunks so a long list never blocks input.
function fillPalGrid(pals) {
  const token = ++palGridToken;
  palGrid.replaceChildren();
  let next = 0;
  const appendChunk = () => {
    if (token !== palGridToken) return;
    const fragment = document.createDocumentFragment();
    const end = Math.min(next + PAL_GRID_CHUNK, pals.length);
    for (; next < end; next++) fragment.appendChild(palCard(pals[next]));
    palGrid.appendChild(fragment);
    if (next < pals.length) whenIdle(appendChunk);
  };
  appendChunk();
}

function renderPalGrid() {
  const query = String(palSearch.value || "").trim().toLowerCase();
  if (showAllPalsBtn) {
    showAllPalsBtn.hidden = palGridExpanded || Boolean(query);
    showAllPalsBtn.textContent = `Show all ${appData.pals.length} Pals`;
  }
  if (!query && !palGridExpanded) {
    ++palGridToken;
    palGrid.replaceChildren(...featuredPalCards);
    if (palGridStatus) palGridStatus.textContent = "Featured Pals. Tap one to use it in the calculator.";
    return;
  }
  const pals = query ? appData.pals.filter((pal) => pal.name.toLowerCase().includes(query)) : appData.pals;
  if (palGridStatus) {
    palGridStatus.textContent = query
      ? `${pals.length} Pal${pals.length === 1 ? "" : "s"} match "${palSearch.value.trim()}".`
      : `${pals.length} Pals in database.`;
  }
  fillPalGrid(pals);
}

// ---- Chain planner roster: searchable chips (works with touch, unlike <select multiple>) ----
const chainRoster = [];

function renderChainChips() {
  if (!chainAdditionalChips) return;
  chainAdditionalChips.replaceChildren(
    ...chainRoster.map((name) => {
      const chip = document.createElement("li");
      chip.className = "roster-chip";
      chip.innerHTML = `<span>${escapeHtml(name)}</span><button type="button" class="roster-chip-remove" data-remove="${escapeHtml(name)}" aria-label="Remove ${escapeHtml(name)}">×</button>`;
      return chip;
    })
  );
}

function renderChainSuggestions() {
  if (!chainAdditionalOptions || !chainAdditionalSearch) return;
  const query = chainAdditionalSearch.value.trim().toLowerCase();
  if (!query) {
    chainAdditionalOptions.hidden = true;
    chainAdditionalOptions.replaceChildren();
    return;
  }
  const matches = appData.pals
    .filter((pal) => pal.name.toLowerCase().includes(query) && !chainRoster.includes(pal.name))
    .sort((x, y) => Number(!x.name.toLowerCase().startsWith(query)) - Number(!y.name.toLowerCase().startsWith(query)))
    .slice(0, CHAIN_SUGGESTIONS_MAX);
  chainAdditionalOptions.innerHTML = matches.length
    ? matches
        .map((pal) => `<button type="button" class="chip-option" data-add="${escapeHtml(pal.name)}" aria-label="Add ${escapeHtml(pal.name)}">+ ${escapeHtml(pal.name)}</button>`)
        .join("")
    : `<span class="muted">No matching species.</span>`;
  chainAdditionalOptions.hidden = false;
}

function addToChainRoster(name) {
  const pal = palByNameLower.get(String(name).toLowerCase());
  if (!pal || chainRoster.includes(pal.name)) return;
  chainRoster.push(pal.name);
  renderChainChips();
  if (chainAdditionalSearch) chainAdditionalSearch.value = "";
  renderChainSuggestions();
}

function removeFromChainRoster(name) {
  const index = chainRoster.indexOf(name);
  if (index < 0) return;
  chainRoster.splice(index, 1);
  renderChainChips();
  renderChainSuggestions();
}

function quickPickPal(palName) {
  const target = pickForSelect.value;
  if (target === "parentA") {
    parentASelect.value = palName;
    scheduleRenderResult();
    return;
  }
  if (target === "parentB") {
    parentBSelect.value = palName;
    scheduleRenderResult();
    return;
  }
  targetChildSelect.value = palName;
  combosDiv.innerHTML = `<span class="muted">Target set to <strong>${escapeHtml(palName)}</strong>. Click <strong>Find Combinations</strong>.</span>`;
}

function renderKidBackground() {
  if (!kidBg) return;
  // Mobile PSI: skip decorative stickers (fixed/in-viewport → always download).
  if (window.matchMedia("(max-width: 900px), (prefers-reduced-motion: reduce)").matches) {
    kidBg.replaceChildren();
    return;
  }
  const pals = ["Anubis", "Jetragon", "Frostallion", "Lamball", "Foxparks", "Blazamut"];
  const slots = [
    { l: 2, t: 8, s: 56, r: -10 },
    { l: 90, t: 6, s: 52, r: 12 },
    { l: 4, t: 72, s: 48, r: 8 },
    { l: 88, t: 70, s: 50, r: -8 },
    { l: 8, t: 40, s: 44, r: -6 },
    { l: 86, t: 38, s: 44, r: 7 }
  ];
  const frag = document.createDocumentFragment();
  slots.forEach((sl, i) => {
    const img = document.createElement("img");
    img.className = "bg-pal-sticker";
    img.src = getPalImageUrl(pals[i % pals.length]);
    img.alt = "";
    img.width = sl.s;
    img.height = sl.s;
    img.decoding = "async";
    img.fetchPriority = "low";
    img.loading = "lazy";
    img.style.cssText = `left:${sl.l}%;top:${sl.t}%;width:${sl.s}px;height:${sl.s}px;transform:rotate(${sl.r}deg);animation-delay:${(i % 8) * 0.22}s;animation-duration:${7 + (i % 6)}s;`;
    img.onerror = () => {
      img.onerror = null;
      img.src = PAL_PLACEHOLDER;
    };
    frag.appendChild(img);
  });
  kidBg.replaceChildren(frag);
}

function resolvePalName(raw, palNames) {
  if (!raw) {
    return null;
  }
  if (palNames.has(raw)) {
    return raw;
  }
  const query = raw.toLowerCase();
  const slug = palSlug(raw);
  for (const name of palNames) {
    if (name.toLowerCase() === query || palSlug(name) === slug) {
      return name;
    }
  }
  return null;
}

function applyParentQueryFromUrl() {
  const params = new URLSearchParams(globalThis.location.search);
  const palNames = new Set(appData.pals.map((p) => p.name));
  const matchedA = resolvePalName(params.get("parentA") || params.get("parent_a"), palNames);
  const matchedB = resolvePalName(params.get("parentB") || params.get("parent_b"), palNames);
  if (matchedA) {
    parentASelect.value = matchedA;
  }
  if (matchedB) {
    parentBSelect.value = matchedB;
  }
}

/** Reverse lookup and chain planner fields; run after their selects are filled. */
function applyQueryFromUrl() {
  const params = new URLSearchParams(globalThis.location.search);
  const target = params.get("target") || params.get("child");
  const owned = params.get("owned");
  const goal = params.get("goal");
  const palNames = new Set(appData.pals.map((p) => p.name));

  const matchedTarget = resolvePalName(target, palNames);
  const matchedOwned = resolvePalName(owned, palNames);
  const matchedGoal = resolvePalName(goal, palNames);
  for (const name of params.get("also_owned")?.split(",") || []) {
    const matched = resolvePalName(name.trim(), palNames);
    if (matched) addToChainRoster(matched);
  }
  if (matchedTarget) {
    targetChildSelect.value = matchedTarget;
  }
  if (chainOwnedSelect && matchedOwned) {
    chainOwnedSelect.value = matchedOwned;
  }
  if (chainGoalSelect && matchedGoal) {
    chainGoalSelect.value = matchedGoal;
  }
  return {
    hasTarget: Boolean(matchedTarget),
    hasChain: Boolean(matchedOwned && matchedGoal)
  };
}

async function bootstrap() {
  appData = await apiFetch("/api/bootstrap");
  buildPalIndexes();
  const dataNote = document.getElementById("dataNote");
  if (dataNote && appData.data_provenance) {
    const metadata = appData.data_provenance;
    dataNote.textContent = `${appData.pals.length} species · ${formatCount(metadata.outcome_count)} source outcomes · Data imported ${metadata.imported_on}. Species predictions use a pinned PalCalc community snapshot; mutations and passive traits are not modeled.`;
  }
  // The server already rendered the breeding hero strip; keep it on the homepage.
  lastHeroView = heroStrip?.childElementCount ? "breeding" : "";
  for (const card of featuredPalCards) {
    const pal = palByNameLower.get(card.dataset.palName?.toLowerCase());
    if (pal) palCardCache.set(pal.name, card);
  }
  // The calculator comes first; the other selects are filled when the browser is idle.
  populateSelect(parentASelect);
  populateSelect(parentBSelect);

  // A filled <select> always reports the first option, so an empty-value
  // check never applied these defaults and both parents stayed the same Pal.
  parentASelect.value = "Anubis";
  parentBSelect.value = "Jetragon";
  applyParentQueryFromUrl();
  renderResult();
  // Let the calculator paint before the rest of the page is wired up.
  await yieldToMain();

  const savedTheme = localStorage.getItem("palworldTheme");
  setTheme(savedTheme === "light" ? "light" : "dark");

  renderStatsBar();
  renderPalGrid();
  whenIdle(renderKidBackground, 2000);
  const initialView = resolveViewFromPath(globalThis.location.pathname);
  const panelPromise = renderDatabasePanel(initialView);
  if (initialView === "map") {
    await panelPromise;
  } else {
    void panelPromise;
  }

  const secondarySelects = [targetChildSelect, chainOwnedSelect, chainGoalSelect].filter(Boolean);
  const populateNextSelect = () => {
    const select = secondarySelects.shift();
    if (select) {
      populateSelect(select);
      whenIdle(populateNextSelect, 300);
      return;
    }
    targetChildSelect.value = "Frostallion";
    runDeferred(applyQueryFromUrl());
  };

  const runDeferred = ({ hasTarget, hasChain }) => {
    if (hasTarget) {
      void renderCombinations();
    } else if (combosDiv) {
      combosDiv.innerHTML =
        '<span class="muted">Select a target Pal and click <strong>Find Combinations</strong>.</span>';
    }
    if (hasChain || initialView === "chain") {
      void renderChainBreeding();
    } else if (chainResultDiv) {
      chainResultDiv.innerHTML =
        '<span class="muted">Pick the Pal you own and your goal, then click <strong>Find Breeding Chain</strong>.</span>';
    }
  };
  whenIdle(populateNextSelect, 300);
  setTimeout(() => focusViewCard(initialView), 120);
}

calculateBtn.addEventListener("click", () => renderResult({ celebrate: true }));
swapBtn.addEventListener("click", () => {
  const currentA = parentASelect.value;
  parentASelect.value = parentBSelect.value;
  parentBSelect.value = currentA;
  if (parentAGenderSelect && parentBGenderSelect) {
    const genderA = parentAGenderSelect.value;
    parentAGenderSelect.value = parentBGenderSelect.value;
    parentBGenderSelect.value = genderA;
  }
  renderResult({ celebrate: true });
});
findCombosBtn.addEventListener("click", () => renderCombinations());
findChainBtn?.addEventListener("click", () => renderChainBreeding());
themeToggleBtn.addEventListener("click", () => {
  const nextTheme = document.body.dataset.theme === "light" ? "dark" : "light";
  setTheme(nextTheme);
});
navButtons.forEach((button) => {
  button.addEventListener("click", (event) => {
    const view = button.dataset.view;
    if (button instanceof HTMLAnchorElement) {
      event.preventDefault();
      const href = button.getAttribute("href");
      if (href) {
        globalThis.history.pushState({}, "", href);
      }
    }
    renderDatabasePanel(view);
    focusViewCard(view);
  });
});
globalThis.addEventListener("popstate", () => {
  const view = resolveViewFromPath(globalThis.location.pathname);
  renderDatabasePanel(view);
  focusViewCard(view);
});
function onParentSelectUpdate() {
  scheduleRenderResult();
}

parentAGenderSelect?.addEventListener("change", onParentSelectUpdate);
parentBGenderSelect?.addEventListener("change", onParentSelectUpdate);
parentASelect.addEventListener("change", onParentSelectUpdate);
parentBSelect.addEventListener("change", onParentSelectUpdate);
parentASelect.addEventListener("input", onParentSelectUpdate);
parentBSelect.addEventListener("input", onParentSelectUpdate);
palSearch.addEventListener("input", debounce(renderPalGrid, 150));
showAllPalsBtn?.addEventListener("click", () => {
  palGridExpanded = true;
  renderPalGrid();
});
chainAdditionalSearch?.addEventListener("input", debounce(renderChainSuggestions, 100));
chainAdditionalSearch?.addEventListener("keydown", (event) => {
  if (event.key !== "Enter") return;
  event.preventDefault();
  renderChainSuggestions();
  const first = chainAdditionalOptions?.querySelector("[data-add]");
  if (first) addToChainRoster(first.dataset.add);
});
chainAdditionalOptions?.addEventListener("click", (event) => {
  const option = event.target.closest("[data-add]");
  if (!option) return;
  addToChainRoster(option.dataset.add);
  chainAdditionalSearch?.focus();
});
chainAdditionalChips?.addEventListener("click", (event) => {
  const remove = event.target.closest("[data-remove]");
  if (remove) removeFromChainRoster(remove.dataset.remove);
});
palGrid.addEventListener("click", (event) => {
  const button = event.target.closest("[data-pal-name]");
  if (!button) {
    return;
  }
  quickPickPal(button.dataset.palName);
});

try {
  await bootstrap();
} catch (error) {
  console.error(error);
  renderKidBackground();
  if (databasePanelTitle && databasePanelBody) {
    databasePanelTitle.textContent = "Rust API Required";
    databasePanelBody.innerHTML =
      "Start the server with <code>cargo run</code> or <strong>START-SERVER.bat</strong>, then refresh this page.";
  }
  if (resultDiv) {
    resultDiv.innerHTML =
      '<span class="muted">Server not running. Double-click <strong>START-SERVER.bat</strong> in the project folder.</span>';
  }
}

