mod chain;
mod data;
mod seo_copy;

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use axum::http::{header, HeaderValue};
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;

#[derive(Clone)]
struct AppState {
    pals: Vec<Pal>,
    pals_by_name: HashMap<String, usize>,
    power_order: Vec<usize>,
    pal_locations: HashMap<String, data::PalLocation>,
    items: Vec<ItemData>,
    technologies: Vec<TechnologyData>,
    special_combos: HashMap<String, String>,
    reverse_combos: Arc<HashMap<String, Vec<PairResult>>>,
    base_url: String,
    index_template: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Pal {
    name: String,
    power: i32,
}

#[derive(Debug, Clone, Serialize)]
struct ItemData {
    item: String,
    source: String,
    notes: String,
}

#[derive(Debug, Clone, Serialize)]
struct TechnologyData {
    level: i32,
    name: String,
    cost: String,
}

#[derive(Debug, Serialize)]
struct BootstrapResponse {
    pals: Vec<Pal>,
    items: Vec<ItemData>,
    technologies: Vec<TechnologyData>,
    special_combos_count: usize,
    special_combos: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct CalculateRequest {
    parent_a: String,
    parent_b: String,
}

#[derive(Debug, Serialize)]
struct CalculateResponse {
    child: Pal,
    method: String,
    distance: Option<i32>,
}

#[derive(Debug, Clone, Serialize)]
struct PairResult {
    a: String,
    b: String,
    method: String,
}

#[derive(Debug, Serialize)]
struct CaptureResponse {
    target: String,
    estimate_percent: i32,
    easier_targets: Vec<Pal>,
}

/// Primary calculator URL for consolidations and internal links.
const PRIMARY_CALCULATOR_PATH: &str = "/palworld-breeding-calculator";

/// High-value breed targets for hub pages (how-to-breed + combos index).
const PRIORITY_BREED_TARGETS: &[&str] = &[
    "Anubis", "Jetragon", "Frostallion", "Necromus", "Paladius", "Lyleen", "Blazamut", "Suzaku",
    "Jormuntide", "Shadowbeak", "Bellanoir", "Relaxaurus", "Penking", "Digtoise", "Astegon",
    "Chikipi", "Foxparks", "Lamball", "Cattiva", "Grizzbolt", "Orserk", "Helzephyr", "Kingpaca",
];

/// High-value parent pairs for combo page discovery (beyond special_combos.json).
const LEGENDARY_COMBO_SEEDS: &[(&str, &str)] = &[
    ("Anubis", "Jetragon"),
    ("Necromus", "Paladius"),
    ("Blazamut", "Suzaku"),
    ("Grizzbolt", "Lyleen"),
    ("Relaxaurus", "Mammorest"),
    ("Penking", "Bushi"),
    ("Frostallion", "Frostallion Noct"),
    ("Lamball", "Cattiva"),
    ("Foxparks", "Rooby"),
    ("Mossanda", "Grizzbolt"),
    ("Kitsun", "Astegon"),
    ("Vanwyrm", "Anubis"),
    ("Surfent", "Dumud"),
    ("Incineram", "Maraith"),
    ("Jolthog", "Pengullet"),
    ("Suzaku", "Jormuntide"),
];

#[derive(Clone, Copy)]
struct SeoPage {
    path: &'static str,
    title: &'static str,
    meta_description: &'static str,
    h1: &'static str,
    badge: &'static str,
    /// 50–60 words shown under the H1 (route subtitle).
    h1_characteristics: &'static str,
    /// 150 words shown in the About section for this route.
    page_description: &'static str,
    /// When set, canonical points here instead of `path` (duplicate route consolidation).
    canonical_path: Option<&'static str>,
}

#[derive(Clone, Copy)]
struct GuidePage {
    path: &'static str,
    title: &'static str,
    description: &'static str,
    heading: &'static str,
    body_html: &'static str,
}

const SEO_PAGES: [SeoPage; 11] = [
    SeoPage {
        path: "/",
        title: "Palworld Breeding Calculator – Free Tool, Combos & Reverse Lookup",
        meta_description: "Free Palworld breeding calculator with parent pair testing, reverse lookup for any Pal, combo pages, and guides for Anubis, Jetragon, and legendaries.",
        h1: "Palworld Breeding Calculator",
        badge: "Home",
        h1_characteristics: seo_copy::HOME_H1,
        page_description: seo_copy::HOME_DESC,
        canonical_path: None,
    },
    SeoPage {
        path: "/breeding-calculator",
        title: "Palworld Breeding Calculator – Best Combos, Reverse Lookup & Egg Guide",
        meta_description: "Use the ultimate Palworld Breeding Calculator to find breeding combinations, reverse breeding results, egg routes, and legendary pal combos instantly.",
        h1: "Palworld Parent Combo Calculator",
        badge: "Breeding Calculator",
        h1_characteristics: seo_copy::BREEDING_H1,
        page_description: seo_copy::BREEDING_DESC,
        canonical_path: Some(PRIMARY_CALCULATOR_PATH),
    },
    SeoPage {
        path: "/pals",
        title: "Palworld Pals Database - Powers and References",
        meta_description: "Browse Palworld Pals with breeding power values, quick picks, and links to combo and capture planning tools.",
        h1: "Palworld Pals Database",
        badge: "Pals Database",
        h1_characteristics: seo_copy::PALS_H1,
        page_description: seo_copy::PALS_DESC,
        canonical_path: None,
    },
    SeoPage {
        path: "/map",
        title: "Palworld Map Reference - Pal Locations",
        meta_description: "Find Palworld spawn regions and coordinates for key Pals to shorten farming routes before breeding setup.",
        h1: "Palworld Map & Pal Locations",
        badge: "Map Reference",
        h1_characteristics: seo_copy::MAP_H1,
        page_description: seo_copy::MAP_DESC,
        canonical_path: None,
    },
    SeoPage {
        path: "/items",
        title: "Palworld Items Database - Sources and Drops",
        meta_description: "Explore Palworld item sources, drop notes, and farming tips that support breeding, crafting, and base progression.",
        h1: "Palworld Items Database",
        badge: "Items Database",
        h1_characteristics: seo_copy::ITEMS_H1,
        page_description: seo_copy::ITEMS_DESC,
        canonical_path: None,
    },
    SeoPage {
        path: "/technology",
        title: "Palworld Technology Tree Guide - Key Milestones",
        meta_description: "Track Palworld technology unlock levels for breeding, incubation, crafting, and production milestones.",
        h1: "Palworld Technology Milestones",
        badge: "Technology Guide",
        h1_characteristics: seo_copy::TECH_H1,
        page_description: seo_copy::TECH_DESC,
        canonical_path: None,
    },
    SeoPage {
        path: "/capture-rate",
        title: "Palworld Capture Rate Guide - Difficulty Estimates",
        meta_description: "Estimate Palworld capture difficulty from breeding power and pick easier parent targets before expensive breeding loops.",
        h1: "Palworld Capture Rate Estimates",
        badge: "Capture Rate",
        h1_characteristics: seo_copy::CAPTURE_H1,
        page_description: seo_copy::CAPTURE_DESC,
        canonical_path: None,
    },
    SeoPage {
        path: "/palworld-breeding-calculator",
        title: "Palworld Breeding Calculator – Best Combos, Reverse Lookup & Egg Guide",
        meta_description: "Use the ultimate Palworld Breeding Calculator to find breeding combinations, reverse breeding results, passive skills, egg chains, and legendary pal combos instantly.",
        h1: "Palworld Breeding Calculator Online",
        badge: "Online Calculator",
        h1_characteristics: seo_copy::KEYWORD_BREED_H1,
        page_description: seo_copy::KEYWORD_BREED_DESC,
        canonical_path: None,
    },
    SeoPage {
        path: "/palworld-breeding-combinations",
        title: "Palworld Breeding Combinations - Parent Pair Database",
        meta_description: "Explore Palworld breeding combinations, special pair outcomes, and parent routes for rare and meta Pals.",
        h1: "Palworld Breeding Combinations",
        badge: "Combinations",
        h1_characteristics: seo_copy::KEYWORD_COMBOS_H1,
        page_description: seo_copy::KEYWORD_COMBOS_DESC,
        canonical_path: None,
    },
    SeoPage {
        path: "/palworld-capture-rate-calculator",
        title: "Palworld Capture Rate Calculator - Catch Difficulty Estimator",
        meta_description: "Palworld capture rate calculator with difficulty estimates and easier parent suggestions for faster breeding setup.",
        h1: "Palworld Capture Rate Calculator",
        badge: "Capture Calculator",
        h1_characteristics: seo_copy::KEYWORD_CAPTURE_H1,
        page_description: seo_copy::KEYWORD_CAPTURE_DESC,
        canonical_path: None,
    },
    SeoPage {
        path: "/palworld-chain-breeding",
        title: "Palworld Chain Breeding Calculator - Shortest Path to Any Pal",
        meta_description: "Find the shortest Palworld breeding chain from a Pal you own to any target. Step-by-step parent pairs, special combos, and links to verify each egg.",
        h1: "Palworld Chain Breeding Path Finder",
        badge: "Chain Breeder",
        h1_characteristics: seo_copy::CHAIN_H1,
        page_description: seo_copy::CHAIN_DESC,
        canonical_path: None,
    },
];

const GUIDE_PAGES: [GuidePage; 11] = [
    GuidePage {
        path: "/guides/how-to-breed-anubis",
        title: "How to Breed Anubis in Palworld - Fast Parent Paths",
        description: "Learn practical ways to breed Anubis in Palworld using parent path strategy and breeding power logic.",
        heading: "How to Breed Anubis in Palworld",
        body_html: "<p>Anubis is one of the most valuable mid-to-late game Pals because of handiwork and combat utility. This guide walks you through the fastest routes using our <a href=\"/palworld-breeding-calculator\">breeding calculator</a> and live combo tables.</p><h2>Step 1: Unlock breeding</h2><p>Research the Breeding Farm and Egg Incubator, build both at your base, and keep cake in the feed box before you start.</p><h2>Step 2: Use reverse lookup</h2><p>Open the calculator, set <strong>Anubis</strong> as the target child, and compare every listed parent pair. Special combinations are listed first when they exist.</p><h2>Step 3: Farm easier parents first</h2><p>Capture lower breeding-power parents before rare legendaries. Check the <a href=\"/pal/anubis\">Anubis Pal page</a> and <a href=\"/combos/anubis\">combo hub</a> for direct links.</p><h2>Step 4: Run the egg loop</h2><p>Place parents in the Breeding Farm, incubate the egg, and repeat the chain until you hatch Anubis with your desired traits.</p><p><strong>Tip:</strong> Gender does not change the predicted child in standard breeding math—focus on valid parent pairs.</p>",
    },
    GuidePage {
        path: "/guides/best-breeding-combos",
        title: "Best Palworld Breeding Combos - Practical Combo Guide",
        description: "Explore strong Palworld breeding combo strategy and how to evaluate parent pairs for better child outcomes.",
        heading: "Best Breeding Combos Strategy",
        body_html: "<p>The best Palworld breeding combo is not always the rarest pair—it is the route that gets your target child with the least farming, cake, and failed eggs. This guide explains how to rank combos using our <a href=\"/palworld-breeding-calculator\">breeding calculator</a> and combo pages.</p><h2>1. Check special combinations first</h2><p>Some parent pairs override breeding power and always produce a fixed child. These are listed as <strong>Special combination</strong> in results. Always verify these before long power-average experiments.</p><h2>2. Compare power-average fallbacks</h2><p>When no special pair exists, the child is chosen by breeding power average. Pick parent pairs with smaller distance values and easier captures.</p><h2>3. Use reverse lookup for target-first planning</h2><p>Select your goal Pal (Anubis, Jetragon, Lyleen) in the reverse calculator and compare every valid route. Open <a href=\"/combos/anubis\">Anubis combo hub</a> or the <a href=\"/pal/jetragon\">Jetragon Pal page</a> for quick links.</p><h2>4. Stack capture + map + tech</h2><p>Pair combo planning with <a href=\"/map\">map locations</a>, <a href=\"/palworld-capture-rate-calculator\">capture estimates</a>, and <a href=\"/technology\">technology milestones</a> so incubators and cake keep up with your chain.</p><h2>5. Chain breeding for legendaries</h2><p>Endgame targets often need multiple eggs. Use the <a href=\"/palworld-chain-breeding\">chain breeding tool</a> to see shortest paths from Pals you already own.</p><p><strong>Pro tip:</strong> Bookmark combo URLs you reuse and share them with co-op teammates instead of re-testing pairs every session.</p>",
    },
    GuidePage {
        path: "/guides/capture-rate-explained",
        title: "Palworld Capture Rate Explained - Better Catch Planning",
        description: "Understand capture-rate difficulty estimates in Palworld and plan better targets before breeding runs.",
        heading: "Capture Rate Planning for Breeding",
        body_html: "<p>Capture rate planning is half of successful Palworld breeding. If you cannot catch parents reliably, every combo spreadsheet fails. This guide explains how our <a href=\"/palworld-capture-rate-calculator\">capture rate calculator</a> estimates difficulty and how to act on it.</p><h2>How estimates work</h2><p>We use breeding power as a practical difficulty proxy: higher power usually means rarer spawns and tougher catches. Estimates are planning guidance—always confirm in-game after patches.</p><h2>Step-by-step capture workflow</h2><ol><li>Pick your target child in the reverse calculator.</li><li>List all parent pairs and sort by easiest parent power.</li><li>Upgrade spheres and team levels before alpha or legendary routes.</li><li>Use <a href=\"/map\">map coordinates</a> to plan one efficient farming trip.</li><li>After each capture, re-run reverse lookup—new parents unlock better pairs.</li></ol><h2>When to delay a legendary</h2><p>If capture estimates stay low and substitutes exist, breed stepping-stone children first. The <a href=\"/legendary-breeding\">legendary breeding guide</a> covers multi-step chains for Jetragon, Frostallion, and Necromus.</p><p>Combine this page with <a href=\"/guides/breeding-not-working\">breeding troubleshooting</a> if eggs stall after captures succeed.</p>",
    },
    GuidePage {
        path: "/fastest-anubis-breed",
        title: "Fastest Anubis Breed Route in Palworld - Parent Combos",
        description: "Find the fastest Anubis breeding routes in Palworld with parent pair tables, power logic, and calculator links.",
        heading: "Fastest Anubis Breed Routes",
        body_html: "<p>Anubis is one of the best Palworld breeding investments for handiwork and combat. This page focuses on speed: the fewest captures and eggs to hatch Anubis on your save.</p><h2>Step 1: Reverse lookup</h2><p>Open the <a href=\"/palworld-breeding-calculator\">breeding calculator</a>, set Anubis as target child, and export every parent pair. Open <a href=\"/combos/anubis\">Anubis combo hub</a> for bookmarkable URLs.</p><h2>Step 2: Sort by capture difficulty</h2><p>Using <a href=\"/palworld-capture-rate-calculator\">capture estimates</a>, farm the easiest parents first—even if the chain has one extra egg.</p><h2>Step 3: Special pairs win</h2><p>If any pair shows <strong>Special combination</strong>, test it before power-average alternatives. Open specific <a href=\"/combo-pages\">combo pages</a> to document results.</p><h2>Step 4: Chain tool check</h2><p>Run <a href=\"/palworld-chain-breeding?goal=Anubis\">chain breeding to Anubis</a> from Lamball or your most common Pal to see if a shorter multi-step path exists.</p><p>Full walkthrough: <a href=\"/guides/how-to-breed-anubis\">how to breed Anubis guide</a>. Compare roles in <a href=\"/anubis-vs-lyleen\">Anubis vs Lyleen</a>.</p>",
    },
    GuidePage {
        path: "/legendary-breeding",
        title: "Legendary Pal Breeding Guide - Palworld Routes",
        description: "Legendary Pal breeding routes for Jetragon, Frostallion, Necromus, Paladius, and other rare targets in Palworld.",
        heading: "Legendary Pal Breeding Guide",
        body_html: "<p>Legendary Pal breeding in Palworld is a chain project: you rarely breed Jetragon or Frostallion from two random captures in one egg. This guide outlines a repeatable legendary workflow using our calculator, chain tool, and Pal pages.</p><h2>Phase 1: Infrastructure</h2><p>Unlock Breeding Farm, Egg Incubator, and steady cake production. See the <a href=\"/egg-incubation-guide\">egg incubation guide</a> and <a href=\"/technology\">technology list</a>.</p><h2>Phase 2: Stepping-stone parents</h2><p>Farm mid-tier parents first (Anubis-tier, Penking-tier) before ultra-rare legendaries. Use <a href=\"/palworld-capture-rate-calculator\">capture planning</a> to avoid wasted trips.</p><h2>Phase 3: Chain or reverse planning</h2><p>For each legendary target, open its <a href=\"/pal/jetragon\">Pal page</a>, <a href=\"/how-to-breed/jetragon\">how-to-breed hub</a>, and run the <a href=\"/palworld-chain-breeding\">chain breeder</a> from a Pal you already own.</p><h2>Popular legendary targets</h2><ul><li><a href=\"/pal/jetragon\">Jetragon</a> — flying mount endgame</li><li><a href=\"/pal/frostallion\">Frostallion</a> — ice legendary</li><li><a href=\"/pal/necromus\">Necromus</a> / <a href=\"/pal/paladius\">Paladius</a> — duo bosses</li><li><a href=\"/pal/lyleen\">Lyleen</a> — late support breeder</li></ul><p>Verify every step on combo pages before spending cake. Legendary projects often take multiple real-world days—track pairs in notes or bookmarks.</p>",
    },
    GuidePage {
        path: "/best-early-game-breeding-combo",
        title: "Best Early Game Breeding Combos in Palworld",
        description: "Early game Palworld breeding combos using easy-to-catch parents and practical child outcomes for base progression.",
        heading: "Best Early Game Breeding Combos",
        body_html: "<p>Early game Palworld breeding should optimize for fast eggs and useful workers—not legendaries. This guide lists practical early combos and habits before you chase Anubis or Jetragon.</p><h2>Best starter parents</h2><p>Lamball, Cattiva, Chikipi, Foxparks, and Pengullet are common, easy to catch, and useful in power-average chains. Browse their <a href=\"/pal/lamball\">Pal pages</a> and test pairs in the <a href=\"/palworld-breeding-calculator\">calculator</a>.</p><h2>Early combos to try</h2><ul><li>Lamball + Cattiva — often routes toward Foxparks-tier children (verify in calculator).</li><li>Foxparks + Rooby — check special combination flags in results.</li><li>Any easy pair with low breeding power distance for quick hatch cycles.</li></ul><h2>What to avoid early</h2><p>Do not start legendary chains until you have Hyper Spheres, strong levels, and multiple incubators. Use <a href=\"/best-early-game-breeding-combo\">this page</a> with <a href=\"/guides/capture-rate-explained\">capture planning</a> when tempted to rush.</p><p>Unlock technology for Breeding Farm + incubator before batch breeding. Cake supply matters more than perfect pair theory in the first ten hours.</p>",
    },
    GuidePage {
        path: "/egg-incubation-guide",
        title: "Palworld Egg Incubation Guide - Hatch Time & Breeding Setup",
        description: "Palworld egg incubation basics: breeding farm unlock, incubator setup, and how incubation fits your combo plan.",
        heading: "Egg Incubation Guide",
        body_html: "<p>Egg incubation is the bottleneck between choosing a valid parent pair and hatching your target Pal. This Palworld egg guide covers unlock order, timing habits, and how incubation fits combo planning.</p><h2>Unlock order</h2><ol><li>Pal Spheres and basic base tech.</li><li>Egg Incubator (technology tree).</li><li>Breeding Farm — required before eggs appear from assigned parents.</li></ol><p>See exact levels on our <a href=\"/technology\">technology milestones</a> page.</p><h2>Workflow tips</h2><ul><li>Keep cake in the Breeding Farm feed box at all times.</li><li>Collect eggs immediately so parents can cycle again.</li><li>Run multiple incubators in parallel for legendary chains.</li><li>Match incubator warmth to egg type when the game requires it.</li></ul><h2>Link incubation to combos</h2><p>Before incubating, confirm the pair in the <a href=\"/palworld-breeding-calculator\">breeding calculator</a> or on a <a href=\"/combo-pages\">combo page</a>. If hatches do not match predictions, read <a href=\"/guides/breeding-not-working\">breeding not working</a>.</p><p>For long projects, use <a href=\"/palworld-chain-breeding\">chain breeding</a> so you know every intermediate egg in advance.</p>",
    },
    GuidePage {
        path: "/best-mining-pal-breeding",
        title: "Best Mining Pal Breeding - Palworld Work Pal Routes",
        description: "Find mining-focused Pal breeding routes in Palworld for base ore loops and work suitability planning.",
        heading: "Best Mining Pal Breeding",
        body_html: "<p>Mining Pals power ore bases in Palworld. Breeding them is often indirect—you breed toward breeding power tiers that trend into Digtoise, Tombat, or other work specialists. This guide connects mining goals to calculator workflows.</p><h2>Define your mining target</h2><p>Pick a end worker (Digtoise is a common goal). Open its <a href=\"/pal/digtoise\">Pal page</a> and run reverse lookup in the <a href=\"/palworld-breeding-calculator\">calculator</a>.</p><h2>Prefer easier parent routes</h2><p>Mining projects fail when players chase perfect IVs before basic capture routes work. Use <a href=\"/palworld-capture-rate-calculator\">capture estimates</a> and farm mid-tier parents first.</p><h2>Work suitability vs breeding math</h2><p>Breeding power predicts the child species—not passive skills. You may need multiple hatches to roll Mining Level upgrades. Pair breeding with item farming from our <a href=\"/items\">items database</a>.</p><p>For base-wide planning, compare <a href=\"/anubis-vs-lyleen\">Anubis vs Lyleen</a> if you split combat and support roles.</p>",
    },
    GuidePage {
        path: "/fastest-flying-mount-breeding",
        title: "Fastest Flying Mount Breeding - Jetragon & Legendary Routes",
        description: "Flying mount breeding routes in Palworld including Jetragon paths, parent combos, and capture planning.",
        heading: "Fastest Flying Mount Breeding",
        body_html: "<p>Flying mount breeding in Palworld usually means Jetragon or other late-game legendaries. Speed comes from planning captures and chains—not lucky single eggs. Use this guide with our <a href=\"/pal/jetragon\">Jetragon page</a> and <a href=\"/how-to-breed/jetragon\">how-to-breed hub</a>.</p><h2>Fastest route principles</h2><ol><li>Reverse-list every parent pair for Jetragon.</li><li>Identify special combinations vs power-average paths.</li><li>Farm the easiest high-power parents on your map first.</li><li>Run the <a href=\"/palworld-chain-breeding\">chain breeder</a> from Pals you already own.</li></ol><h2>Capture before breed</h2><p>Jetragon capture itself is endgame content. Many players breed stepping-stone Pals first. Read <a href=\"/legendary-breeding\">legendary breeding</a> and <a href=\"/guides/capture-rate-explained\">capture rate planning</a> before committing cake.</p><p>Compare alternative flying-adjacent Pals on <a href=\"/pal-pages\">all Pal pages</a> if Jetragon is months away for your save.</p>",
    },
    GuidePage {
        path: "/anubis-vs-lyleen",
        title: "Anubis vs Lyleen in Palworld - Breeding & Role Comparison",
        description: "Compare Anubis vs Lyleen breeding power, roles, and combo planning in Palworld with calculator links.",
        heading: "Anubis vs Lyleen",
        body_html: "<p><strong>Anubis</strong> and <strong>Lyleen</strong> are two of the most searched Palworld breeding targets. They serve different roster roles and breeding timelines. This comparison helps you choose which to chase first.</p><h2>Anubis overview</h2><p>Anubis offers strong handiwork and combat utility with mid-game accessibility. Breeding power is higher (easier tier) than ultra-late Pals. Start with <a href=\"/how-to-breed/anubis\">how to breed Anubis</a> and <a href=\"/fastest-anubis-breed\">fastest Anubis routes</a>.</p><h2>Lyleen overview</h2><p>Lyleen is a late support breeder with lower breeding power (rarer). Chains take longer but pay off for advanced bases. See <a href=\"/pal/lyleen\">Lyleen Pal page</a> and reverse lookup results.</p><h2>Which to breed first?</h2><p>Most players should complete Anubis routes before Lyleen unless you specifically need Lyleen passives for endgame production. Use the calculator side-by-side: set each as target child and compare parent difficulty.</p><h2>Shared tips</h2><ul><li>Both benefit from cake stock and multiple incubators.</li><li>Special combinations beat random power-average tests.</li><li>Link to <a href=\"/guides/best-breeding-combos\">best combos guide</a> for ranking methods.</li></ul>",
    },
    GuidePage {
        path: "/guides/breeding-not-working",
        title: "Palworld Breeding Not Working – Fixes & Checklist",
        description: "Fix Palworld breeding when eggs do not appear, cake runs out, or parent pairs fail. Step-by-step troubleshooting before you waste resources.",
        heading: "Breeding Not Working? Fix Checklist",
        body_html: "<p>If breeding fails in Palworld, check these items in order before changing parent pairs.</p><h2>1. Breeding Farm setup</h2><p>Confirm the Breeding Farm is built, powered, and both parent slots are filled with compatible Pals.</p><h2>2. Cake supply</h2><p>The feed box must have cake. Empty cake stops the breeding process even when parents are assigned.</p><h2>3. Valid parent pair</h2><p>Use our <a href=\"/palworld-breeding-calculator\">calculator</a> to verify the pair can produce your target child. Invalid pairs waste time.</p><h2>4. Incubator space</h2><p>Collect eggs and place them in an incubator. Full inventory or missing incubator delays progress.</p><h2>5. Pal conditions</h2><p>Sick, starving, or depressed Pals may block breeding—heal and feed parents first.</p><p>Still stuck? Compare routes on the <a href=\"/how-to-breed/anubis\">Anubis guide</a> or your target <a href=\"/pals\">Pal page</a>.</p>",
    },
];

/// Production URL for canonicals, OG tags, and sitemaps.
/// Prefer `BASE_URL` (custom domain). On Render, set `BASE_URL=https://palworld-breeding-calculator.us`.
fn resolve_base_url() -> String {
    for key in ["BASE_URL", "SITE_URL", "PUBLIC_URL"] {
        if let Ok(url) = std::env::var(key) {
            let trimmed = url.trim().trim_end_matches('/').to_string();
            if !trimmed.is_empty() {
                return trimmed;
            }
        }
    }
    if let Ok(url) = std::env::var("RENDER_EXTERNAL_URL") {
        let trimmed = url.trim().trim_end_matches('/').to_string();
        if !trimmed.is_empty() {
            eprintln!(
                "Note: using RENDER_EXTERNAL_URL for SEO URLs. Set BASE_URL to your custom domain when ready."
            );
            return trimmed;
        }
    }
    "http://127.0.0.1:3000".to_string()
}

#[tokio::main]
async fn main() {
    let index_template = std::fs::read_to_string("index.html")
        .expect("index.html must exist in project root");

    let state = build_app_state(index_template, resolve_base_url());
    let base_url_hint = state.base_url.clone();

    let assets_cache = SetResponseHeaderLayer::if_not_present(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=604800"),
    );

    let app = Router::new()
        .route("/", get(index_page))
        .route("/breeding-calculator", get(redirect_to_primary_calculator))
        .route("/how-to-breed/:name", get(how_to_breed_page))
        .route("/combos/:name", get(combos_hub_page))
        .route("/pals", get(index_pals_page))
        .route("/map", get(index_map_page))
        .route("/items", get(index_items_page))
        .route("/technology", get(index_technology_page))
        .route("/capture-rate", get(index_capture_page))
        .route("/palworld-breeding-calculator", get(index_keyword_breeding_page))
        .route("/palworld-breeding-combinations", get(index_keyword_combos_page))
        .route("/palworld-capture-rate-calculator", get(index_keyword_capture_page))
        .route("/palworld-chain-breeding", get(index_chain_breeding_page))
        .route("/pal-pages", get(pal_pages_directory))
        .route("/combo-pages", get(combo_pages_directory))
        .route("/pal/:name", get(pal_detail_page))
        .route("/combo/:parent_a/:parent_b", get(combo_detail_page))
        .route("/guides/how-to-breed-anubis", get(guide_page_handler))
        .route("/guides/best-breeding-combos", get(guide_page_handler))
        .route("/guides/capture-rate-explained", get(guide_page_handler))
        .route("/fastest-anubis-breed", get(guide_page_handler))
        .route("/legendary-breeding", get(guide_page_handler))
        .route("/best-early-game-breeding-combo", get(guide_page_handler))
        .route("/egg-incubation-guide", get(guide_page_handler))
        .route("/best-mining-pal-breeding", get(guide_page_handler))
        .route("/fastest-flying-mount-breeding", get(guide_page_handler))
        .route("/anubis-vs-lyleen", get(guide_page_handler))
        .route("/guides/breeding-not-working", get(guide_page_handler))
        .route("/breed/:name", get(breed_alias_redirect))
        .route("/robots.txt", get(robots_txt))
        .route("/sitemap.xml", get(sitemap_index))
        .route("/sitemap-pages.xml", get(sitemap_pages))
        .route("/sitemap-pals.xml", get(sitemap_pals))
        .route("/sitemap-guides.xml", get(sitemap_guides))
        .route("/sitemap-combos.xml", get(sitemap_combos))
        .route("/sitemap-hubs.xml", get(sitemap_hubs))
        .route("/api/bootstrap", get(api_bootstrap))
        .route("/api/locations", get(api_locations))
        .route("/api/calculate", post(api_calculate))
        .route("/api/combinations/:target", get(api_combinations))
        .route("/api/capture/:target", get(api_capture))
        .route("/api/chain", get(api_chain))
        .nest_service(
            "/assets",
            Router::new()
                .nest_service("/", ServeDir::new("assets"))
                .layer(assets_cache),
        )
        .with_state(state)
        .fallback_service(
            Router::new()
                .nest_service("/", ServeDir::new("."))
                .layer(SetResponseHeaderLayer::if_not_present(
                    header::CACHE_CONTROL,
                    HeaderValue::from_static("public, max-age=3600"),
                )),
        );

    // Render and other hosts require 0.0.0.0 so the port is reachable externally.
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let bind_addr = format!("{host}:{port}");
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .unwrap_or_else(|err| panic!("failed to bind to {bind_addr}: {err}"));

    println!("Server running at http://{bind_addr}");
    if base_url_hint.contains("127.0.0.1") || base_url_hint.contains("localhost") {
        eprintln!("Deploy tip: set BASE_URL=https://your-live-domain.com for correct canonicals and sitemaps.");
    }
    axum::serve(listener, app).await.expect("server error");
}

async fn api_bootstrap(State(state): State<AppState>) -> impl IntoResponse {
    Json(BootstrapResponse {
        pals: state.pals.clone(),
        items: state.items.clone(),
        technologies: state.technologies.clone(),
        special_combos_count: state.special_combos.len(),
        special_combos: state.special_combos.clone(),
    })
}

async fn api_locations(State(state): State<AppState>) -> impl IntoResponse {
    Json(state.pal_locations.clone())
}

async fn api_calculate(
    State(state): State<AppState>,
    Json(body): Json<CalculateRequest>,
) -> Result<Json<CalculateResponse>, (StatusCode, String)> {
    calculate_child(&state, &body.parent_a, &body.parent_b)
        .map(Json)
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Invalid parent names".to_string()))
}

pub(crate) fn combinations_for_target(state: &AppState, target: &str) -> Vec<PairResult> {
    let Some(pal) = find_pal_in_state(state, target) else {
        return Vec::new();
    };
    state
        .reverse_combos
        .get(&pal.name)
        .cloned()
        .unwrap_or_default()
}

fn build_app_state(index_template: String, base_url: String) -> AppState {
    let pals = data::load_pals();
    let special_combos = data::load_special_combos();
    let started = Instant::now();

    let mut pals_by_name = HashMap::with_capacity(pals.len());
    for (i, pal) in pals.iter().enumerate() {
        pals_by_name.insert(normalize_pal_name(&pal.name), i);
    }

    let mut power_order: Vec<usize> = (0..pals.len()).collect();
    power_order.sort_by_key(|&i| pals[i].power);

    let reverse_combos =
        build_reverse_index(&pals, &pals_by_name, &power_order, &special_combos);
    eprintln!(
        "Breeding index ready: {} pals, {} targets, {} ms",
        pals.len(),
        reverse_combos.len(),
        started.elapsed().as_millis()
    );

    let mut state = AppState {
        pals,
        pals_by_name,
        power_order,
        pal_locations: data::load_pal_locations(),
        items: build_items(),
        technologies: build_technologies(),
        special_combos,
        reverse_combos: Arc::new(reverse_combos),
        base_url,
        index_template: String::new(),
    };
    state.index_template = prepare_index_template(index_template, &state);
    state
}

fn prepare_index_template(mut template: String, state: &AppState) -> String {
    template = template.replace(
        "<!--PAL_PAGES_LINKS-->",
        &seo_pal_links_html(&state.pals, Some(60)),
    );
    template = template.replace(
        "<!--COMBO_PAGES_LINKS-->",
        &seo_combo_links_html(state, Some(80)),
    );
    template.replace("<!--SITE_FOOTER-->", seo_copy::SITE_FOOTER_HTML)
}

fn seo_pal_links_html(pals: &[Pal], max: Option<usize>) -> String {
    let mut sorted: Vec<&Pal> = pals.iter().collect();
    sorted.sort_by(|a, b| a.name.cmp(&b.name));
    let limit = max.unwrap_or(sorted.len());
    sorted
        .into_iter()
        .take(limit)
        .map(|pal| {
            format!(
                r#"<li><a href="/pal/{}">{}</a></li>"#,
                pal_slug(&pal.name),
                pal.name
            )
        })
        .collect::<Vec<_>>()
        .join("\n          ")
}

fn collect_combo_page_pairs(state: &AppState, max: Option<usize>) -> Vec<(String, String)> {
    let mut seen = HashSet::new();
    let mut pairs = Vec::new();

    for key in state.special_combos.keys() {
        let Some((a, b)) = key.split_once('|') else {
            continue;
        };
        let dedupe = combo_key(a, b);
        if seen.insert(dedupe) {
            pairs.push((a.to_string(), b.to_string()));
        }
    }

    for name in PRIORITY_BREED_TARGETS {
        for pair in combinations_for_target(state, name).iter().take(8) {
            let dedupe = combo_key(&pair.a, &pair.b);
            if seen.insert(dedupe) {
                pairs.push((pair.a.clone(), pair.b.clone()));
            }
        }
    }

    for pair in LEGENDARY_COMBO_SEEDS {
        let dedupe = combo_key(pair.0, pair.1);
        if seen.insert(dedupe) {
            pairs.push((pair.0.to_string(), pair.1.to_string()));
        }
    }

    if let Some(limit) = max {
        pairs.truncate(limit);
    }
    pairs
}

fn seo_combo_links_html(state: &AppState, max: Option<usize>) -> String {
    collect_combo_page_pairs(state, max)
        .into_iter()
        .map(|(a, b)| combo_pair_list_item(&a, &b))
        .collect::<Vec<_>>()
        .join("\n          ")
}

fn combo_pair_list_item(a: &str, b: &str) -> String {
    format!(
        r#"<li><a href="/combo/{}/{}">{} + {}</a></li>"#,
        pal_slug(a),
        pal_slug(b),
        a,
        b
    )
}

fn normalize_pal_name(name: &str) -> String {
    name.trim().to_lowercase()
}

fn find_pal_in_state<'a>(state: &'a AppState, name: &str) -> Option<&'a Pal> {
    let key = normalize_pal_name(name);
    state.pals_by_name.get(&key).map(|&i| &state.pals[i])
}

fn nearest_pal_index(pals: &[Pal], power_order: &[usize], target_power: i32) -> usize {
    if power_order.is_empty() {
        return 0;
    }
    let mut lo = 0usize;
    let mut hi = power_order.len();
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if pals[power_order[mid]].power < target_power {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    let right = lo.min(power_order.len() - 1);
    let left = right.saturating_sub(1);
    let r_idx = power_order[right];
    if left == right {
        return r_idx;
    }
    let l_idx = power_order[left];
    if (pals[l_idx].power - target_power).abs() <= (pals[r_idx].power - target_power).abs() {
        l_idx
    } else {
        r_idx
    }
}

fn build_reverse_index(
    pals: &[Pal],
    pals_by_name: &HashMap<String, usize>,
    power_order: &[usize],
    special_combos: &HashMap<String, String>,
) -> HashMap<String, Vec<PairResult>> {
    let mut reverse: HashMap<String, Vec<PairResult>> = HashMap::new();
    for i in 0..pals.len() {
        for j in (i + 1)..pals.len() {
            let Some((child_idx, method)) =
                child_for_pair(pals, pals_by_name, power_order, special_combos, i, j)
            else {
                continue;
            };
            let child_name = pals[child_idx].name.clone();
            reverse
                .entry(child_name)
                .or_default()
                .push(PairResult {
                    a: pals[i].name.clone(),
                    b: pals[j].name.clone(),
                    method,
                });
        }
    }
    reverse
}

fn child_for_pair(
    pals: &[Pal],
    pals_by_name: &HashMap<String, usize>,
    power_order: &[usize],
    special_combos: &HashMap<String, String>,
    i: usize,
    j: usize,
) -> Option<(usize, String)> {
    let key = combo_key(&pals[i].name, &pals[j].name);
    if let Some(child_name) = special_combos.get(&key) {
        let child_key = normalize_pal_name(child_name);
        let child_idx = *pals_by_name.get(&child_key)?;
        return Some((child_idx, "Special combination".to_string()));
    }
    let target_power = (pals[i].power + pals[j].power) / 2;
    let child_idx = nearest_pal_index(pals, power_order, target_power);
    Some((
        child_idx,
        format!("Power average ({target_power})"),
    ))
}

async fn api_combinations(
    State(state): State<AppState>,
    Path(target): Path<String>,
) -> Result<Json<Vec<PairResult>>, (StatusCode, String)> {
    let Some(pal) = find_pal_by_name(&state.pals, &target) else {
        return Err((StatusCode::BAD_REQUEST, "Unknown target pal".to_string()));
    };

    Ok(Json(combinations_for_target(&state, &pal.name)))
}

#[derive(Deserialize)]
struct ChainQuery {
    owned: String,
    goal: String,
}

async fn api_chain(
    State(state): State<AppState>,
    Query(query): Query<ChainQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let owned = query.owned.trim().to_string();
    let goal = query.goal.trim().to_string();
    if owned.is_empty() || goal.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "owned and goal are required".to_string()));
    }

    let owned_check = owned.clone();
    let goal_check = goal.clone();
    let work = tokio::task::spawn_blocking(move || {
        let steps = chain::find_breeding_chain(&state, &owned_check, &goal_check, 15);
        let hint = steps
            .as_ref()
            .is_none()
            .then(|| chain_failure_hint(&state, &owned_check, &goal_check))
            .flatten();
        let partial = steps.as_ref().is_none().then(|| {
            chain::direct_parent_combo_for_owned(&state, &owned_check, &goal_check).map(|pair| {
                let partner = if pair.a.eq_ignore_ascii_case(&owned_check) {
                    pair.b.clone()
                } else {
                    pair.a.clone()
                };
                serde_json::json!({
                    "parent_a": pair.a,
                    "parent_b": pair.b,
                    "child": goal_check,
                    "method": pair.method,
                    "missing_partner": partner,
                })
            })
        }).flatten();
        (steps, hint, partial)
    });

    let (steps, hint, partial) = match tokio::time::timeout(std::time::Duration::from_secs(12), work).await {
        Ok(Ok(result)) => result,
        Ok(Err(_)) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Chain search task failed".to_string(),
            ));
        }
        Err(_) => {
            return Err((
                StatusCode::REQUEST_TIMEOUT,
                format!(
                    "Chain search timed out for {owned} → {goal}. Try closer Pals or use reverse lookup."
                ),
            ));
        }
    };

    match steps {
        Some(steps) => Ok(Json(serde_json::json!({
            "owned": owned,
            "goal": goal,
            "found": true,
            "steps": steps,
            "step_count": steps.len()
        }))),
        None => Ok(Json(serde_json::json!({
            "owned": owned,
            "goal": goal,
            "found": false,
            "steps": [],
            "step_count": 0,
            "hint": hint,
            "partial": partial,
            "message": hint.clone().unwrap_or_else(|| format!(
                "No breeding chain found from {owned} to {goal} within 15 steps. Try a different owned Pal or use reverse lookup."
            ))
        }))),
    }
}

fn chain_failure_hint(state: &AppState, owned: &str, goal: &str) -> Option<String> {
    let owned_pal = find_pal_by_name(&state.pals, owned)?;
    let goal_pal = find_pal_by_name(&state.pals, goal)?;

    if let Some(pair) = chain::direct_parent_combo_for_owned(state, owned, goal) {
        let partner = if pair.a.eq_ignore_ascii_case(owned) {
            pair.b.as_str()
        } else {
            pair.a.as_str()
        };
        let partner_pal = find_pal_by_name(&state.pals, partner)?;
        return Some(format!(
            "{owned} can breed {goal} with {partner}, but the planner could not reach {partner} (breeding power {}) starting from only {owned} (power {}). Capture or breed {partner} first, then use that pair in the calculator.",
            partner_pal.power,
            owned_pal.power
        ));
    }

    let gap = goal_pal.power.saturating_sub(owned_pal.power);
    if gap > 100 {
        return Some(format!(
            "{owned} (breeding power {}) is far below {goal} (power {}). Pick an owned Pal closer to your goal, or use reverse lookup for parent pairs."
            ,
            owned_pal.power,
            goal_pal.power
        ));
    }

    None
}

async fn api_capture(
    State(state): State<AppState>,
    Path(target): Path<String>,
) -> Result<Json<CaptureResponse>, (StatusCode, String)> {
    let target_pal = state
        .pals
        .iter()
        .find(|pal| pal.name == target)
        .cloned()
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Unknown target pal".to_string()))?;

    let estimate_percent = ((1600 - target_pal.power) as f32 / 16.0).round() as i32;
    let estimate_percent = estimate_percent.clamp(4, 95);

    let mut easier_targets = state.pals.clone();
    easier_targets.sort_by_key(|pal| pal.power);
    easier_targets.truncate(5);

    Ok(Json(CaptureResponse {
        target: target_pal.name,
        estimate_percent,
        easier_targets,
    }))
}

async fn index_page(State(state): State<AppState>) -> Result<Html<String>, (StatusCode, String)> {
    render_index_with_seo(&state, SEO_PAGES[0]).await
}

async fn redirect_to_primary_calculator() -> axum::response::Redirect {
    axum::response::Redirect::permanent(PRIMARY_CALCULATOR_PATH)
}

async fn index_pals_page(State(state): State<AppState>) -> Result<Html<String>, (StatusCode, String)> {
    render_index_with_seo(&state, SEO_PAGES[2]).await
}

async fn index_map_page(State(state): State<AppState>) -> Result<Html<String>, (StatusCode, String)> {
    render_index_with_seo(&state, SEO_PAGES[3]).await
}

async fn index_items_page(
    State(state): State<AppState>,
) -> Result<Html<String>, (StatusCode, String)> {
    render_index_with_seo(&state, SEO_PAGES[4]).await
}

async fn index_capture_page(
    State(state): State<AppState>,
) -> Result<Html<String>, (StatusCode, String)> {
    render_index_with_seo(&state, SEO_PAGES[6]).await
}

async fn index_keyword_breeding_page(
    State(state): State<AppState>,
) -> Result<Html<String>, (StatusCode, String)> {
    render_index_with_seo(&state, SEO_PAGES[7]).await
}

async fn index_keyword_combos_page(
    State(state): State<AppState>,
) -> Result<Html<String>, (StatusCode, String)> {
    render_index_with_seo(&state, SEO_PAGES[8]).await
}

async fn index_keyword_capture_page(
    State(state): State<AppState>,
) -> Result<Html<String>, (StatusCode, String)> {
    render_index_with_seo(&state, SEO_PAGES[9]).await
}

async fn index_chain_breeding_page(
    State(state): State<AppState>,
) -> Result<Html<String>, (StatusCode, String)> {
    render_index_with_seo(&state, SEO_PAGES[10]).await
}

async fn index_technology_page(
    State(state): State<AppState>,
) -> Result<Html<String>, (StatusCode, String)> {
    render_index_with_seo(&state, SEO_PAGES[5]).await
}

async fn guide_page_handler(
    State(state): State<AppState>,
    axum::extract::OriginalUri(uri): axum::extract::OriginalUri,
) -> Result<Html<String>, (StatusCode, String)> {
    let path = uri.path();
    serve_guide_page(&state, path)
}

fn serve_guide_page(state: &AppState, path: &str) -> Result<Html<String>, (StatusCode, String)> {
    let page = GUIDE_PAGES
        .iter()
        .find(|p| p.path == path)
        .copied()
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Guide not found".to_string()))?;

    let mut html = build_guide_page_html(&state.base_url, page);
    let extra = match path {
        "/fastest-anubis-breed" => combinations_section_html(state, "Anubis"),
        "/legendary-breeding" => legendary_pal_links_html(state),
        "/fastest-flying-mount-breeding" => combinations_section_html(state, "Jetragon"),
        _ => String::new(),
    };
    if !extra.is_empty() {
        html = html.replace("</main>", &format!("{extra}</main>"));
    }
    Ok(Html(html))
}

async fn breed_alias_redirect(Path(name): Path<String>) -> axum::response::Redirect {
    let slug = pal_slug(&name);
    axum::response::Redirect::permanent(&format!("/pal/{slug}"))
}

fn pal_slug(name: &str) -> String {
    name.to_lowercase().replace(' ', "-")
}

fn combos_producing_target(state: &AppState, target_name: &str) -> Vec<(String, String, String)> {
    combinations_for_target(state, target_name)
        .into_iter()
        .map(|pair| (pair.a, pair.b, pair.method))
        .collect()
}

async fn pal_pages_directory(State(state): State<AppState>) -> Html<String> {
    Html(build_pal_pages_directory_html(&state))
}

async fn combo_pages_directory(State(state): State<AppState>) -> Html<String> {
    Html(build_combo_pages_directory_html(&state))
}

fn build_pal_pages_directory_html(state: &AppState) -> String {
    let links: String = seo_pal_links_html(&state.pals, None);
    let count = state.pals.len();
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>All Palworld Pal Breeding Pages ({count}) | Pal Breeding Calculator</title>
  <meta name="description" content="Browse {count} Palworld Pal breeding pages with parent combinations, breeding power, map locations, and calculator shortcuts." />
  <link rel="canonical" href="{base}/pal-pages" />
  <style>
    body {{ margin:0; font-family:Arial,sans-serif; background:#0b0f14; color:#e5ecf4; }}
    .wrap {{ max-width:1100px; margin:2rem auto; padding:0 1rem; }}
    a {{ color:#8ec8ff; }}
    h1 {{ margin-bottom:0.4rem; }}
    .muted {{ color:#9fb2c8; }}
    .seo-directory-grid {{ display:grid; grid-template-columns:repeat(auto-fill,minmax(150px,1fr)); gap:0.4rem 0.75rem; margin:1rem 0; }}
    .seo-directory-grid a {{ color:#8ec8ff; text-decoration:none; font-size:0.9rem; }}
    {SEO_BACKGROUND_STYLES}
  </style>
</head>
<body>
  {kid_bg}
  <main class="wrap">
    <p class="muted"><a href="/">Home</a> / Pal Pages</p>
    <h1>All Pal Breeding Pages</h1>
    <p class="muted">{count} Pal profile pages with combos, map data, and breeding calculator links.</p>
    <ul class="seo-directory-grid">{links}</ul>
    <p><a href="/palworld-breeding-calculator">Open breeding calculator</a></p>
  </main>
</body>
</html>"#,
        base = state.base_url,
        count = count,
        links = links,
        kid_bg = seo_kid_background_html(),
        SEO_BACKGROUND_STYLES = SEO_BACKGROUND_STYLES,
    )
}

fn build_combo_pages_directory_html(state: &AppState) -> String {
    let pairs = collect_combo_page_pairs(state, None);
    let count = pairs.len();
    let links = pairs
        .iter()
        .map(|(a, b)| combo_pair_list_item(a, b))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>Palworld Breeding Combo Pages ({count}) | Parent Pair Results</title>
  <meta name="description" content="Browse {count} Palworld parent-pair combo pages with child outcomes, special combinations, and breeding method details." />
  <link rel="canonical" href="{base}/combo-pages" />
  <style>
    body {{ margin:0; font-family:Arial,sans-serif; background:#0b0f14; color:#e5ecf4; }}
    .wrap {{ max-width:1100px; margin:2rem auto; padding:0 1rem; }}
    a {{ color:#8ec8ff; }}
    h1 {{ margin-bottom:0.4rem; }}
    .muted {{ color:#9fb2c8; }}
    .seo-directory-grid {{ display:grid; grid-template-columns:repeat(auto-fill,minmax(200px,1fr)); gap:0.4rem 0.75rem; margin:1rem 0; list-style:none; padding:0; }}
    .seo-directory-grid li {{ list-style:none; }}
    .seo-directory-grid a {{ color:#8ec8ff; text-decoration:none; font-size:0.9rem; }}
    {SEO_BACKGROUND_STYLES}
  </style>
</head>
<body>
  {kid_bg}
  <main class="wrap">
    <p class="muted"><a href="/">Home</a> / Combo Pages</p>
    <h1>Featured Breeding Combo Pages</h1>
    <p class="muted">{count} parent-pair pages including special combinations and top legendary routes.</p>
    <ul class="seo-directory-grid">{links}</ul>
    <p><a href="/palworld-breeding-calculator">Open breeding calculator</a></p>
  </main>
</body>
</html>"#,
        base = state.base_url,
        count = count,
        links = links,
        kid_bg = seo_kid_background_html(),
        SEO_BACKGROUND_STYLES = SEO_BACKGROUND_STYLES,
    )
}

async fn how_to_breed_page(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Html<String>, (StatusCode, String)> {
    let target = find_pal_by_slug(&state.pals, &name)
        .cloned()
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Pal not found".to_string()))?;
    let combos = combos_producing_target(&state, &target.name);
    Ok(Html(build_how_to_breed_html(
        &state.base_url,
        &target,
        &combos,
    )))
}

async fn combos_hub_page(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Html<String>, (StatusCode, String)> {
    let target = find_pal_by_slug(&state.pals, &name)
        .cloned()
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Pal not found".to_string()))?;
    let combos = combos_producing_target(&state, &target.name);
    Ok(Html(build_combos_hub_html(
        &state.base_url,
        &target,
        &combos,
    )))
}

async fn pal_detail_page(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Html<String>, (StatusCode, String)> {
    let target = find_pal_by_slug(&state.pals, &name)
        .cloned()
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Pal not found".to_string()))?;

    let location = state.pal_locations.get(&target.name);
    let mut combos = combos_producing_target(&state, &target.name);
    combos.truncate(25);

    let related: Vec<&Pal> = state
        .pals
        .iter()
        .filter(|pal| pal.name != target.name)
        .take(8)
        .collect();

    Ok(Html(build_pal_page_html(
        &state.base_url,
        &target,
        location,
        &combos,
        &related,
    )))
}

async fn combo_detail_page(
    State(state): State<AppState>,
    Path((parent_a_slug, parent_b_slug)): Path<(String, String)>,
) -> Result<Html<String>, (StatusCode, String)> {
    let parent_a = find_pal_by_slug(&state.pals, &parent_a_slug)
        .cloned()
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Parent A not found".to_string()))?;
    let parent_b = find_pal_by_slug(&state.pals, &parent_b_slug)
        .cloned()
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Parent B not found".to_string()))?;

    let result = calculate_child(&state, &parent_a.name, &parent_b.name)
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Could not calculate combo".to_string()))?;

    let alternative_pairs: Vec<(String, String, String)> = combinations_for_target(&state, &result.child.name)
        .into_iter()
        .map(|pair| (pair.a, pair.b, pair.method))
        .take(60)
        .collect();

    Ok(Html(build_combo_page_html(
        &state.base_url,
        &parent_a,
        &parent_b,
        &result,
        &alternative_pairs,
    )))
}

async fn render_index_with_seo(
    state: &AppState,
    page: SeoPage,
) -> Result<Html<String>, (StatusCode, String)> {
    let seo_tags = build_seo_tags(&state.base_url, page);
    let breadcrumb = visible_breadcrumb_html(page);
    let about_heading = if page.path == "/" {
        "About This Tool"
    } else {
        "About This Page"
    };
    let about_block = format!(
        "<section class=\"card route-seo-blurb\" id=\"aboutSite\"><h2>{about_heading}</h2><p>{}</p></section>",
        page.page_description
    );

    let route_intro_attr = page
        .h1_characteristics
        .replace('&', "&amp;")
        .replace('"', "&quot;");
    let body_tag = format!(r#"<body data-route-intro="{route_intro_attr}">"#);

    let html = state
        .index_template
        .replace("</head>", &format!("{seo_tags}\n  </head>"))
        .replace("<body>", &body_tag)
        .replace("__SITE_H1__", page.h1)
        .replace("__ROUTE_BADGE__", page.badge)
        .replace("__ROUTE_INTRO__", page.h1_characteristics)
        .replace("<!--VISIBLE_BREADCRUMB-->", &breadcrumb)
        .replace("<!--ROUTE_ABOUT_BLOCK-->", &about_block);
    Ok(Html(html))
}

fn visible_breadcrumb_html(page: SeoPage) -> String {
    if page.path == "/" {
        return String::from(
            r#"<nav class="visible-breadcrumb" aria-label="Breadcrumb"><span>Home</span></nav>"#,
        );
    }
    format!(
        r#"<nav class="visible-breadcrumb" aria-label="Breadcrumb"><a href="/">Home</a> / <span>{}</span></nav>"#,
        page.badge
    )
}

async fn robots_txt(State(state): State<AppState>) -> impl IntoResponse {
    let body = format!("User-agent: *\nAllow: /\n\nSitemap: {}/sitemap.xml\n", state.base_url);
    ([(axum::http::header::CONTENT_TYPE, "text/plain; charset=utf-8")], body)
}

async fn sitemap_index(State(state): State<AppState>) -> impl IntoResponse {
    let base = &state.base_url;
    let body = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <sitemap><loc>{base}/sitemap-pages.xml</loc></sitemap>
  <sitemap><loc>{base}/sitemap-pals.xml</loc></sitemap>
  <sitemap><loc>{base}/sitemap-guides.xml</loc></sitemap>
  <sitemap><loc>{base}/sitemap-combos.xml</loc></sitemap>
  <sitemap><loc>{base}/sitemap-hubs.xml</loc></sitemap>
</sitemapindex>
"#
    );
    xml_response(body)
}

async fn sitemap_pages(State(state): State<AppState>) -> impl IntoResponse {
    let lastmod = chrono_like_today();
    let mut urls = String::new();
    for page in SEO_PAGES {
        if page.path == "/breeding-calculator" {
            continue;
        }
        let path = page.canonical_path.unwrap_or(page.path);
        push_url(
            &mut urls,
            &format!("{}{}", state.base_url, path),
            &lastmod,
            "0.9",
        );
    }
    push_url(
        &mut urls,
        &format!("{}/pal-pages", state.base_url),
        &lastmod,
        "0.88",
    );
    push_url(
        &mut urls,
        &format!("{}/combo-pages", state.base_url),
        &lastmod,
        "0.87",
    );
    xml_response(urlset_body(&urls))
}

async fn sitemap_hubs(State(state): State<AppState>) -> impl IntoResponse {
    let lastmod = chrono_like_today();
    let mut urls = String::new();
    for name in PRIORITY_BREED_TARGETS {
        let slug = pal_slug(name);
        push_url(
            &mut urls,
            &format!("{}/how-to-breed/{slug}", state.base_url),
            &lastmod,
            "0.88",
        );
        push_url(
            &mut urls,
            &format!("{}/combos/{slug}", state.base_url),
            &lastmod,
            "0.86",
        );
    }
    xml_response(urlset_body(&urls))
}

async fn sitemap_guides(State(state): State<AppState>) -> impl IntoResponse {
    let lastmod = chrono_like_today();
    let mut urls = String::new();
    for page in GUIDE_PAGES {
        push_url(&mut urls, &format!("{}{}", state.base_url, page.path), &lastmod, "0.85");
    }
    xml_response(urlset_body(&urls))
}

async fn sitemap_pals(State(state): State<AppState>) -> impl IntoResponse {
    let lastmod = chrono_like_today();
    let mut urls = String::new();
    for pal in &state.pals {
        let slug = pal.name.to_lowercase().replace(' ', "-");
        push_url(
            &mut urls,
            &format!("{}/pal/{slug}", state.base_url),
            &lastmod,
            "0.8",
        );
    }
    xml_response(urlset_body(&urls))
}

async fn sitemap_combos(State(state): State<AppState>) -> impl IntoResponse {
    let lastmod = chrono_like_today();
    let mut urls = String::new();
    for (i, first) in state.pals.iter().enumerate() {
        for second in state.pals.iter().skip(i) {
            if !should_index_combo(&state, first, second) {
                continue;
            }
            let loc = format!(
                "{}/combo/{}/{}",
                state.base_url,
                first.name.to_lowercase().replace(' ', "-"),
                second.name.to_lowercase().replace(' ', "-")
            );
            push_url(&mut urls, &loc, &lastmod, "0.7");
        }
    }
    xml_response(urlset_body(&urls))
}

fn push_url(urls: &mut String, loc: &str, lastmod: &str, priority: &str) {
    urls.push_str(&format!(
        "  <url>\n    <loc>{loc}</loc>\n    <lastmod>{lastmod}</lastmod>\n    <changefreq>weekly</changefreq>\n    <priority>{priority}</priority>\n  </url>\n"
    ));
}

fn urlset_body(urls: &str) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n{urls}</urlset>\n"
    )
}

fn xml_response(body: String) -> impl IntoResponse {
    ([(axum::http::header::CONTENT_TYPE, "application/xml; charset=utf-8")], body)
}

fn should_index_combo(state: &AppState, first: &Pal, second: &Pal) -> bool {
    let Some(result) = calculate_child(state, &first.name, &second.name) else {
        return false;
    };
    if result.method.contains("Special") {
        return true;
    }
    result.child.power <= 350 || first.power <= 350 || second.power <= 350
}

fn chrono_like_today() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // Approximate UTC day without external chrono dependency.
    // Good enough for sitemap freshness metadata.
    let days = seconds / 86_400;
    // 1970-01-01 + days (rough conversion by civil algorithm)
    let z = days as i64 + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if m <= 2 { 1 } else { 0 };
    format!("{year:04}-{m:02}-{d:02}")
}

fn build_seo_tags(base_url: &str, page: SeoPage) -> String {
    let canonical_path = page.canonical_path.unwrap_or(page.path);
    let page_url = format!("{base_url}{canonical_path}");
    format!(
        r#"
    <title>{title}</title>
    <meta name="description" content="{description}" />
    <meta name="robots" content="index,follow,max-image-preview:large" />
    <link rel="canonical" href="{page_url}" />
    <link rel="manifest" href="/manifest.webmanifest" />
    <link rel="icon" href="/favicon.svg" type="image/svg+xml" />
    <link rel="apple-touch-icon" href="/assets/pals/anubis.webp" />
    <meta property="og:type" content="website" />
    <meta property="og:title" content="{title}" />
    <meta property="og:description" content="{short_description}" />
    <meta property="og:url" content="{page_url}" />
    <meta property="og:image" content="{base_url}/assets/pals/anubis.webp" />
    <meta name="twitter:card" content="summary_large_image" />
    <meta name="twitter:title" content="{title}" />
    <meta name="twitter:description" content="{short_description}" />
    <meta name="twitter:image" content="{base_url}/assets/pals/anubis.webp" />
    <script type="application/ld+json">
      {{
        "@context": "https://schema.org",
        "@type": "WebApplication",
        "name": "Palworld Breeding Calculator",
        "applicationCategory": "GameApplication",
        "operatingSystem": "Web",
        "url": "{page_url}",
        "description": "{app_description}"
      }}
    </script>
    <script type="application/ld+json">
      {{
        "@context":"https://schema.org",
        "@type":"WebSite",
        "name":"Palworld Breeding Calculator",
        "url":"{base_url}/",
        "description": "{app_description}"
      }}
    </script>
    <script type="application/ld+json">
      {{
        "@context":"https://schema.org",
        "@type":"Organization",
        "name":"Palworld Breeding Calculator",
        "url":"{base_url}/",
        "logo":"{base_url}/favicon.svg"
      }}
    </script>
    <script type="application/ld+json">
      {{
        "@context": "https://schema.org",
        "@type": "FAQPage",
        "mainEntity": [
          {{
            "@type": "Question",
            "name": "How do I breed Anubis?",
            "acceptedAnswer": {{
              "@type": "Answer",
              "text": "Use the combination explorer and reverse calculator to test high-value parent pairs, then follow the dedicated Anubis breeding guide."
            }}
          }},
          {{
            "@type": "Question",
            "name": "How do I find best breeding combos?",
            "acceptedAnswer": {{
              "@type": "Answer",
              "text": "Start with the Reverse Calculator, select your target Pal, and compare all listed parent pairs and method types."
            }}
          }},
          {{
            "@type": "Question",
            "name": "Can I use this on mobile?",
            "acceptedAnswer": {{
              "@type": "Answer",
              "text": "Yes. The interface is responsive and works on phones and tablets for quick breeding checks on the go."
            }}
          }},
          {{
            "@type": "Question",
            "name": "Can legendary pals breed?",
            "acceptedAnswer": {{
              "@type": "Answer",
              "text": "Yes. Legendary Pals can breed when placed in a Breeding Farm, but they often require long parent chains and special combinations."
            }}
          }},
          {{
            "@type": "Question",
            "name": "Does gender matter in Palworld breeding?",
            "acceptedAnswer": {{
              "@type": "Answer",
              "text": "Gender does not change the predicted child outcome in standard breeding calculations. Focus on parent pair combinations and breeding power."
            }}
          }},
          {{
            "@type": "Question",
            "name": "Why is breeding not working?",
            "acceptedAnswer": {{
              "@type": "Answer",
              "text": "Check that your Breeding Farm is built, both parents are assigned, and you have cake in the feed box. Also verify you are using a valid parent pair for your target child."
            }}
          }},
          {{
            "@type": "Question",
            "name": "What is the best early game breeding combo?",
            "acceptedAnswer": {{
              "@type": "Answer",
              "text": "Early game combos usually use easy Pals like Lamball, Cattiva, and Foxparks. Use the calculator to test low-risk pairs before targeting rare children."
            }}
          }}
        ]
      }}
    </script>"#,
        title = page.title,
        description = page.page_description,
        short_description = page.meta_description,
        app_description = page.meta_description,
        page_url = page_url
    )
}

fn query_escape(s: &str) -> String {
    s.replace(' ', "%20").replace('&', "%26")
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

const SEO_BACKGROUND_STYLES: &str = r#"
      body { position: relative; min-height: 100vh; overflow-x: hidden; }
      .wrap { position: relative; z-index: 2; }
      .kid-bg {
        position: fixed;
        inset: 0;
        z-index: 0;
        pointer-events: none;
        overflow: hidden;
        background:
          radial-gradient(ellipse 80% 50% at 20% 30%, rgba(83, 232, 255, 0.12), transparent 55%),
          radial-gradient(ellipse 70% 45% at 80% 70%, rgba(255, 79, 216, 0.1), transparent 50%),
          radial-gradient(ellipse 60% 40% at 50% 50%, rgba(255, 241, 95, 0.06), transparent 60%);
      }
      .bg-pal-sticker {
        position: absolute;
        border-radius: 10px;
        opacity: 0.28;
        filter: saturate(1.4) brightness(1.1);
        border: 1px solid rgba(255, 255, 255, 0.35);
        box-shadow: 0 0 12px rgba(83, 232, 255, 0.35);
        animation-name: stickerFloat;
        animation-timing-function: ease-in-out;
        animation-iteration-count: infinite;
        mix-blend-mode: screen;
        object-fit: cover;
      }
      @keyframes stickerFloat {
        0%, 100% { translate: 0 0; }
        50% { translate: 0 -8px; }
      }
      @media (prefers-reduced-motion: reduce) {
        .bg-pal-sticker { animation: none !important; }
      }
"#;

const SEO_FOOTER_STYLES: &str = r#"
      .site-footer { margin-top: 2.5rem; padding: 2rem 1rem 1.25rem; border-top: 1px solid #2d3a4d; background: #0a0e14; color: #c5d4e8; font-size: 0.88rem; }
      .site-footer-inner { max-width: 1100px; margin: 0 auto; display: grid; grid-template-columns: repeat(auto-fit, minmax(160px, 1fr)); gap: 1.25rem 1.5rem; }
      .site-footer-title { font-weight: 700; margin: 0 0 0.35rem; color: #e8f0fa; }
      .site-footer-muted { margin: 0; color: #8fa3bc; line-height: 1.45; font-size: 0.82rem; }
      .site-footer-heading { margin: 0 0 0.45rem; font-weight: 700; color: #b8d4f0; font-size: 0.8rem; text-transform: uppercase; letter-spacing: 0.04em; }
      .site-footer-col ul { list-style: none; margin: 0; padding: 0; }
      .site-footer-col li { margin: 0.28rem 0; }
      .site-footer a { color: #8ec8ff; text-decoration: none; }
      .site-footer a:hover { text-decoration: underline; }
      .site-footer-copy { max-width: 1100px; margin: 1.25rem auto 0; padding-top: 0.85rem; border-top: 1px solid #1e2a3a; text-align: center; color: #6d8299; font-size: 0.78rem; }
"#;

const SEO_PAL_IMG_STYLES: &str = r#"
      .pal-table-link { display: inline-flex; align-items: center; gap: 0.4rem; text-decoration: none; color: #8ec8ff; }
      .pal-table-link span { line-height: 1.2; }
      .pal-table-img { width: 36px; height: 36px; border-radius: 8px; object-fit: cover; border: 1px solid #4b6281; background: #0b1220; flex: 0 0 auto; }
      .combo-hero { display: flex; align-items: center; gap: 0.45rem; margin: 0 0 0.85rem; flex-wrap: wrap; }
      .combo-hero-img { width: 52px; height: 52px; border-radius: 10px; object-fit: cover; border: 1px solid #4b6281; background: #0b1220; }
      .combo-hero-eq { color: #9fb2c8; font-weight: 700; font-size: 1.1rem; }
      .combo-pair-cell { vertical-align: middle; }
      .pal-table-pair { display: inline-flex; align-items: center; gap: 0.35rem; flex-wrap: wrap; }
      .pal-table-pair .plus { color: #9fb2c8; font-weight: 700; }
"#;

fn seo_kid_background_html() -> String {
    const PALS: &[&str] = &[
        "Lamball", "Cattiva", "Chikipi", "Foxparks", "Pengullet", "Anubis", "Jetragon", "Frostallion",
        "Blazamut", "Suzaku", "Necromus", "Paladius", "Relaxaurus", "Penking", "Elizabee", "Grizzbolt",
        "Lyleen", "Mossanda", "Azurobe", "Incineram", "Beakon", "Sibelyx", "Astegon", "Shadowbeak",
        "Bellanoir", "Kitsun", "Rooby", "Daedream",
    ];
    const SLOTS: &[(u8, u8, u8, i8)] = &[
        (1, 4, 76, -12),
        (3, 28, 58, 6),
        (2, 52, 64, -8),
        (4, 76, 54, 10),
        (1, 90, 62, -15),
        (88, 3, 72, 14),
        (91, 26, 56, -9),
        (89, 48, 68, 11),
        (92, 70, 60, -7),
        (87, 88, 66, 16),
        (14, 2, 50, 8),
        (78, 2, 48, -11),
        (8, 42, 44, -5),
        (84, 38, 46, 7),
        (6, 64, 42, 12),
        (86, 58, 44, -13),
        (18, 86, 40, 6),
        (76, 84, 42, -8),
        (22, 14, 38, -4),
        (72, 16, 38, 5),
        (10, 18, 36, 9),
        (82, 20, 36, -6),
    ];
    let mut imgs = String::new();
    for (i, &(l, t, s, r)) in SLOTS.iter().enumerate() {
        let name = PALS[i % PALS.len()];
        let slug = pal_slug(name);
        let cdn = query_escape(name);
        let delay = (i % 8) as f32 * 0.22;
        let dur = 7 + (i % 6);
        imgs.push_str(&format!(
            r#"<img class="bg-pal-sticker" src="/assets/pals/{slug}.webp" alt="" loading="lazy" decoding="async" style="left:{l}%;top:{t}%;width:{s}px;height:{s}px;transform:rotate({r}deg);animation-delay:{delay}s;animation-duration:{dur}s;" onerror="if(!this.dataset.f){{this.dataset.f='cdn';this.src='https://ggservers.com/images/palworld/{cdn}.webp';}}else{{this.onerror=null;this.src='/assets/pals/placeholder.svg';}}" />"#
        ));
    }
    format!(r#"<div class="kid-bg" aria-hidden="true">{imgs}</div>"#)
}

fn pal_img_tag(name: &str, class: &str) -> String {
    let slug = pal_slug(name);
    let esc = html_escape(name);
    let cdn = query_escape(name);
    format!(
        r#"<img class="{class}" src="/assets/pals/{slug}.webp" alt="{esc}" width="36" height="36" loading="lazy" decoding="async" onerror="if(!this.dataset.f){{this.dataset.f='cdn';this.src='https://ggservers.com/images/palworld/{cdn}.webp';}}else{{this.onerror=null;this.src='/assets/pals/placeholder.svg';}}" />"#
    )
}

fn combo_parent_cell(name: &str, combo_a: &str, combo_b: &str) -> String {
    let a_slug = pal_slug(combo_a);
    let b_slug = pal_slug(combo_b);
    format!(
        r#"<a class="pal-table-link" href="/combo/{a_slug}/{b_slug}">{img}<span>{label}</span></a>"#,
        img = pal_img_tag(name, "pal-table-img"),
        label = html_escape(name)
    )
}

fn combo_pair_table_row(a: &str, b: &str, method: &str) -> String {
    format!(
        "<tr><td>{}</td><td>{}</td><td>{}</td></tr>",
        combo_parent_cell(a, a, b),
        combo_parent_cell(b, a, b),
        html_escape(method)
    )
}

fn combo_table_rows(combos: &[(String, String, String)], limit: usize) -> String {
    combos
        .iter()
        .take(limit)
        .map(|(a, b, method)| {
            let a_slug = pal_slug(a);
            let b_slug = pal_slug(b);
            format!(
                r#"<tr><td class="combo-pair-cell"><span class="pal-table-pair">{pa}<span class="plus">+</span>{pb}</span></td><td>{method}</td></tr>"#,
                pa = combo_parent_cell(a, &a_slug, &b_slug),
                pb = combo_parent_cell(b, &a_slug, &b_slug),
                method = html_escape(method)
            )
        })
        .collect::<Vec<_>>()
        .join("")
}

fn pal_tier_note(power: i32) -> &'static str {
    if power <= 350 {
        "high-priority rare / legendary tier"
    } else if power <= 800 {
        "mid-game breeding target"
    } else {
        "often used as an accessible parent in chains"
    }
}

fn build_how_to_breed_html(
    base_url: &str,
    target: &Pal,
    combos: &[(String, String, String)],
) -> String {
    let slug = pal_slug(&target.name);
    let page_url = format!("{base_url}/how-to-breed/{slug}");
    let title = format!(
        "How to Breed {} in Palworld – Fastest Combos & Parent Guide",
        target.name
    );
    let description = format!(
        "Learn how to breed {} in Palworld with every parent pair, special combos, breeding power tips, and links to our free reverse breeding calculator.",
        target.name
    );
    let rows = combo_table_rows(combos, 20);
    let calc_link = format!(
        "{}?target={}",
        PRIMARY_CALCULATOR_PATH,
        query_escape(&target.name)
    );
    format!(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>{title}</title>
    <meta name="description" content="{description}" />
    <meta name="robots" content="index,follow,max-image-preview:large" />
    <link rel="canonical" href="{page_url}" />
    <meta property="og:title" content="{title}" />
    <meta property="og:description" content="{description}" />
    <meta property="og:url" content="{page_url}" />
    <style>
      body {{ margin: 0; font-family: Arial, Helvetica, sans-serif; background: #0b0f14; color: #e5ecf4; }}
      .wrap {{ max-width: 920px; margin: 2rem auto; padding: 0 1rem; }}
      a {{ color: #8ec8ff; }}
      .crumbs {{ color: #9fb2c8; font-size: 0.9rem; margin-bottom: 0.8rem; }}
      .card {{ background: #131a23; border: 1px solid #2d3a4d; border-radius: 10px; padding: 1rem; margin-bottom: 1rem; }}
      table {{ width: 100%; border-collapse: collapse; }}
      th, td {{ border: 1px solid #33465a; padding: 0.45rem 0.52rem; text-align: left; font-size: 0.9rem; vertical-align: middle; }}
      th {{ background: #182230; }}
      {SEO_BACKGROUND_STYLES}
      {SEO_PAL_IMG_STYLES}
    </style>
    <script type="application/ld+json">
      {{"@context":"https://schema.org","@type":"HowTo","name":"How to breed {pal_name} in Palworld","description":"{description}","step":[{{"@type":"HowToStep","name":"Unlock Breeding Farm","text":"Research and build a Breeding Farm and Egg Incubator with cake in the feed box."}},{{"@type":"HowToStep","name":"Find parent pairs","text":"Use the reverse calculator to list valid parents for {pal_name}."}},{{"@type":"HowToStep","name":"Breed and incubate","text":"Assign parents, collect the egg, and incubate until {pal_name} hatches."}}]}}
    </script>
    <script type="application/ld+json">
      {{"@context":"https://schema.org","@type":"FAQPage","mainEntity":[{{"@type":"Question","name":"How do you breed {pal_name}?","acceptedAnswer":{{"@type":"Answer","text":"Use valid parent pairs listed on this page or the reverse calculator. Special combinations override power averages when available."}}}},{{"@type":"Question","name":"How many combos produce {pal_name}?","acceptedAnswer":{{"@type":"Answer","text":"This database lists {combo_count} parent pair routes that can produce {pal_name}."}}}}]}}
    </script>
  </head>
  <body>
    {kid_bg}
    <main class="wrap">
      <nav class="crumbs" aria-label="Breadcrumb"><a href="/">Home</a> / <a href="/palworld-breeding-calculator">Calculator</a> / <a href="/pal/{slug}">{pal_name}</a> / How to Breed</nav>
      <div class="card">
        <h1>How to Breed {pal_name} in Palworld</h1>
        <p><strong>Breeding power:</strong> {power} — {tier_note}</p>
        <p>Follow this route: unlock breeding → pick parents from the table → run eggs in the incubator. Use the <a href="{calc_link}">breeding calculator</a> with {pal_name} pre-selected as target.</p>
        <h2>Quick steps</h2>
        <ol>
          <li>Build <strong>Breeding Farm</strong> + <strong>Egg Incubator</strong> and keep cake stocked.</li>
          <li>Open the <a href="/combos/{slug}">all combos for {pal_name}</a> page or table below.</li>
          <li>Farm the easiest parents first (higher breeding power = usually easier captures).</li>
          <li>Place parents, hatch the child, repeat if you need better passives.</li>
        </ol>
        <p><a href="/pal/{slug}">{pal_name} Pal profile</a> • <a href="/palworld-breeding-calculator">Full calculator</a></p>
      </div>
      <div class="card">
        <h2>Parent pairs that produce {pal_name}</h2>
        <p>Showing up to 20 routes. Method shows special combo vs power average.</p>
        <table><thead><tr><th>Parents</th><th>Method</th></tr></thead><tbody>{rows}</tbody></table>
      </div>
    </main>
  </body>
</html>"#,
        title = title,
        description = description,
        page_url = page_url,
        pal_name = target.name,
        slug = slug,
        power = target.power,
        combo_count = combos.len(),
        rows = rows,
        calc_link = calc_link,
        tier_note = pal_tier_note(target.power),
        SEO_PAL_IMG_STYLES = SEO_PAL_IMG_STYLES,
        SEO_BACKGROUND_STYLES = SEO_BACKGROUND_STYLES,
        kid_bg = seo_kid_background_html(),
    )
}

fn build_combos_hub_html(
    base_url: &str,
    target: &Pal,
    combos: &[(String, String, String)],
) -> String {
    let slug = pal_slug(&target.name);
    let page_url = format!("{base_url}/combos/{slug}");
    let title = format!(
        "{} Breeding Combinations – All Parent Pairs | Palworld",
        target.name
    );
    let description = format!(
        "Complete list of Palworld breeding combinations to get {}. {} parent pairs with special combos and power-average routes linked to our calculator.",
        target.name,
        combos.len()
    );
    let rows = combo_table_rows(combos, 30);
    let calc_link = format!(
        "{}?target={}",
        PRIMARY_CALCULATOR_PATH,
        query_escape(&target.name)
    );
    format!(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>{title}</title>
    <meta name="description" content="{description}" />
    <meta name="robots" content="index,follow,max-image-preview:large" />
    <link rel="canonical" href="{page_url}" />
    <meta property="og:title" content="{title}" />
    <meta property="og:description" content="{description}" />
    <meta property="og:url" content="{page_url}" />
    <style>
      body {{ margin: 0; font-family: Arial, Helvetica, sans-serif; background: #0b0f14; color: #e5ecf4; }}
      .wrap {{ max-width: 960px; margin: 2rem auto; padding: 0 1rem; }}
      a {{ color: #8ec8ff; }}
      .crumbs {{ color: #9fb2c8; font-size: 0.9rem; margin-bottom: 0.8rem; }}
      .card {{ background: #131a23; border: 1px solid #2d3a4d; border-radius: 10px; padding: 1rem; margin-bottom: 1rem; }}
      table {{ width: 100%; border-collapse: collapse; }}
      th, td {{ border: 1px solid #33465a; padding: 0.45rem 0.52rem; text-align: left; font-size: 0.9rem; vertical-align: middle; }}
      th {{ background: #182230; }}
      {SEO_BACKGROUND_STYLES}
      {SEO_PAL_IMG_STYLES}
    </style>
    <script type="application/ld+json">
      {{"@context":"https://schema.org","@type":"ItemList","name":"{pal_name} breeding combinations","numberOfItems":{combo_count},"itemListElement":[]}}
    </script>
  </head>
  <body>
    {kid_bg}
    <main class="wrap">
      <nav class="crumbs" aria-label="Breadcrumb"><a href="/">Home</a> / <a href="/palworld-breeding-combinations">Combinations</a> / {pal_name}</nav>
      <div class="card">
        <h1>{pal_name} Breeding Combinations</h1>
        <p><strong>{combo_count}</strong> parent pair routes can produce <strong>{pal_name}</strong> (breeding power {power}).</p>
        <p><a href="{calc_link}">Open reverse calculator for {pal_name}</a> • <a href="/how-to-breed/{slug}">How to breed {pal_name}</a> • <a href="/pal/{slug}">Pal page</a></p>
      </div>
      <div class="card">
        <h2>All parent pairs</h2>
        <table><thead><tr><th>Parents</th><th>Method</th></tr></thead><tbody>{rows}</tbody></table>
      </div>
    </main>
  </body>
</html>"#,
        title = title,
        description = description,
        page_url = page_url,
        pal_name = target.name,
        slug = slug,
        power = target.power,
        combo_count = combos.len(),
        rows = rows,
        calc_link = calc_link,
        SEO_PAL_IMG_STYLES = SEO_PAL_IMG_STYLES,
        SEO_BACKGROUND_STYLES = SEO_BACKGROUND_STYLES,
        kid_bg = seo_kid_background_html(),
    )
}

fn build_guide_page_html(base_url: &str, page: GuidePage) -> String {
    let page_url = format!("{base_url}{}", page.path);
    format!(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>{title}</title>
    <meta name="description" content="{description}" />
    <meta name="robots" content="index,follow,max-image-preview:large" />
    <link rel="canonical" href="{page_url}" />
    <meta property="og:type" content="article" />
    <meta property="og:title" content="{title}" />
    <meta property="og:description" content="{description}" />
    <meta property="og:url" content="{page_url}" />
    <meta property="og:image" content="{base_url}/assets/pals/anubis.webp" />
    <meta name="twitter:card" content="summary_large_image" />
    <meta name="twitter:title" content="{title}" />
    <meta name="twitter:description" content="{description}" />
    <style>
      body {{ margin: 0; font-family: Arial, Helvetica, sans-serif; background: #0b0f14; color: #e5ecf4; }}
      .wrap {{ max-width: 860px; margin: 2rem auto; padding: 0 1rem; }}
      a {{ color: #8ec8ff; }}
      .card {{ background: #131a23; border: 1px solid #2d3a4d; border-radius: 10px; padding: 1rem; }}
      h1 {{ margin-top: 0; }}
      .links {{ margin-top: 1rem; display: flex; gap: 0.75rem; flex-wrap: wrap; }}
      {SEO_BACKGROUND_STYLES}
      {SEO_FOOTER_STYLES}
    </style>
    <script type="application/ld+json">
      {{
        "@context":"https://schema.org",
        "@type":"Article",
        "headline":"{title}",
        "description":"{description}",
        "mainEntityOfPage":"{page_url}"
      }}
    </script>
    <script type="application/ld+json">
      {{
        "@context":"https://schema.org",
        "@type":"BreadcrumbList",
        "itemListElement":[
          {{"@type":"ListItem","position":1,"name":"Home","item":"{base_url}/"}},
          {{"@type":"ListItem","position":2,"name":"Guides","item":"{base_url}/guides/best-breeding-combos"}},
          {{"@type":"ListItem","position":3,"name":"{heading}","item":"{page_url}"}}
        ]
      }}
    </script>
  </head>
  <body>
    {kid_bg}
    <main class="wrap">
      <nav class="crumbs" aria-label="Breadcrumb"><a href="/">Home</a> / <a href="/palworld-breeding-calculator">Calculator</a> / {heading}</nav>
      <div class="card">
        <h1>{heading}</h1>
        {body_html}
        <div class="links">
          <a href="/palworld-breeding-calculator">Breeding Calculator</a>
          <a href="/how-to-breed/anubis">How to Breed Anubis</a>
          <a href="/guides/best-breeding-combos">Best Combos Guide</a>
          <a href="/guides/breeding-not-working">Breeding Fixes</a>
        </div>
      </div>
    </main>
    {site_footer}
  </body>
</html>"#
        ,
        title = page.title,
        description = page.description,
        heading = page.heading,
        body_html = page.body_html,
        page_url = page_url,
        kid_bg = seo_kid_background_html(),
        SEO_BACKGROUND_STYLES = SEO_BACKGROUND_STYLES,
        SEO_FOOTER_STYLES = SEO_FOOTER_STYLES,
        site_footer = seo_copy::SITE_FOOTER_HTML,
    )
}

fn build_pal_page_html(
    base_url: &str,
    target: &Pal,
    location: Option<&data::PalLocation>,
    combos: &[(String, String, String)],
    related: &[&Pal],
) -> String {
    let slug = target.name.to_lowercase().replace(' ', "-");
    let page_url = format!("{base_url}/pal/{slug}");
    let title = format!(
        "{} Breeding Calculator – All Combos, Chain Paths & How to Breed | Palworld",
        target.name
    );
    let description = pal_page_meta_description(target, combos.len());
    let combo_rows = combos
        .iter()
        .map(|(a, b, method)| combo_pair_table_row(a, b, method))
        .collect::<Vec<_>>()
        .join("");
    let related_links = related
        .iter()
        .map(|pal| {
            let s = pal.name.to_lowercase().replace(' ', "-");
            format!("<a href=\"/pal/{s}\">{}</a>", pal.name)
        })
        .collect::<Vec<_>>()
        .join(" • ");
    let combo_links = combos
        .iter()
        .take(6)
        .map(|(a, b, _)| {
            format!(
                "<a href=\"/combo/{}/{}\">{} + {}</a>",
                a.to_lowercase().replace(' ', "-"),
                b.to_lowercase().replace(' ', "-"),
                a,
                b
            )
        })
        .collect::<Vec<_>>()
        .join(" • ");
    let area = location
        .map(|l| l.area.as_str())
        .unwrap_or("Palpagos Island (search in-game map)");
    let coords = location.map(|l| l.coords.as_str()).unwrap_or("Open Map tab");
    let dynamic_summary = format!(
        "This page lists {} parent pair routes that can produce {} using live breeding logic. With breeding power {}, {} is {} to plan in long chains.",
        combos.len(),
        target.name,
        target.power,
        target.name,
        pal_tier_note(target.power)
    );
    let combo_count = combos.len();
    let calc_link = format!(
        "{}?target={}",
        PRIMARY_CALCULATOR_PATH,
        query_escape(&target.name)
    );
    let pal_query = query_escape(&target.name);
    let guide_links = format!(
        "<a href=\"/how-to-breed/{slug}\">How to breed {}</a> • <a href=\"/guides/best-breeding-combos\">Best Breeding Combos</a> • <a href=\"/guides/capture-rate-explained\">Capture Rate</a> • <a href=\"/guides/breeding-not-working\">Breeding fixes</a>",
        target.name
    );
    format!(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>{title}</title>
    <meta name="description" content="{description}" />
    <meta name="robots" content="index,follow,max-image-preview:large" />
    <link rel="canonical" href="{page_url}" />
    <meta property="og:type" content="article" />
    <meta property="og:title" content="{title}" />
    <meta property="og:description" content="{description}" />
    <meta property="og:url" content="{page_url}" />
    <meta property="og:image" content="{base_url}/assets/pals/{slug}.webp" />
    <style>
      body {{ margin: 0; font-family: Arial, Helvetica, sans-serif; background: #0b0f14; color: #e5ecf4; }}
      .wrap {{ max-width: 980px; margin: 2rem auto; padding: 0 1rem; }}
      a {{ color: #8ec8ff; }}
      .crumbs {{ color: #9fb2c8; font-size: 0.9rem; margin-bottom: 0.8rem; }}
      .card {{ background: #131a23; border: 1px solid #2d3a4d; border-radius: 10px; padding: 1rem; margin-bottom: 1rem; }}
      table {{ width: 100%; border-collapse: collapse; }}
      th, td {{ border: 1px solid #33465a; padding: 0.45rem 0.52rem; text-align: left; font-size: 0.9rem; vertical-align: middle; }}
      th {{ background: #182230; }}
      {SEO_BACKGROUND_STYLES}
      {SEO_PAL_IMG_STYLES}
      {SEO_FOOTER_STYLES}
    </style>
    <script type="application/ld+json">
      {{
        "@context":"https://schema.org",
        "@type":"Article",
        "headline":"{title}",
        "description":"{description}",
        "mainEntityOfPage":"{page_url}"
      }}
    </script>
    <script type="application/ld+json">
      {{
        "@context":"https://schema.org",
        "@type":"BreadcrumbList",
        "itemListElement":[
          {{"@type":"ListItem","position":1,"name":"Home","item":"{base_url}/"}},
          {{"@type":"ListItem","position":2,"name":"Pals","item":"{base_url}/pals"}},
          {{"@type":"ListItem","position":3,"name":"{pal_name}","item":"{page_url}"}}
        ]
      }}
    </script>
  </head>
  <body>
    {kid_bg}
    <main class="wrap">
      <div class="crumbs"><a href="/">Home</a> / <a href="/pals">Pals</a> / {pal_name}</div>
      <div class="card">
        <h1>{pal_name} in Palworld</h1>
        <p><strong>Breeding Power:</strong> {power}</p>
        <p><strong>Map Area:</strong> {area} | <strong>Coordinates:</strong> {coords}</p>
        <p>{dynamic_summary}</p>
        <p><a href="/how-to-breed/{slug}">How to breed {pal_name}</a> • <a href="/combos/{slug}">All {pal_name} combos ({combo_count})</a> • <a href="{calc_link}">Reverse calculator</a> • <a href="/palworld-chain-breeding?owned=Lamball&goal={pal_query}">Chain from Lamball</a> • <a href="/palworld-breeding-calculator">Breeding tool</a></p>
      </div>
      <div class="card">
        <h2>{pal_name} Parent Combinations</h2>
        <table>
          <thead><tr><th>Parent A</th><th>Parent B</th><th>Method</th></tr></thead>
          <tbody>{combo_rows}</tbody>
        </table>
      </div>
      <div class="card">
        <h2>Related Pal Pages</h2>
        <p>{related_links}</p>
        <h3>Related Combo Pages</h3>
        <p>{combo_links}</p>
        <h3>Related Guides</h3>
        <p>{guide_links}</p>
        <p><a href="/">Back to Calculator</a></p>
      </div>
    </main>
    {site_footer}
  </body>
</html>"#,
        title = title,
        description = description,
        page_url = page_url,
        slug = slug,
        pal_name = target.name,
        power = target.power,
        area = area,
        coords = coords,
        combo_rows = combo_rows,
        related_links = related_links,
        combo_links = combo_links,
        guide_links = guide_links,
        dynamic_summary = dynamic_summary,
        combo_count = combo_count,
        calc_link = calc_link,
        pal_query = pal_query,
        SEO_PAL_IMG_STYLES = SEO_PAL_IMG_STYLES,
        SEO_BACKGROUND_STYLES = SEO_BACKGROUND_STYLES,
        SEO_FOOTER_STYLES = SEO_FOOTER_STYLES,
        site_footer = seo_copy::SITE_FOOTER_HTML,
        kid_bg = seo_kid_background_html(),
    )
}

fn pal_page_meta_description(pal: &Pal, combo_count: usize) -> String {
    let base = format!(
        "Complete {} Palworld breeding guide with {} verified parent pairs, breeding power {}, map location, chain breeding paths, and step-by-step how-to-breed instructions. Use our free calculator to reverse lookup every combo that produces {} and compare special combinations versus power-average outcomes before you spend cake in the Breeding Farm. Links to dedicated combo pages, capture tips, and related legendary routes help you plan full chains from early-game Pals to endgame targets without wasted eggs.",
        pal.name, combo_count, pal.power, pal.name
    );
    if base.len() > 320 {
        base
    } else {
        format!(
            "{base} Popular searches: how to breed {}, {} breeding combinations, {} breeding calculator, fastest {} path, {} parent pairs.",
            pal.name, pal.name, pal.name, pal.name, pal.name
        )
    }
}

fn build_combo_page_html(
    base_url: &str,
    parent_a: &Pal,
    parent_b: &Pal,
    result: &CalculateResponse,
    alternatives: &[(String, String, String)],
) -> String {
    let parent_a_slug = parent_a.name.to_lowercase().replace(' ', "-");
    let parent_b_slug = parent_b.name.to_lowercase().replace(' ', "-");
    let page_url = format!("{base_url}/combo/{parent_a_slug}/{parent_b_slug}");
    let title = format!(
        "{} + {} Breeding Result – {} | Palworld Calculator",
        parent_a.name, parent_b.name, result.child.name
    );
    let description = format!(
        "What does {} + {} breed in Palworld? Child: {}. Method: {}. See alternative parent routes, egg chains, and calculator links for this combo instantly.",
        parent_a.name, parent_b.name, result.child.name, result.method
    );
    let alt_rows = alternatives
        .iter()
        .map(|(a, b, method)| combo_pair_table_row(a, b, method))
        .collect::<Vec<_>>()
        .join("");
    let combo_hero = format!(
        r#"<div class="combo-hero">{pa}<span class="combo-hero-eq">+</span>{pb}<span class="combo-hero-eq">=</span>{child}</div>"#,
        pa = pal_img_tag(&parent_a.name, "combo-hero-img"),
        pb = pal_img_tag(&parent_b.name, "combo-hero-img"),
        child = pal_img_tag(&result.child.name, "combo-hero-img"),
    );
    let related_combo_links = alternatives
        .iter()
        .take(8)
        .map(|(a, b, _)| {
            format!(
                "<a href=\"/combo/{}/{}\">{} + {}</a>",
                a.to_lowercase().replace(' ', "-"),
                b.to_lowercase().replace(' ', "-"),
                a,
                b
            )
        })
        .collect::<Vec<_>>()
        .join(" • ");
    let related_guides = [
        ("/guides/best-breeding-combos", "Best Breeding Combos"),
        ("/guides/capture-rate-explained", "Capture Rate Explained"),
        ("/guides/how-to-breed-anubis", "How to Breed Anubis"),
    ]
    .iter()
    .map(|(href, label)| format!("<a href=\"{href}\">{label}</a>"))
    .collect::<Vec<_>>()
    .join(" • ");
    let distance = result
        .distance
        .map(|d| d.to_string())
        .unwrap_or_else(|| "N/A".to_string());

    format!(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>{title}</title>
    <meta name="description" content="{description}" />
    <meta name="robots" content="index,follow,max-image-preview:large" />
    <link rel="canonical" href="{page_url}" />
    <meta property="og:type" content="article" />
    <meta property="og:title" content="{title}" />
    <meta property="og:description" content="{description}" />
    <meta property="og:url" content="{page_url}" />
    <meta property="og:image" content="{base_url}/assets/pals/{child_slug}.webp" />
    <style>
      body {{ margin: 0; font-family: Arial, Helvetica, sans-serif; background: #0b0f14; color: #e5ecf4; }}
      .wrap {{ max-width: 980px; margin: 2rem auto; padding: 0 1rem; }}
      a {{ color: #8ec8ff; }}
      .crumbs {{ color: #9fb2c8; font-size: 0.9rem; margin-bottom: 0.8rem; }}
      .card {{ background: #131a23; border: 1px solid #2d3a4d; border-radius: 10px; padding: 1rem; margin-bottom: 1rem; }}
      table {{ width: 100%; border-collapse: collapse; }}
      th, td {{ border: 1px solid #33465a; padding: 0.45rem 0.52rem; text-align: left; font-size: 0.9rem; vertical-align: middle; }}
      th {{ background: #182230; }}
      {SEO_BACKGROUND_STYLES}
      {SEO_PAL_IMG_STYLES}
      {SEO_FOOTER_STYLES}
    </style>
    <script type="application/ld+json">
      {{
        "@context":"https://schema.org",
        "@type":"FAQPage",
        "mainEntity":[
          {{
            "@type":"Question",
            "name":"What child do you get from {parent_a_name} and {parent_b_name}?",
            "acceptedAnswer":{{"@type":"Answer","text":"This combo produces {child_name} in this calculator."}}
          }},
          {{
            "@type":"Question",
            "name":"Is this a special combination?",
            "acceptedAnswer":{{"@type":"Answer","text":"Method type: {method}."}}
          }}
        ]
      }}
    </script>
    <script type="application/ld+json">
      {{
        "@context":"https://schema.org",
        "@type":"BreadcrumbList",
        "itemListElement":[
          {{"@type":"ListItem","position":1,"name":"Home","item":"{base_url}/"}},
          {{"@type":"ListItem","position":2,"name":"Breeding Calculator","item":"{base_url}{calc_path}"}},
          {{"@type":"ListItem","position":3,"name":"{parent_a_name} + {parent_b_name}","item":"{page_url}"}}
        ]
      }}
    </script>
  </head>
  <body>
    {kid_bg}
    <main class="wrap">
      <div class="crumbs"><a href="/">Home</a> / <a href="{calc_path}">Breeding Calculator</a> / {parent_a_name} + {parent_b_name}</div>
      <div class="card">
        {combo_hero}
        <h1>{parent_a_name} + {parent_b_name} = {child_name}</h1>
        <p><strong>Method:</strong> {method}</p>
        <p><strong>Distance:</strong> {distance}</p>
        <p>This combination page helps players compare direct outcome and alternative parent routes for the same child.</p>
      </div>
      <div class="card">
        <h2>Alternative Parent Routes for {child_name}</h2>
        <table>
          <thead><tr><th>Parent A</th><th>Parent B</th><th>Method</th></tr></thead>
          <tbody>{alt_rows}</tbody>
        </table>
      </div>
      <div class="card">
        <h2>Related Combo Pages</h2>
        <p>{related_combo_links}</p>
        <h3>Related Guides</h3>
        <p>{related_guides}</p>
        <p><a href="/pal/{parent_a_slug}">Open {parent_a_name} page</a> • <a href="/pal/{parent_b_slug}">Open {parent_b_name} page</a> • <a href="/pal/{child_slug}">Open {child_name} page</a> • <a href="{calc_path}">Breeding calculator</a></p>
      </div>
    </main>
    {site_footer}
  </body>
</html>"#,
        title = title,
        description = description,
        page_url = page_url,
        child_slug = result.child.name.to_lowercase().replace(' ', "-"),
        parent_a_slug = parent_a_slug,
        parent_b_slug = parent_b_slug,
        parent_a_name = parent_a.name,
        parent_b_name = parent_b.name,
        child_name = result.child.name,
        method = result.method,
        distance = distance,
        alt_rows = alt_rows,
        combo_hero = combo_hero,
        SEO_PAL_IMG_STYLES = SEO_PAL_IMG_STYLES,
        SEO_BACKGROUND_STYLES = SEO_BACKGROUND_STYLES,
        SEO_FOOTER_STYLES = SEO_FOOTER_STYLES,
        site_footer = seo_copy::SITE_FOOTER_HTML,
        kid_bg = seo_kid_background_html(),
        related_combo_links = related_combo_links,
        related_guides = related_guides,
        calc_path = PRIMARY_CALCULATOR_PATH
    )
}

fn find_pal_by_slug<'a>(pals: &'a [Pal], slug: &str) -> Option<&'a Pal> {
    pals.iter()
        .find(|pal| pal.name.to_lowercase().replace(' ', "-") == slug.to_lowercase())
}

pub(crate) fn find_pal_by_name<'a>(pals: &'a [Pal], name: &str) -> Option<&'a Pal> {
    let trimmed = name.trim();
    pals
        .iter()
        .find(|pal| pal.name.eq_ignore_ascii_case(trimmed))
}

fn calculate_child(state: &AppState, parent_a: &str, parent_b: &str) -> Option<CalculateResponse> {
    let first = find_pal_in_state(state, parent_a)?;
    let second = find_pal_in_state(state, parent_b)?;
    let fi = state.pals_by_name.get(&normalize_pal_name(&first.name))?;
    let si = state.pals_by_name.get(&normalize_pal_name(&second.name))?;

    let (child_idx, method) = child_for_pair(
        &state.pals,
        &state.pals_by_name,
        &state.power_order,
        &state.special_combos,
        *fi,
        *si,
    )?;
    let child = state.pals[child_idx].clone();
    let distance = if method.starts_with("Power average") {
        let target_power = (first.power + second.power) / 2;
        Some((child.power - target_power).abs())
    } else {
        None
    };

    Some(CalculateResponse {
        child,
        method,
        distance,
    })
}

fn combo_key(a: &str, b: &str) -> String {
    if a <= b {
        format!("{a}|{b}")
    } else {
        format!("{b}|{a}")
    }
}

fn combinations_section_html(state: &AppState, target: &str) -> String {
    let mut rows = String::new();
    let mut count = 0usize;
    for (i, first) in state.pals.iter().enumerate() {
        for second in state.pals.iter().skip(i) {
            if let Some(calc) = calculate_child(state, &first.name, &second.name) {
                if calc.child.name == target {
                    let a_slug = first.name.to_lowercase().replace(' ', "-");
                    let b_slug = second.name.to_lowercase().replace(' ', "-");
                    rows.push_str(&format!(
                        "<tr><td><a href=\"/combo/{a_slug}/{b_slug}\">{} + {}</a></td><td>{}</td></tr>",
                        first.name,
                        second.name,
                        calc.method
                    ));
                    count += 1;
                    if count >= 15 {
                        break;
                    }
                }
            }
        }
        if count >= 15 {
            break;
        }
    }
    format!(
        r#"<div class="card"><h2>Top parent pairs for {target}</h2>
        <p>Auto-generated from live breeding logic. Open any row for a dedicated combo page.</p>
        <table><thead><tr><th>Parents</th><th>Method</th></tr></thead><tbody>{rows}</tbody></table>
        <p><a href="/palworld-breeding-calculator">Open breeding calculator</a></p></div>"#
    )
}

fn legendary_pal_links_html(state: &AppState) -> String {
    let legends: Vec<_> = state
        .pals
        .iter()
        .filter(|p| p.power <= 120)
        .take(12)
        .collect();
    let links = legends
        .iter()
        .map(|p| {
            let slug = p.name.to_lowercase().replace(' ', "-");
            format!("<a href=\"/pal/{slug}\">{}</a> (power {})", p.name, p.power)
        })
        .collect::<Vec<_>>()
        .join(" • ");
    format!(
        r#"<div class="card"><h2>Legendary Pal pages</h2><p>{links}</p></div>"#
    )
}

fn build_items() -> Vec<ItemData> {
    vec![
        ItemData { item: "Wool".to_string(), source: "Lamball".to_string(), notes: "Ranch / drops".to_string() },
        ItemData { item: "Leather".to_string(), source: "Foxparks".to_string(), notes: "Frequent drop".to_string() },
        ItemData { item: "Flame Organ".to_string(), source: "Rooby".to_string(), notes: "Fire pal material".to_string() },
        ItemData { item: "Ice Organ".to_string(), source: "Frostallion".to_string(), notes: "Late-game material".to_string() },
        ItemData { item: "Electric Organ".to_string(), source: "Jolthog".to_string(), notes: "Electric crafting".to_string() },
        ItemData { item: "Ancient Civilization Parts".to_string(), source: "Alpha bosses".to_string(), notes: "Boss rewards".to_string() },
        ItemData { item: "Pal Fluids".to_string(), source: "Pengullet".to_string(), notes: "Water crafting".to_string() },
        ItemData { item: "High Quality Pal Oil".to_string(), source: "Mammorest class".to_string(), notes: "Advanced recipes".to_string() },
    ]
}

fn build_technologies() -> Vec<TechnologyData> {
    vec![
        TechnologyData { level: 2, name: "Pal Sphere".to_string(), cost: "1 Tech Point".to_string() },
        TechnologyData { level: 6, name: "Egg Incubator".to_string(), cost: "2 Tech Points".to_string() },
        TechnologyData { level: 7, name: "Breeding Farm".to_string(), cost: "2 Tech Points".to_string() },
        TechnologyData { level: 14, name: "Weapon Workbench".to_string(), cost: "2 Tech Points".to_string() },
        TechnologyData { level: 20, name: "Electric Kitchen".to_string(), cost: "3 Tech Points".to_string() },
        TechnologyData { level: 26, name: "Production Assembly Line".to_string(), cost: "3 Tech Points".to_string() },
        TechnologyData { level: 35, name: "Legendary Sphere".to_string(), cost: "4 Tech Points".to_string() },
        TechnologyData { level: 42, name: "Advanced Production Line".to_string(), cost: "4 Tech Points".to_string() },
    ]
}
