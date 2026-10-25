const assert = require('node:assert/strict');
const fs = require('node:fs');
const vm = require('node:vm');
const root = require('node:path').resolve(__dirname, '..');
const stub = { addEventListener() {}, dataset: {}, value: '', querySelectorAll() { return []; } };
const sandbox = { console, document: { body: { dataset: {} }, getElementById: () => stub, querySelectorAll: () => [] }, setTimeout, clearTimeout, URLSearchParams };
vm.createContext(sandbox);
const source = fs.readFileSync(`${root}/app.js`, 'utf8').split('calculateBtn.addEventListener')[0];
vm.runInContext(source, sandbox);
const fixture = {
  pals: JSON.parse(fs.readFileSync(`${root}/data/pals.json`)).pals,
  special_combos: Object.fromEntries(JSON.parse(fs.readFileSync(`${root}/data/special_combos.json`)).combos.map(c => [[c.parent_a,c.parent_b].sort().join('|'),c.child])),
  breeding_table: JSON.parse(fs.readFileSync(`${root}/data/breeding_table.json`))
};
sandbox.fixture = fixture;
vm.runInContext('appData = fixture; buildPalIndexes();', sandbox);
const calculate = (...args) => { sandbox.args = args; return vm.runInContext('calculateChildLocal(...args)', sandbox); };
let checked = 0;
for (let a=0; a<fixture.pals.length; a++) {
  for (let b=a; b<fixture.pals.length; b++) {
    const offset = a*fixture.pals.length-a*(a-1)/2+b-a;
    const child = fixture.breeding_table.children[offset];
    const first = fixture.pals[a].name, second = fixture.pals[b].name;
    if (child < 0) {
      assert.ok(calculate(first,second).error);
      continue;
    }
    assert.equal(calculate(first,second).child.name, fixture.pals[child].name);
    assert.equal(calculate(second,first).child.name, fixture.pals[child].name);
    checked++;
  }
}
assert.equal(calculate('Jetragon','Jetragon').child.name,'Jetragon');
assert.equal(calculate('Penking','Bushi').child.name,'Sibelyx');
assert.equal(calculate('Kitsun Noct','Eidrolon Ignis').child.name,'Anubis');
assert.equal(calculate('Katress','Wixen','female','male').child.name,'Katress Ignis');
assert.equal(calculate('Wixen','Katress','female','male').child.name,'Wixen Noct');
assert.equal(calculate('Katress','Wixen','male','female').child.name,'Wixen Noct');
assert.equal(calculate('Wixen','Katress','male','female').child.name,'Katress Ignis');
assert.equal(calculate('missing','Lamball'),null);
console.log(`Frontend replay passed: ${checked.toLocaleString()} ungendered pairs in both orders, same-species, golden Anubis pair, and all gender-specific orientations.`);
