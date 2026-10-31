//! Concise route headings and visitor-facing introductions.
//! Meta descriptions live on each page and stay about 150–160 characters.

pub const HOME_H1: &str = "Palworld Breeding Calculator";

pub const HOME_DESC: &str = "Choose two parents or search for a target species. Results use a pinned PalCalc community dataset with 299 species and gender-specific outcomes. Check your game version before committing resources.";

pub const BREEDING_H1: &str = "Choose parents. Find their child.";

pub const BREEDING_DESC: &str = "Select parent species and calculate their offspring. Same-species parents are supported. Katress + Wixen requires both parent genders because reversing them changes the child.";

pub const PALS_H1: &str = "Pal breeding database";

pub const PALS_DESC: &str = "Browse the species in the imported dataset. Open a profile to inspect parent combinations and linked tools. Breeding power is a breeding statistic, not combat strength or capture probability.";

pub const MAP_H1: &str = "Pal location references";

pub const MAP_DESC: &str = "Use these location notes as a starting point and verify them in your game. The location dataset has separate coverage from the breeding table; an absent location does not mean a Pal cannot be obtained.";

pub const ITEMS_H1: &str = "Breeding item references";

pub const ITEMS_DESC: &str = "Review the included item notes while planning a breeding session. Item availability and unlock requirements can change with game updates and world settings.";

pub const TECH_H1: &str = "Technology references";

pub const TECH_DESC: &str = "Use these progression notes alongside the technology menu in your current game. The breeding table does not verify technology levels or material requirements.";

pub const CAPTURE_H1: &str = "Breeding-power difficulty heuristic";

pub const CAPTURE_DESC: &str = "This score uses breeding power only. It is not an in-game capture probability and does not model sphere type, health, level, status effects, or world settings.";

pub const KEYWORD_BREED_H1: &str = "Palworld parent-pair calculator";

pub const KEYWORD_BREED_DESC: &str = "Test a parent pair, including same-species and gender-specific combinations. Reverse lookup lists the pairs in the pinned source snapshot for your chosen child.";

pub const KEYWORD_COMBOS_H1: &str = "Find parent combinations";

pub const KEYWORD_COMBOS_DESC: &str = "Select a target species to list supported parent pairs. Gender requirements appear beside the affected pairs. Same-species-only targets require acquiring that species first.";

pub const KEYWORD_CAPTURE_H1: &str = "Capture planning: know the limits";

pub const KEYWORD_CAPTURE_DESC: &str = "Use the breeding-power score as an explicitly limited comparison. Check actual capture chances in the game; this tool does not calculate them.";

pub const CHAIN_H1: &str = "Plan an executable breeding route";

pub const CHAIN_DESC: &str = "Select the species you own and your goal. The bounded planner uses only available parents, assumes suitable genders, and may not find every route. Acquire missing partners through reverse lookup first.";

/// Sitewide footer: internal links for crawl paths and user navigation (injected on all page types).
pub const SITE_FOOTER_HTML: &str = r#"<footer class="site-footer" role="contentinfo">
  <div class="site-footer-inner">
    <div class="site-footer-col">
      <p class="site-footer-title">Palworld Breeding Calculator</p>
      <p class="site-footer-muted">Species predictions, reverse lookup, and owned-roster route planning.</p>
    </div>
    <nav class="site-footer-col" aria-label="Tools">
      <p class="site-footer-heading">Tools</p>
      <ul>
        <li><a href="/">Breeding Calculator</a></li>
        <li><a href="/palworld-breeding-combinations">Breeding Combinations</a></li>
        <li><a href="/palworld-chain-breeding">Chain Breeding</a></li>
        <li><a href="/palworld-capture-rate-calculator">Capture Rate</a></li>
        <li><a href="/pals">Pals Database</a></li>
        <li><a href="/map">Map Locations</a></li>
      </ul>
    </nav>
    <nav class="site-footer-col" aria-label="Guides">
      <p class="site-footer-heading">Guides</p>
      <ul>
        <li><a href="/how-to-breed/anubis">How to Breed Anubis</a></li>
        <li><a href="/fastest-anubis-breed">Fastest Anubis Breed</a></li>
        <li><a href="/legendary-breeding">Legendary Breeding</a></li>
        <li><a href="/guides/best-breeding-combos">Best Breeding Combos</a></li>
        <li><a href="/guides/breeding-not-working">Breeding Not Working</a></li>
        <li><a href="/egg-incubation-guide">Egg Incubation</a></li>
        <li><a href="/guides/breeding-statistics">Breeding Statistics</a></li>
        <li><a href="/guides/breeding-farm-setup">Breeding Farm Guide</a></li>
      </ul>
    </nav>
    <nav class="site-footer-col" aria-label="Popular Pals">
      <p class="site-footer-heading">Popular Pals</p>
      <ul>
        <li><a href="/pal/anubis">Anubis</a></li>
        <li><a href="/pal/jetragon">Jetragon</a></li>
        <li><a href="/pal/frostallion">Frostallion</a></li>
        <li><a href="/pal/lyleen">Lyleen</a></li>
        <li><a href="/pal-pages">All Pal Pages</a></li>
        <li><a href="/combo-pages">Combo Pages</a></li>
      </ul>
    </nav>
    <nav class="site-footer-col" aria-label="Site">
      <p class="site-footer-heading">Site</p>
      <ul>
        <li><a href="/about">About &amp; Data Sources</a></li>
        <li><a href="/about#contact">Contact</a></li>
        <li><a href="/privacy">Privacy Policy</a></li>
      </ul>
    </nav>
  </div>
  <p class="site-footer-copy">Palworld Breeding Calculator — unofficial fan tool. Not affiliated with Pocketpair. Data derived from <a href="https://github.com/tylercamp/palcalc/tree/66f18a000e2107640b219562437357ced5aff7eb">PalCalc</a>, MIT license; imported 25 October 2026 and cross-checked against Palworld 1.0.5 (<a href="https://palsphere.app">PalSphere</a> extract).</p>
</footer>"#;

/// Guide meta descriptions, about 150–160 characters, unique per page.
pub const GUIDE_HOW_TO_BREED_ANUBIS: &str = "Learn how to breed Anubis in Palworld: unlock the Breeding Farm, keep cake stocked, and compare parent paths in the calculator before hatching on your save.";

pub const GUIDE_BEST_COMBOS: &str = "Choose better Palworld breeding combos by checking special pairs first, then easier parents, and confirm the child in the calculator before using cake.";

pub const GUIDE_CAPTURE_RATE: &str = "Plan Palworld captures for breeding with power as a difficulty guide, farm easier parents first, and delay legendaries you cannot catch yet on your save.";

pub const GUIDE_FASTEST_ANUBIS: &str = "Get Anubis faster in Palworld by choosing parent pairs with fewer catches and fewer eggs, then confirm that route in the breeding calculator on your save.";

pub const GUIDE_LEGENDARY: &str = "Check legendary Pal breeding restrictions for Jetragon, Frostallion, and Necromus. Use reverse lookup to confirm same-species pairs before planning any eggs.";

pub const GUIDE_EARLY_GAME: &str = "Best early-game Palworld breeding combos use easy parents like Lamball, Cattiva, and Foxparks. Test every pair in the calculator before any legendary.";

pub const GUIDE_EGG_INCUBATION: &str = "Palworld egg incubation guide for the incubator unlock, cake in the Breeding Farm, and keeping eggs moving after both parents are assigned on your save.";

pub const GUIDE_MINING: &str = "Breed mining Pals such as Digtoise in Palworld by listing parent pairs, catching the easier parent first, and checking the calculator result on your save.";

pub const GUIDE_FLYING_MOUNT: &str = "Plan flying mount breeding with supported parent pairs. Check Jetragon same-species restrictions and choose an available target before spending cake in game.";

pub const GUIDE_ANUBIS_VS_LYLEEN: &str = "Compare Anubis and Lyleen in Palworld by breeding time, parent difficulty, and base role so you know which of the two to breed first on your current save.";

pub const GUIDE_BREEDING_STATS: &str = "Palworld breeding statistics from 44,850 parent pairs: the most common results, Pals that only breed from their own species, and how many routes lead to Anubis.";

pub const GUIDE_BREEDING_FARM: &str = "Palworld Breeding Farm guide: setup checklist, how to pick parents, same-species rules, and all 81 special combinations by variant (Palworld 1.0.5 data).";

pub const GUIDE_BREEDING_NOT_WORKING: &str = "If Palworld breeding fails, check the Breeding Farm chest for cake, suitable male and female parents, and supported pairs before changing your setup in game.";



pub const PAGE_ABOUT: &str = "Who runs this unofficial Palworld breeding calculator, where its breeding data comes from, how results are checked against game 1.0.5, and how to report errors.";

pub const PAGE_PRIVACY: &str = "Privacy policy for the Palworld Breeding Calculator: no accounts, no ads, no tracking cookies. Only a theme setting is saved in your browser, plus host logs.";

pub const ABOUT_BODY_HTML: &str = r#"<p>The Palworld Breeding Calculator is a free, unofficial fan tool for planning Pal breeding. It predicts the child of any two parents, lists every parent pair that produces a chosen Pal, and plans multi-step breeding chains from the Pals you already own. It is not affiliated with or endorsed by Pocketpair.</p>
<h2>Where the data comes from</h2>
<p>Breeding results come from the open-source <a href="https://github.com/tylercamp/palcalc">PalCalc</a> project (MIT license), imported on 25 October 2026 and cross-checked against a Palworld 1.0.5 data extract. The table covers 299 Pals, 44,850 unordered parent pairs and 81 special combinations that override the normal breeding-power average. Gender-specific and same-species-only results are handled separately.</p>
<h2>How results are checked</h2>
<p>Automated tests run on every change to confirm known special combinations, same-species legendaries and a sample of power-average results. When a game patch changes breeding, the data is re-imported and the version and date shown on each page are updated. Capture difficulty figures are estimates from breeding power, not the in-game capture formula.</p>
<h2 id="contact">Contact and corrections</h2>
<p>Found a wrong result, a missing Pal or a broken page? Open an issue on <a href="https://github.com/harisarshad29/palworld-breeding-calculator/issues">GitHub</a> with the two parents and the child you got in game. Corrections are checked against the data source and usually fixed in the next update.</p>"#;

pub const PRIVACY_BODY_HTML: &str = r#"<p>Last updated: 31 October 2026.</p>
<h2>What we collect</h2>
<p>This site has no accounts, no sign-up, no advertising and no analytics or tracking cookies. The calculator runs in your browser and the Pals you pick are not sent to us.</p>
<h2>What is stored on your device</h2>
<p>Your light or dark theme choice is saved in your browser's local storage so it stays the same on your next visit. You can clear it at any time through your browser settings.</p>
<h2>Hosting logs</h2>
<p>Like any website, the hosting provider (Netlify) records standard request logs such as IP address, browser type and the page requested, to deliver the site and protect it from abuse. We do not combine these logs with any other data.</p>
<h2>Third-party links</h2>
<p>Some pages link to other sites, such as the PalCalc project on GitHub. Their own privacy policies apply once you leave this site.</p>
<h2>Questions</h2>
<p>For privacy questions, see the <a href="/about#contact">contact section on the About page</a>.</p>"#;
