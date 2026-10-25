const { chromium } = require('playwright');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const base = process.env.BROWSER_BASE_URL || 'http://127.0.0.1:3000';
(async () => {
  const browser = await chromium.launch({ executablePath: process.env.BROWSER_EXECUTABLE_PATH || undefined, args: ['--no-sandbox'] });
  fs.mkdirSync('test-results', { recursive: true });
  try {
    for (const [name, viewport] of [['desktop', { width: 1440, height: 1000 }], ['mobile', { width: 390, height: 844 }]]) {
      const context = await browser.newContext({ viewport, reducedMotion: 'reduce' });
      const page = await context.newPage();
      const errors = [];
      page.on('pageerror', error => errors.push(error.message));
      await page.goto(`${base}/palworld-breeding-calculator`);
      await page.locator('#dataNote').filter({ hasText: '299 species' }).waitFor();
      const choose = async (a,b,ga='',gb='') => {
        await page.locator('#parentA').selectOption(a);
        await page.locator('#parentB').selectOption(b);
        await page.locator('#parentAGender').selectOption(ga);
        await page.locator('#parentBGender').selectOption(gb);
        await page.getByRole('button', { name: 'Calculate Child', exact: true }).click();
      };
      await choose('Jetragon', 'Jetragon');
      await page.locator('#result .arena-name').last().filter({ hasText: /^Jetragon$/ }).waitFor();
      await choose('Katress', 'Wixen');
      await page.locator('#result .result-error').filter({ hasText: 'gender-specific' }).waitFor();
      await choose('Katress', 'Wixen', 'female', 'male');
      await page.locator('#result .arena-name').last().filter({ hasText: /^Katress Ignis$/ }).waitFor();
      await page.getByRole('button', { name: 'Swap Parents', exact: true }).click();
      await page.locator('#result .arena-name').last().filter({ hasText: /^Katress Ignis$/ }).waitFor();
      await choose('Katress', 'Wixen', 'male', 'female');
      await page.locator('#result .arena-name').last().filter({ hasText: /^Wixen Noct$/ }).waitFor();
      await choose('Kitsun Noct', 'Eidrolon Ignis');
      await page.locator('#result .arena-name').last().filter({ hasText: /^Anubis$/ }).waitFor();
      await page.locator('#targetChild').selectOption('Jetragon');
      await page.getByRole('button', { name: 'Find Combinations', exact: true }).click();
      await page.locator('#combos .target-row').filter({ hasText: 'Found 1 combinations' }).waitFor();
      await page.locator('#chainOwned').selectOption('Kitsun Noct');
      await page.locator('#chainAdditional').selectOption(['Eidrolon Ignis']);
      await page.locator('#chainGoal').selectOption('Anubis');
      await page.getByRole('button', { name: 'Find Breeding Chain', exact: true }).click();
      await page.locator('#chainResult .chain-step').filter({ hasText: 'Anubis' }).waitFor();
      await page.locator('#arenaCard').scrollIntoViewIfNeeded();
      assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth + 1), `${name}: horizontal overflow`);
      await page.screenshot({ path: `test-results/${name}.png`, fullPage: false });
      await page.locator('#themeToggleBtn').click();
      assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth + 1));
      assert.deepEqual(errors, [], `${name}: browser errors`);
      console.log(`${name}: parent results, gender outcomes, swap, reverse lookup, owned-roster chain, theme, and overflow checks passed`);
      await context.close();
    }
  } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
