(function () {
  const parentA = document.getElementById("pbc-parent-a");
  const parentB = document.getElementById("pbc-parent-b");
  const target = document.getElementById("pbc-target");
  const result = document.getElementById("pbc-result");
  const combos = document.getElementById("pbc-combos");
  const calcBtn = document.getElementById("pbc-calc-btn");
  const swapBtn = document.getElementById("pbc-swap-btn");
  const combosBtn = document.getElementById("pbc-combos-btn");

  if (!parentA || !window.pbcConfig) {
    return;
  }

  let pals = [];
  let specialCombos = {};

  function comboKey(a, b) {
    return a <= b ? `${a}|${b}` : `${b}|${a}`;
  }

  function findPal(name) {
    const q = String(name || "").trim();
    if (!q) return null;
    const exact = pals.find((p) => p.name === q);
    if (exact) return exact;
    const lower = q.toLowerCase();
    return pals.find((p) => p.name.toLowerCase() === lower) || null;
  }

  function buildComboMap(entries) {
    const map = {};
    for (const row of entries || []) {
      map[comboKey(row.parent_a, row.parent_b)] = row.child;
    }
    return map;
  }

  function calculateChild(nameA, nameB) {
    const first = findPal(nameA);
    const second = findPal(nameB);
    if (!first || !second) {
      return { error: "Invalid parent names." };
    }

    const key = comboKey(first.name, second.name);
    if (specialCombos[key]) {
      const child = findPal(specialCombos[key]);
      if (child) {
        return {
          child,
          method: "Special combination",
        };
      }
    }

    const targetPower = Math.floor((first.power + second.power) / 2);
    let nearest = pals[0];
    let bestDistance = Math.abs(nearest.power - targetPower);
    for (const pal of pals) {
      const distance = Math.abs(pal.power - targetPower);
      if (distance < bestDistance) {
        bestDistance = distance;
        nearest = pal;
      }
    }

    return {
      child: nearest,
      method: `Power average (${targetPower})`,
      distance: bestDistance,
    };
  }

  function combinationsForTarget(targetName) {
    const canonical = findPal(targetName);
    if (!canonical) return [];
    const out = [];
    for (let i = 0; i < pals.length; i += 1) {
      for (let j = i; j < pals.length; j += 1) {
        const calc = calculateChild(pals[i].name, pals[j].name);
        if (calc.child && calc.child.name === canonical.name) {
          out.push({
            a: pals[i].name,
            b: pals[j].name,
            method: calc.method,
          });
        }
      }
    }
    return out;
  }

  function fillSelect(select) {
    select.innerHTML = "";
    for (const pal of pals) {
      const opt = document.createElement("option");
      opt.value = pal.name;
      opt.textContent = `${pal.name} (Power ${pal.power})`;
      select.appendChild(opt);
    }
  }

  function showResult() {
    const out = calculateChild(parentA.value, parentB.value);
    if (out.error) {
      result.innerHTML = `<p class="pbc-error">${out.error}</p>`;
      return;
    }
    const dist =
      out.distance != null
        ? `<p class="pbc-muted">Distance from average: ${out.distance}</p>`
        : "";
    result.innerHTML = `
      <h3>Result</h3>
      <p><strong>Child:</strong> ${out.child.name} (Power ${out.child.power})</p>
      <p><strong>Method:</strong> ${out.method}</p>
      ${dist}
    `;
  }

  function showCombos() {
    combos.innerHTML = "<p class=\"pbc-muted\">Searching combinations…</p>";
    setTimeout(() => {
      const list = combinationsForTarget(target.value);
      if (!list.length) {
        combos.innerHTML = "<p class=\"pbc-muted\">No combinations found.</p>";
        return;
      }
      const items = list
        .slice(0, 80)
        .map(
          (row) =>
            `<li><strong>${row.a}</strong> + <strong>${row.b}</strong> <span class="pbc-muted">(${row.method})</span></li>`
        )
        .join("");
      const more =
        list.length > 80
          ? `<p class="pbc-muted">Showing 80 of ${list.length} combinations.</p>`
          : "";
      combos.innerHTML = `
        <h3>Combinations for ${target.value}</h3>
        <ul class="pbc-combo-list">${items}</ul>
        ${more}
      `;
    }, 10);
  }

  async function init() {
    try {
      const [palsRes, combosRes] = await Promise.all([
        fetch(pbcConfig.palsUrl),
        fetch(pbcConfig.combosUrl),
      ]);
      const palsJson = await palsRes.json();
      const combosJson = await combosRes.json();
      pals = palsJson.pals || [];
      pals.sort((a, b) => a.name.localeCompare(b.name));
      specialCombos = buildComboMap(combosJson.combos);
      fillSelect(parentA);
      fillSelect(parentB);
      fillSelect(target);
      const anubis = findPal("Anubis");
      const jetragon = findPal("Jetragon");
      const frost = findPal("Frostallion");
      if (anubis) parentA.value = anubis.name;
      if (jetragon) parentB.value = jetragon.name;
      if (frost) target.value = frost.name;
      result.innerHTML = "<p class=\"pbc-muted\">Ready. Click Calculate child.</p>";
    } catch (e) {
      result.innerHTML = "<p class=\"pbc-error\">Could not load Pal data.</p>";
    }
  }

  calcBtn.addEventListener("click", showResult);
  swapBtn.addEventListener("click", () => {
    const tmp = parentA.value;
    parentA.value = parentB.value;
    parentB.value = tmp;
    showResult();
  });
  combosBtn.addEventListener("click", showCombos);

  init();
})();
