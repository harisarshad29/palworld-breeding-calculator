const parentASelect = document.getElementById("parentA");
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
const palBoxCard = document.getElementById("palBoxCard");
const heroStrip = document.getElementById("heroStrip");
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
  "/palworld-capture-rate-calculator": "capture"
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
    subtitle: "Use capture estimates and easier targets to plan parent farming efficiently.",
    focusCardId: "reverseCard"
  }
};
const routeIconSeeds = {
  breeding: ["anubis", "jetragon", "frostallion", "blazamut", "suzaku", "jormuntide", "necromus"],
  pals: ["lamball", "cattiva", "chikipi", "lifmunk", "tanzee", "foxparks", "rooby"],
  map: ["eikthyrdeer", "pengullet", "daedream", "pyrin", "anubis", "jetragon", "suzaku"],
  items: ["lamball", "foxparks", "rooby", "frostallion", "jolthog", "pengullet", "anubis"],
  technology: ["lifmunk", "tanzee", "eikthyrdeer", "jormuntide", "paladius", "necromus", "jetragon"],
  capture: ["jetragon", "frostallion", "necromus", "paladius", "jormuntide", "anubis", "suzaku"]
};

let appData = {
  pals: [],
  pal_locations: {},
  items: [],
  technologies: [],
  special_combos_count: 0
};

/** Show every Pal in Pal Box (full dataset is 198) */
const PAL_GRID_INITIAL = 9999;
let locationsLoaded = false;
let locationsLoading = null;
let lastHeroView = "";
let combinationsRequestId = 0;
const MAP_ROWS_INITIAL = 9999;
const serverRouteIntro =
  document.body.dataset.routeIntro?.trim() ||
  document.getElementById("routeSubtitle")?.textContent?.trim() ||
  "";

function getPalAltText(palName) {
  return `${palName} Palworld breeding combination`;
}

function palSlug(palName) {
  return palName
    .toLowerCase()
    .trim()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
}

function getPalImageUrl(palName) {
  return `/assets/pals/${palSlug(palName)}.webp`;
}

const PAL_PLACEHOLDER = "/assets/pals/placeholder.svg";
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
  }, 100);
}

function buildPalChip(palName) {
  return `
    <div class="pal-chip">
      <img class="pal-image" src="${getPalImageUrl(palName)}" alt="${getPalAltText(palName)}" loading="lazy"
        onerror="this.onerror=null;this.src='${PAL_PLACEHOLDER}'" />
      <span>${palName}</span>
    </div>
  `;
}

function buildArenaCard(palName, role) {
  const roleClass = role === "child" ? " child" : "";
  return `
    <div class="arena-card${roleClass}">
      <img src="${getPalImageUrl(palName)}" alt="${getPalAltText(palName)}" loading="lazy"
        onerror="this.onerror=null;this.src='${PAL_PLACEHOLDER}'" />
      <div class="arena-name">${palName}</div>
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
    <span class="stat-pill">Special combos: <strong>${appData.special_combos_count}</strong></span>
    <span class="stat-pill">Parent pair checks: <strong>${totalPairs}</strong></span>
  `;
}

function celebrateConfetti() {
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
  if (locationsLoaded) {
    return;
  }
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
    renderHeroStrip(view);
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
      <div class="database-caption">Starter technology milestones for progression.</div>
      <table class="database-table">
        <thead><tr><th>Level</th><th>Technology</th><th>Cost</th></tr></thead>
        <tbody>${rows}</tbody>
      </table>
    `;
    return;
  }

  if (view === "capture") {
    const targetName = targetChildSelect.value || appData.pals[0].name;
    const capture = await apiFetch(`/api/capture/${encodeURIComponent(targetName)}`);
    const list = capture.easier_targets.map((pal) => `<li>${pal.name} (${pal.power})</li>`).join("");
    databasePanelTitle.textContent = "Capture Rate";
    databasePanelBody.innerHTML = `
      <div class="database-caption">
        Estimated capture chance for <strong>${capture.target}</strong>: <strong>${capture.estimate_percent}%</strong>
      </div>
      <div class="database-caption">Easier capture targets in this roster:</div>
      <ul class="database-list">${list}</ul>
    `;
    return;
  }

  databasePanelTitle.textContent = "Breeding Calculator";
  databasePanelBody.innerHTML = `Breeding calculator is active. Current special combinations: <strong>${appData.special_combos_count}</strong>.`;
}

function renderHeroStrip(view) {
  const icons = (routeIconSeeds[view] || routeIconSeeds.breeding).slice(0, 5);
  heroStrip.innerHTML = icons
    .map(
      (name) => `
      <img class="hero-icon" src="/assets/pals/${name}.webp" alt="${name}" loading="lazy"
        onerror="this.onerror=null;this.src='${PAL_PLACEHOLDER}'" />
    `
    )
    .join("");
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
  if (pathToView[normalized]) {
    return pathToView[normalized];
  }
  if (normalized.startsWith("/pal/")) {
    return "pals";
  }
  if (normalized.startsWith("/combo/")) {
    return "breeding";
  }
  if (normalized.startsWith("/guides/")) {
    return "breeding";
  }
  return "breeding";
}

function focusViewCard(view) {
  const meta = viewMeta[view] || viewMeta.breeding;
  const target = document.getElementById(meta.focusCardId);
  if (!target) {
    return;
  }
  target.scrollIntoView({ behavior: "smooth", block: "start" });
}

async function renderResult(options = {}) {
  const { celebrate = false } = options;
  const { parentA, parentB } = getParentNames();

  if (!parentA || !parentB) {
    showResultError("Choose both Parent A and Parent B.");
    return;
  }

  if (parentA === parentB) {
    showResultError("Parent A and Parent B must be different Pals.");
    return;
  }

  const requestId = ++calculateRequestId;
  calculateBtn.disabled = true;
  resultDiv.innerHTML = `<div class="muted">Calculating ${escapeHtml(parentA)} + ${escapeHtml(parentB)}…</div>`;

  try {
    const calc = await apiFetch("/api/calculate", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ parent_a: parentA, parent_b: parentB })
    });

    if (requestId !== calculateRequestId) {
      return;
    }

    const details =
      typeof calc.distance === "number"
        ? `<div class="muted">Closest power distance: ${calc.distance}</div>`
        : "";

    resultDiv.innerHTML = `
      <div class="arena">
        ${buildArenaCard(parentA, "parent")}
        <span class="flow-symbol">+</span>
        ${buildArenaCard(parentB, "parent")}
        <span class="flow-symbol">=</span>
        ${buildArenaCard(calc.child.name, "child")}
      </div>
      <div class="muted">${escapeHtml(calc.method)}</div>
      ${details}
    `;

    if (celebrate) {
      celebrateConfetti();
    }
  } catch (error) {
    if (requestId !== calculateRequestId) {
      return;
    }
    console.error(error);
    showResultError(
      error.message?.includes("Invalid parent")
        ? "Could not find those Pals. Refresh the page or pick names from the list."
        : `Calculation failed: ${error.message}`
    );
  } finally {
    if (requestId === calculateRequestId) {
      calculateBtn.disabled = false;
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
    const pairs = await apiFetch(`/api/combinations/${encodeURIComponent(targetName)}`);
    if (requestId !== combinationsRequestId) {
      return;
    }
    if (!pairs.length) {
      combosDiv.innerHTML = `<span class="muted">No combinations found for ${escapeHtml(targetName)}.</span>`;
      return;
    }

    const items = pairs
      .slice(0, 80)
      .map(
        (pair) => `
        <li class="combo-item">
          <div class="combo-pair"><strong>${escapeHtml(pair.a)}</strong><span class="flow-symbol">+</span><strong>${escapeHtml(pair.b)}</strong></div>
          <span class="muted">(${escapeHtml(pair.method)})</span>
        </li>
      `
      )
      .join("");

    const more =
      pairs.length > 80 ? `<p class="muted">Showing 80 of ${pairs.length} combinations.</p>` : "";

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

function renderPalGrid(showAll = false) {
  const query = String(palSearch.value || "").trim().toLowerCase();
  const filtered = appData.pals.filter((pal) => pal.name.toLowerCase().includes(query));
  const limit = showAll || query ? filtered.length : PAL_GRID_INITIAL;
  const visible = filtered.slice(0, limit);

  document.querySelector(".load-more-pals")?.remove();
  document.querySelector(".pal-grid-count")?.remove();
  palGrid.innerHTML = visible
    .map(
      (pal) => `
      <button class="pal-grid-card" type="button" data-pal-name="${escapeHtml(pal.name)}">
        <img src="${getPalImageUrl(pal.name)}" alt="${getPalAltText(pal.name)}" loading="lazy" decoding="async"
          onerror="this.onerror=null;this.src='${PAL_PLACEHOLDER}'" />
        <div class="pal-grid-name">${escapeHtml(pal.name)}</div>
        <div class="pal-power">Power ${pal.power}</div>
      </button>
    `
    )
    .join("");

  if (!showAll && !query && filtered.length > visible.length) {
    palGrid.insertAdjacentHTML(
      "afterend",
      `<button type="button" class="ghost-btn load-more-pals">Show all ${filtered.length} Pals (${visible.length} visible)</button>`
    );
    document.querySelector(".load-more-pals")?.addEventListener("click", () => renderPalGrid(true));
  } else if (!query && filtered.length > 0) {
    palGrid.insertAdjacentHTML(
      "afterend",
      `<p class="muted pal-grid-count">${filtered.length} Pals in database</p>`
    );
  }
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
  const pals = ["Lamball","Cattiva","Chikipi","Foxparks","Pengullet","Anubis","Jetragon","Frostallion","Blazamut","Suzaku","Necromus","Paladius","Relaxaurus","Penking","Elizabee","Grizzbolt","Lyleen","Mossanda","Azurobe","Incineram","Beakon","Sibelyx","Astegon","Shadowbeak","Bellanoir","Kitsun","Rooby","Daedream"];
  const slots = [
    {l:1,t:4,s:76,r:-12},{l:3,t:28,s:58,r:6},{l:2,t:52,s:64,r:-8},{l:4,t:76,s:54,r:10},{l:1,t:90,s:62,r:-15},
    {l:88,t:3,s:72,r:14},{l:91,t:26,s:56,r:-9},{l:89,t:48,s:68,r:11},{l:92,t:70,s:60,r:-7},{l:87,t:88,s:66,r:16},
    {l:14,t:2,s:50,r:8},{l:78,t:2,s:48,r:-11},{l:8,t:42,s:44,r:-5},{l:84,t:38,s:46,r:7},{l:6,t:64,s:42,r:12},
    {l:86,t:58,s:44,r:-13},{l:18,t:86,s:40,r:6},{l:76,t:84,s:42,r:-8},{l:22,t:14,s:38,r:-4},{l:72,t:16,s:38,r:5},
    {l:10,t:18,s:36,r:9},{l:82,t:20,s:36,r:-6}
  ];
  const frag = document.createDocumentFragment();
  slots.forEach((sl, i) => {
    const img = document.createElement("img");
    img.className = "bg-pal-sticker";
    img.src = getPalImageUrl(pals[i % pals.length]);
    img.alt = "";
    img.loading = "lazy";
    img.style.cssText = `left:${sl.l}%;top:${sl.t}%;width:${sl.s}px;height:${sl.s}px;transform:rotate(${sl.r}deg);animation-delay:${(i%8)*0.22}s;animation-duration:${7+(i%6)}s;`;
    img.onerror = () => { img.onerror = null; img.src = PAL_PLACEHOLDER; };
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
  for (const name of palNames) {
    if (name.toLowerCase() === query) {
      return name;
    }
  }
  return null;
}

function applyQueryFromUrl() {
  const params = new URLSearchParams(globalThis.location.search);
  const parentA = params.get("parentA") || params.get("parent_a");
  const parentB = params.get("parentB") || params.get("parent_b");
  const target = params.get("target") || params.get("child");
  const palNames = new Set(appData.pals.map((p) => p.name));

  const matchedA = resolvePalName(parentA, palNames);
  const matchedB = resolvePalName(parentB, palNames);
  const matchedTarget = resolvePalName(target, palNames);
  if (matchedA) {
    parentASelect.value = matchedA;
  }
  if (matchedB) {
    parentBSelect.value = matchedB;
  }
  if (matchedTarget) {
    targetChildSelect.value = matchedTarget;
  }
  return { hasTarget: Boolean(matchedTarget) };
}

async function bootstrap() {
  renderKidBackground();
  appData = await apiFetch("/api/bootstrap");
  populateSelect(parentASelect);
  populateSelect(parentBSelect);
  populateSelect(targetChildSelect);

  const { hasTarget } = applyQueryFromUrl();
  if (!parentASelect.value) {
    parentASelect.value = "Anubis";
  }
  if (!parentBSelect.value) {
    parentBSelect.value = "Jetragon";
  }
  if (!targetChildSelect.value) {
    targetChildSelect.value = "Frostallion";
  }

  const savedTheme = localStorage.getItem("palworldTheme");
  setTheme(savedTheme === "light" ? "light" : "dark");

  renderStatsBar();
  renderPalGrid(true);
  const initialView = resolveViewFromPath(globalThis.location.pathname);
  const panelPromise = renderDatabasePanel(initialView);
  if (initialView === "map") {
    await panelPromise;
  } else {
    void panelPromise;
  }

  const runDeferred = () => {
    void renderResult();
    if (hasTarget) {
      void renderCombinations();
    } else {
      combosDiv.innerHTML =
        '<span class="muted">Select a target Pal and click <strong>Find Combinations</strong>.</span>';
    }
  };
  if (typeof requestIdleCallback === "function") {
    requestIdleCallback(runDeferred, { timeout: 600 });
  } else {
    setTimeout(runDeferred, 50);
  }
  setTimeout(() => focusViewCard(initialView), 120);
}

calculateBtn.addEventListener("click", async () => renderResult({ celebrate: true }));
swapBtn.addEventListener("click", async () => {
  const currentA = parentASelect.value;
  parentASelect.value = parentBSelect.value;
  parentBSelect.value = currentA;
  await renderResult({ celebrate: true });
});
findCombosBtn.addEventListener("click", () => renderCombinations());
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

parentASelect.addEventListener("change", onParentSelectUpdate);
parentBSelect.addEventListener("change", onParentSelectUpdate);
parentASelect.addEventListener("input", onParentSelectUpdate);
parentBSelect.addEventListener("input", onParentSelectUpdate);
palSearch.addEventListener("input", renderPalGrid);
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

