import { buildComboMap, calculateChild, combinationsForTarget, findPal } from "./logic.js";

const $ = (id) => document.getElementById(id);
const parentASelect = $("parentA");
const parentBSelect = $("parentB");
const targetChildSelect = $("targetChild");
const calculateBtn = $("calculateBtn");
const swapBtn = $("swapBtn");
const pickForSelect = $("pickFor");
const findCombosBtn = $("findCombosBtn");
const resultDiv = $("result");
const combosDiv = $("combos");
const statsBar = $("statsBar");
const themeToggleBtn = $("themeToggleBtn");
const palSearch = $("palSearch");
const palGrid = $("palGrid");
const confettiLayer = $("confettiLayer");
const databasePanelTitle = $("databasePanelTitle");
const databasePanelBody = $("databasePanelBody");
const routeBadge = $("routeBadge");
const routeSubtitle = $("routeSubtitle");
const heroStrip = $("heroStrip");
const navButtons = document.querySelectorAll("[data-view]");

const PAL_GRID_INITIAL = 48;
const MAP_ROWS_INITIAL = 60;
const PAL_PLACEHOLDER = "assets/pals/placeholder.svg";

const STATIC_ITEMS = [
  { item: "Wool", source: "Lamball", notes: "Ranch / drops" },
  { item: "Leather", source: "Foxparks", notes: "Frequent drop" },
  { item: "Flame Organ", source: "Rooby", notes: "Fire pal material" },
  { item: "Ice Organ", source: "Frostallion", notes: "Late-game material" },
  { item: "Electric Organ", source: "Jolthog", notes: "Electric crafting" },
];

const STATIC_TECH = [
  { level: 2, name: "Pal Sphere", cost: "1 Tech Point" },
  { level: 6, name: "Egg Incubator", cost: "2 Tech Points" },
  { level: 7, name: "Breeding Farm", cost: "2 Tech Points" },
  { level: 20, name: "Electric Kitchen", cost: "3 Tech Points" },
];

const viewMeta = {
  breeding: { badge: "Breeding Calculator", focus: "arenaCard" },
  pals: { badge: "Pals Database", focus: "databaseCard" },
  map: { badge: "Map Reference", focus: "databaseCard" },
  items: { badge: "Items", focus: "databaseCard" },
  technology: { badge: "Technology", focus: "databaseCard" },
  capture: { badge: "Capture Rate", focus: "reverseCard" },
};

const routeIconSeeds = {
  breeding: ["anubis", "jetragon", "frostallion", "blazamut", "suzaku"],
  pals: ["lamball", "cattiva", "chikipi", "foxparks", "rooby"],
  map: ["eikthyrdeer", "pengullet", "anubis", "jetragon", "suzaku"],
  items: ["lamball", "foxparks", "rooby", "frostallion", "jolthog"],
  technology: ["lifmunk", "tanzee", "eikthyrdeer", "jormuntide", "paladius"],
  capture: ["jetragon", "frostallion", "necromus", "paladius", "anubis"],
};

let appData = { pals: [], pal_locations: {}, special_combos: {}, special_combos_count: 0 };
let combinationsRequestId = 0;
let lastHeroView = "";

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

function getPalImageUrl(name) {
  return `assets/pals/${palSlug(name)}.webp`;
}

function restoreRouteIntro() {
  const intro = document.body.dataset.routeIntro?.trim();
  if (intro && routeSubtitle) routeSubtitle.textContent = intro;
}

function resolveView() {
  const hash = (location.hash || "#breeding").replace(/^#/, "");
  return viewMeta[hash] ? hash : "breeding";
}

function populateSelect(select) {
  const frag = document.createDocumentFragment();
  for (const pal of appData.pals) {
    const o = document.createElement("option");
    o.value = pal.name;
    o.textContent = pal.name;
    frag.appendChild(o);
  }
  select.replaceChildren(frag);
}

function setTheme(name) {
  document.body.dataset.theme = name;
  localStorage.setItem("palworldTheme", name);
  themeToggleBtn.textContent = name === "light" ? "Dark Mode" : "Light Mode";
}

function renderStats() {
  const pairs = (appData.pals.length * (appData.pals.length + 1)) / 2;
  statsBar.innerHTML = `
    <span class="stat-pill">Pals: <strong>${appData.pals.length}</strong></span>
    <span class="stat-pill">Special combos: <strong>${appData.special_combos_count}</strong></span>
    <span class="stat-pill">Pair checks: <strong>${pairs}</strong></span>`;
}

function renderHero(view) {
  if (view === lastHeroView) return;
  lastHeroView = view;
  heroStrip.innerHTML = (routeIconSeeds[view] || routeIconSeeds.breeding)
    .map(
      (n) =>
        `<img class="hero-icon" src="assets/pals/${n}.webp" alt="" loading="lazy" onerror="this.onerror=null;this.src='${PAL_PLACEHOLDER}'" />`
    )
    .join("");
}

function renderDatabasePanel(view) {
  const meta = viewMeta[view] || viewMeta.breeding;
  routeBadge.textContent = meta.badge;
  restoreRouteIntro();
  renderHero(view);
  navButtons.forEach((b) => b.classList.toggle("is-active", b.dataset.view === view));

  if (view === "pals") {
    const top = [...appData.pals].sort((a, b) => b.power - a.power).slice(0, 3);
    databasePanelTitle.textContent = "Pals";
    databasePanelBody.innerHTML = `Total: <strong>${appData.pals.length}</strong>. Top power: ${top
      .map((p) => `<strong>${escapeHtml(p.name)}</strong> (${p.power})`)
      .join(", ")}.`;
    return;
  }

  if (view === "map") {
    databasePanelTitle.textContent = "Map";
    const entries = appData.pals
      .map((pal) => {
        const loc = appData.pal_locations[pal.name];
        return {
          name: pal.name,
          area: loc?.area || "Palpagos Island",
          coords: loc?.coords || "—",
        };
      })
      .sort((a, b) => a.name.localeCompare(b.name))
      .slice(0, MAP_ROWS_INITIAL);
    databasePanelBody.innerHTML = `
      <table class="database-table"><thead><tr><th>Pal</th><th>Area</th><th>Coords</th></tr></thead>
      <tbody>${entries
        .map(
          (e) =>
            `<tr><td>${escapeHtml(e.name)}</td><td>${escapeHtml(e.area)}</td><td>${escapeHtml(e.coords)}</td></tr>`
        )
        .join("")}</tbody></table>`;
    return;
  }

  if (view === "items") {
    databasePanelTitle.textContent = "Items";
    databasePanelBody.innerHTML = `<table class="database-table"><thead><tr><th>Item</th><th>Source</th><th>Notes</th></tr></thead><tbody>${STATIC_ITEMS.map(
      (i) => `<tr><td>${i.item}</td><td>${i.source}</td><td>${i.notes}</td></tr>`
    ).join("")}</tbody></table>`;
    return;
  }

  if (view === "technology") {
    databasePanelTitle.textContent = "Technology";
    databasePanelBody.innerHTML = `<table class="database-table"><thead><tr><th>Level</th><th>Name</th><th>Cost</th></tr></thead><tbody>${STATIC_TECH.map(
      (t) => `<tr><td>${t.level}</td><td>${t.name}</td><td>${t.cost}</td></tr>`
    ).join("")}</tbody></table>`;
    return;
  }

  if (view === "capture") {
    const target = findPal(appData.pals, targetChildSelect.value) || appData.pals[0];
    let est = Math.round((1600 - target.power) / 16);
    est = Math.max(4, Math.min(95, est));
    const easier = [...appData.pals].sort((a, b) => a.power - b.power).slice(0, 5);
    databasePanelTitle.textContent = "Capture Rate";
    databasePanelBody.innerHTML = `
      <p>Estimate for <strong>${escapeHtml(target.name)}</strong>: <strong>${est}%</strong></p>
      <ul>${easier.map((p) => `<li>${escapeHtml(p.name)} (${p.power})</li>`).join("")}</ul>`;
    return;
  }

  databasePanelTitle.textContent = "Breeding";
  databasePanelBody.innerHTML = `Calculator ready. Special combos: <strong>${appData.special_combos_count}</strong>.`;
}

function arenaCard(name, role) {
  return `<div class="arena-card${role === "child" ? " child" : ""}">
    <img src="${getPalImageUrl(name)}" alt="" loading="lazy" onerror="this.onerror=null;this.src='${PAL_PLACEHOLDER}'" />
    <div class="arena-name">${escapeHtml(name)}</div></div>`;
}

function renderResult(celebrate = false) {
  const a = parentASelect.value;
  const b = parentBSelect.value;
  if (!a || !b) {
    resultDiv.innerHTML = `<div class="result-error">Choose both parents.</div>`;
    return;
  }
  if (a === b) {
    resultDiv.innerHTML = `<div class="result-error">Parents must be different.</div>`;
    return;
  }
  const calc = calculateChild(appData.pals, appData.special_combos, a, b);
  if (!calc) {
    resultDiv.innerHTML = `<div class="result-error">Invalid parents.</div>`;
    return;
  }
  const dist =
    calc.distance != null ? `<div class="muted">Power distance: ${calc.distance}</div>` : "";
  resultDiv.innerHTML = `
    <div class="arena">
      ${arenaCard(a, "parent")}<span class="flow-symbol">+</span>
      ${arenaCard(b, "parent")}<span class="flow-symbol">=</span>
      ${arenaCard(calc.child.name, "child")}
    </div>
    <div class="muted">${escapeHtml(calc.method)}</div>${dist}`;
  if (celebrate) celebrateConfetti();
}

function renderCombinations() {
  const requestId = ++combinationsRequestId;
  const targetName = targetChildSelect.value;
  combosDiv.innerHTML = `<span class="muted">Searching…</span>`;
  findCombosBtn.disabled = true;
  setTimeout(() => {
    if (requestId !== combinationsRequestId) return;
    const pairs = combinationsForTarget(appData.pals, appData.special_combos, targetName);
    if (!pairs.length) {
      combosDiv.innerHTML = `<span class="muted">No combinations found.</span>`;
      findCombosBtn.disabled = false;
      return;
    }
    combosDiv.innerHTML = `
      <p>Found <strong>${pairs.length}</strong> for <strong>${escapeHtml(targetName)}</strong></p>
      <ul class="combo-list">${pairs
        .slice(0, 80)
        .map(
          (p) =>
            `<li class="combo-item"><strong>${escapeHtml(p.a)}</strong> + <strong>${escapeHtml(
              p.b
            )}</strong> <span class="muted">(${escapeHtml(p.method)})</span></li>`
        )
        .join("")}</ul>`;
    findCombosBtn.disabled = false;
  }, 30);
}

function renderPalGrid(showAll = false) {
  const q = palSearch.value.trim().toLowerCase();
  const filtered = appData.pals.filter((p) => p.name.toLowerCase().includes(q));
  const visible = filtered.slice(0, showAll || q ? filtered.length : PAL_GRID_INITIAL);
  document.querySelector(".load-more-pals")?.remove();
  palGrid.innerHTML = visible
    .map(
      (pal) => `
    <button type="button" class="pal-grid-card" data-pal-name="${escapeHtml(pal.name)}">
      <img src="${getPalImageUrl(pal.name)}" alt="" loading="lazy" onerror="this.onerror=null;this.src='${PAL_PLACEHOLDER}'" />
      <div class="pal-grid-name">${escapeHtml(pal.name)}</div>
      <div class="pal-power">Power ${pal.power}</div>
    </button>`
    )
    .join("");
  if (!showAll && !q && filtered.length > visible.length) {
    palGrid.insertAdjacentHTML(
      "afterend",
      `<button type="button" class="ghost-btn load-more-pals">Show all ${filtered.length} Pals</button>`
    );
    document.querySelector(".load-more-pals")?.addEventListener("click", () => renderPalGrid(true));
  }
}

function quickPick(name) {
  const t = pickForSelect.value;
  if (t === "parentA") parentASelect.value = name;
  else if (t === "parentB") parentBSelect.value = name;
  else targetChildSelect.value = name;
  renderResult();
}

function celebrateConfetti() {
  const colors = ["#ff4fd8", "#53e8ff", "#fff15f", "#7dff7a"];
  for (let i = 0; i < 24; i += 1) {
    const p = document.createElement("span");
    p.className = "confetti-piece";
    p.style.left = `${40 + Math.random() * 60}%`;
    p.style.top = "30%";
    p.style.background = colors[i % colors.length];
    p.style.animationDuration = `${700 + Math.random() * 500}ms`;
    confettiLayer.appendChild(p);
    setTimeout(() => p.remove(), 1500);
  }
}

function applyQuery() {
  const params = new URLSearchParams(location.search);
  const names = new Set(appData.pals.map((p) => p.name));
  const pick = (raw) => {
    if (!raw) return null;
    if (names.has(raw)) return raw;
    const lower = raw.toLowerCase();
    for (const n of names) if (n.toLowerCase() === lower) return n;
    return null;
  };
  const a = pick(params.get("parentA"));
  const b = pick(params.get("parentB"));
  const t = pick(params.get("target"));
  if (a) parentASelect.value = a;
  if (b) parentBSelect.value = b;
  if (t) targetChildSelect.value = t;
  return Boolean(t);
}

async function loadData() {
  const [palsRes, combosRes, locRes] = await Promise.all([
    fetch("data/pals.json"),
    fetch("data/special_combos.json"),
    fetch("data/pal_locations.json"),
  ]);
  const palsJson = await palsRes.json();
  const combosJson = await combosRes.json();
  const locJson = await locRes.json();
  return {
    pals: (palsJson.pals || []).sort((a, b) => a.name.localeCompare(b.name)),
    pal_locations: locJson.locations || {},
    special_combos: buildComboMap(combosJson.combos),
    special_combos_count: combosJson.combos?.length || 0,
  };
}

async function bootstrap() {
  appData = await loadData();
  populateSelect(parentASelect);
  populateSelect(parentBSelect);
  populateSelect(targetChildSelect);
  parentASelect.value = findPal(appData.pals, "Anubis")?.name || appData.pals[0]?.name;
  parentBSelect.value = findPal(appData.pals, "Jetragon")?.name || appData.pals[1]?.name;
  targetChildSelect.value = findPal(appData.pals, "Frostallion")?.name || appData.pals[0]?.name;
  setTheme(localStorage.getItem("palworldTheme") === "light" ? "light" : "dark");
  renderStats();
  const hasTarget = applyQuery();
  const view = resolveView();
  renderDatabasePanel(view);
  renderPalGrid();
  renderResult();
  if (hasTarget) renderCombinations();
  else combosDiv.innerHTML = `<span class="muted">Click <strong>Find Combinations</strong> for reverse lookup.</span>`;
}

calculateBtn.addEventListener("click", () => renderResult(true));
swapBtn.addEventListener("click", () => {
  const t = parentASelect.value;
  parentASelect.value = parentBSelect.value;
  parentBSelect.value = t;
  renderResult(true);
});
findCombosBtn.addEventListener("click", renderCombinations);
themeToggleBtn.addEventListener("click", () =>
  setTheme(document.body.dataset.theme === "light" ? "dark" : "light")
);
navButtons.forEach((btn) => {
  btn.addEventListener("click", (e) => {
    e.preventDefault();
    location.hash = btn.dataset.view;
    renderDatabasePanel(btn.dataset.view);
    document.getElementById(viewMeta[btn.dataset.view]?.focus || "arenaCard")?.scrollIntoView({
      behavior: "smooth",
    });
  });
});
window.addEventListener("hashchange", () => renderDatabasePanel(resolveView()));
palSearch.addEventListener("input", () => renderPalGrid());
palGrid.addEventListener("click", (e) => {
  const btn = e.target.closest("[data-pal-name]");
  if (btn) quickPick(btn.dataset.palName);
});

bootstrap().catch(() => {
  resultDiv.innerHTML = `<p class="result-error">Failed to load data. Use a web server or deploy to Netlify.</p>`;
});
