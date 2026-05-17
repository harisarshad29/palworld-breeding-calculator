export function comboKey(a, b) {
  return a <= b ? `${a}|${b}` : `${b}|${a}`;
}

export function findPal(pals, name) {
  const q = String(name || "").trim();
  if (!q) return null;
  const exact = pals.find((p) => p.name === q);
  if (exact) return exact;
  const lower = q.toLowerCase();
  return pals.find((p) => p.name.toLowerCase() === lower) || null;
}

export function buildComboMap(entries) {
  const map = {};
  for (const row of entries || []) {
    map[comboKey(row.parent_a, row.parent_b)] = row.child;
  }
  return map;
}

export function calculateChild(pals, specialCombos, parentA, parentB) {
  const first = findPal(pals, parentA);
  const second = findPal(pals, parentB);
  if (!first || !second) {
    return null;
  }

  const key = comboKey(first.name, second.name);
  if (specialCombos[key]) {
    const child = findPal(pals, specialCombos[key]);
    if (child) {
      return {
        child,
        method: "Special combination",
        distance: null,
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

export function combinationsForTarget(pals, specialCombos, targetName) {
  const canonical = findPal(pals, targetName);
  if (!canonical) return [];

  const results = [];
  for (let i = 0; i < pals.length; i += 1) {
    for (let j = i; j < pals.length; j += 1) {
      const calc = calculateChild(pals, specialCombos, pals[i].name, pals[j].name);
      if (calc && calc.child.name === canonical.name) {
        results.push({
          a: pals[i].name,
          b: pals[j].name,
          method: calc.method,
        });
      }
    }
  }
  return results;
}
