import fs from "fs";

const path = "static-site/app.js";
let s = fs.readFileSync(path, "utf8");

s = `import { buildComboMap, calculateChild, combinationsForTarget, findPal } from "./logic.js";\n\n` + s;

s = s.replace(
  'const PAL_PLACEHOLDER = "/assets/pals/placeholder.svg";',
  'const PAL_PLACEHOLDER = "assets/pals/placeholder.svg";'
);
s = s.replace(
  'return `/assets/pals/${palSlug(palName)}.webp`;',
  'return `assets/pals/${palSlug(palName)}.webp`;'
);
s = s.replace(
  'src="/assets/pals/${name}.webp"',
  'src="assets/pals/${name}.webp"'
);

s = s.replace(
  `let appData = {
  pals: [],
  pal_locations: {},
  items: [],
  technologies: [],
  special_combos_count: 0
};`,
  `let appData = {
  pals: [],
  pal_locations: {},
  items: [],
  technologies: [],
  special_combos_count: 0,
  special_combos: {},
};`
);

s = s.replace(
  `const serverRouteIntro =
  document.body.dataset.routeIntro?.trim() ||
  document.getElementById("routeSubtitle")?.textContent?.trim() ||
  "";`,
  `const serverRouteIntro =
  document.body.dataset.routeIntro?.trim() ||
  document.getElementById("routeSubtitle")?.textContent?.trim() ||
  "";

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
];`
);

s = s.replace(
  `async function apiFetch(url, options) {
  const response = await fetch(url, options);
  if (!response.ok) {
    const message = await response.text();
    throw new Error(message || \`API failed: \${response.status}\`);
  }
  return response.json();
}`,
  `async function loadStaticData() {
  const [palsRes, combosRes, locRes] = await Promise.all([
    fetch("data/pals.json"),
    fetch("data/special_combos.json"),
    fetch("data/pal_locations.json"),
  ]);
  const palsJson = await palsRes.json();
  const combosJson = await combosRes.json();
  const locJson = await locRes.json();
  const special = buildComboMap(combosJson.combos);
  return {
    pals: palsJson.pals || [],
    pal_locations: locJson.locations || {},
    items: STATIC_ITEMS,
    technologies: STATIC_TECH,
    special_combos_count: combosJson.combos?.length || 0,
    special_combos: special,
  };
}

function captureEstimate(targetName) {
  const target = findPal(appData.pals, targetName);
  if (!target) return null;
  let estimate = Math.round((1600 - target.power) / 16);
  estimate = Math.max(4, Math.min(95, estimate));
  const easier = [...appData.pals].sort((a, b) => a.power - b.power).slice(0, 5);
  return { target: target.name, estimate_percent: estimate, easier_targets: easier };
}`
);

s = s.replace(
  `  locationsLoading = apiFetch("/api/locations").then((data) => {
    appData.pal_locations = data;
    locationsLoaded = true;
  });`,
  `  locationsLoading = fetch("data/pal_locations.json")
    .then((r) => r.json())
    .then((data) => {
      appData.pal_locations = data.locations || {};
      locationsLoaded = true;
    });`
);

s = s.replace(
  `    const capture = await apiFetch(\`/api/capture/\${encodeURIComponent(targetName)}\`);`,
  `    const capture = captureEstimate(targetName);
    if (!capture) {
      databasePanelBody.innerHTML = "<p class=\\"muted\\">Unknown Pal.</p>";
      return;
    }`
);

s = s.replace(
  `function resolveViewFromPath(pathname) {
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
}`,
  `function resolveViewFromPath() {
  const hash = (location.hash || "#breeding").replace(/^#/, "");
  if (viewMeta[hash]) return hash;
  return "breeding";
}`
);

s = s.replace(
  `    const calc = await apiFetch("/api/calculate", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ parent_a: parentA, parent_b: parentB })
    });`,
  `    const calc = calculateChild(appData.pals, appData.special_combos, parentA, parentB);
    if (!calc) {
      showResultError("Could not find those Pals.");
      return;
    }`
);

s = s.replace(
  `    const pairs = await apiFetch(\`/api/combinations/\${encodeURIComponent(targetName)}\`);`,
  `    const pairs = combinationsForTarget(appData.pals, appData.special_combos, targetName);`
);

s = s.replace(
  `  appData = await apiFetch("/api/bootstrap");`,
  `  appData = await loadStaticData();`
);

s = s.replace(
  `  const initialView = resolveViewFromPath(globalThis.location.pathname);`,
  `  const initialView = resolveViewFromPath();`
);

s = s.replace(
  `globalThis.addEventListener("popstate", () => {
  const view = resolveViewFromPath(globalThis.location.pathname);
  renderDatabasePanel(view);
  focusViewCard(view);
});`,
  `globalThis.addEventListener("hashchange", () => {
  const view = resolveViewFromPath();
  renderDatabasePanel(view);
  focusViewCard(view);
});`
);

s = s.replace(
  `navButtons.forEach((button) => {
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
});`,
  `navButtons.forEach((button) => {
  button.addEventListener("click", (event) => {
    event.preventDefault();
    const view = button.dataset.view;
    location.hash = view;
    renderDatabasePanel(view);
    focusViewCard(view);
  });
});`
);

fs.writeFileSync(path, s);
console.log("patched static-site/app.js");
