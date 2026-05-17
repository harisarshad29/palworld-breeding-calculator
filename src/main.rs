mod data;
mod seo_copy;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::{
    extract::{Path, State},
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
    pal_locations: HashMap<String, data::PalLocation>,
    items: Vec<ItemData>,
    technologies: Vec<TechnologyData>,
    special_combos: HashMap<String, String>,
    base_url: String,
    index_template: String,
    combinations_cache: Arc<Mutex<HashMap<String, Vec<PairResult>>>>,
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

const SEO_PAGES: [SeoPage; 10] = [
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
        body_html: "<p>The best combo is not only about rarity; it is about getting desired children with minimum farming time. Combine special pairs first, then use breeding-power proximity for fallback pairs.</p><p>Use this calculator to compare multiple parent combinations quickly and keep track of methods marked as special combinations for guaranteed outcomes.</p>",
    },
    GuidePage {
        path: "/guides/capture-rate-explained",
        title: "Palworld Capture Rate Explained - Better Catch Planning",
        description: "Understand capture-rate difficulty estimates in Palworld and plan better targets before breeding runs.",
        heading: "Capture Rate Planning for Breeding",
        body_html: "<p>Capture planning affects breeding speed. Easier-to-catch parent targets reduce setup time and help you iterate combinations faster.</p><p>In this tool, capture-rate estimates use breeding power as a practical difficulty proxy. Use these estimates to decide which parents to farm first.</p>",
    },
    GuidePage {
        path: "/fastest-anubis-breed",
        title: "Fastest Anubis Breed Route in Palworld - Parent Combos",
        description: "Find the fastest Anubis breeding routes in Palworld with parent pair tables, power logic, and calculator links.",
        heading: "Fastest Anubis Breed Routes",
        body_html: "<p>Anubis is a top-tier work and combat Pal. Use the reverse calculator below to list every parent pair that can produce Anubis, then farm the easiest parents first.</p><p>Prioritize special combination pairs when available, then compare power-average routes for fewer failed eggs.</p>",
    },
    GuidePage {
        path: "/legendary-breeding",
        title: "Legendary Pal Breeding Guide - Palworld Routes",
        description: "Legendary Pal breeding routes for Jetragon, Frostallion, Necromus, Paladius, and other rare targets in Palworld.",
        heading: "Legendary Pal Breeding Guide",
        body_html: "<p>Legendary Pals require long breeding chains. Start from easier high-power parents and move toward low breeding-power targets.</p><p>Use dedicated Pal pages and combo pages in this database to compare direct outcomes before committing resources.</p>",
    },
    GuidePage {
        path: "/best-early-game-breeding-combo",
        title: "Best Early Game Breeding Combos in Palworld",
        description: "Early game Palworld breeding combos using easy-to-catch parents and practical child outcomes for base progression.",
        heading: "Best Early Game Breeding Combos",
        body_html: "<p>Early breeding should focus on easy captures like Lamball, Cattiva, Foxparks, and Pengullet while you unlock incubators and breeding farms.</p><p>Test pairs in the calculator and avoid rare targets until your sphere and level support the capture route.</p>",
    },
    GuidePage {
        path: "/egg-incubation-guide",
        title: "Palworld Egg Incubation Guide - Hatch Time & Breeding Setup",
        description: "Palworld egg incubation basics: breeding farm unlock, incubator setup, and how incubation fits your combo plan.",
        heading: "Egg Incubation Guide",
        body_html: "<p>Unlock the Breeding Farm and Egg Incubator through technology points, then place eggs immediately to avoid workflow delays.</p><p>Pair incubation planning with parent farming routes from the map and capture tools in this database.</p>",
    },
    GuidePage {
        path: "/best-mining-pal-breeding",
        title: "Best Mining Pal Breeding - Palworld Work Pal Routes",
        description: "Find mining-focused Pal breeding routes in Palworld for base ore loops and work suitability planning.",
        heading: "Best Mining Pal Breeding",
        body_html: "<p>Mining Pals are often bred indirectly through power-average chains. Use the calculator to test parents that trend toward Digtoise, Tombat, and other mining profiles.</p><p>Compare work-focused Pal pages and capture easier parents before long legendary chains.</p>",
    },
    GuidePage {
        path: "/fastest-flying-mount-breeding",
        title: "Fastest Flying Mount Breeding - Jetragon & Legendary Routes",
        description: "Flying mount breeding routes in Palworld including Jetragon paths, parent combos, and capture planning.",
        heading: "Fastest Flying Mount Breeding",
        body_html: "<p>Jetragon and other flying legends are common endgame targets. Use reverse lookup to list valid parent pairs, then farm easier high-power parents first.</p><p>Check combo pages for direct special outcomes before running long power-average experiments.</p>",
    },
    GuidePage {
        path: "/anubis-vs-lyleen",
        title: "Anubis vs Lyleen in Palworld - Breeding & Role Comparison",
        description: "Compare Anubis vs Lyleen breeding power, roles, and combo planning in Palworld with calculator links.",
        heading: "Anubis vs Lyleen",
        body_html: "<p><strong>Anubis</strong> is an earlier-to-mid powerhouse with strong handiwork and combat utility. <strong>Lyleen</strong> is a late-game support-oriented Pal with a lower breeding power value (rarer tier).</p><p>Anubis is usually faster to breed for most players; Lyleen is a longer chain but valuable for advanced base setups. Use the calculator to compare parent routes for each target.</p>",
    },
    GuidePage {
        path: "/guides/breeding-not-working",
        title: "Palworld Breeding Not Working – Fixes & Checklist",
        description: "Fix Palworld breeding when eggs do not appear, cake runs out, or parent pairs fail. Step-by-step troubleshooting before you waste resources.",
        heading: "Breeding Not Working? Fix Checklist",
        body_html: "<p>If breeding fails in Palworld, check these items in order before changing parent pairs.</p><h2>1. Breeding Farm setup</h2><p>Confirm the Breeding Farm is built, powered, and both parent slots are filled with compatible Pals.</p><h2>2. Cake supply</h2><p>The feed box must have cake. Empty cake stops the breeding process even when parents are assigned.</p><h2>3. Valid parent pair</h2><p>Use our <a href=\"/palworld-breeding-calculator\">calculator</a> to verify the pair can produce your target child. Invalid pairs waste time.</p><h2>4. Incubator space</h2><p>Collect eggs and place them in an incubator. Full inventory or missing incubator delays progress.</p><h2>5. Pal conditions</h2><p>Sick, starving, or depressed Pals may block breeding—heal and feed parents first.</p><p>Still stuck? Compare routes on the <a href=\"/how-to-breed/anubis\">Anubis guide</a> or your target <a href=\"/pals\">Pal page</a>.</p>",
    },
];

#[tokio::main]
async fn main() {
    let index_template = std::fs::read_to_string("index.html")
        .expect("index.html must exist in project root");

    let state = AppState {
        pals: data::load_pals(),
        pal_locations: data::load_pal_locations(),
        items: build_items(),
        technologies: build_technologies(),
        special_combos: data::load_special_combos(),
        base_url: std::env::var("BASE_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:3000".to_string()),
        index_template,
        combinations_cache: Arc::new(Mutex::new(HashMap::new())),
    };
    prewarm_combo_cache(&state);
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

    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
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

fn combinations_for_target(state: &AppState, target: &str) -> Vec<PairResult> {
    let Some(pal) = find_pal_by_name(&state.pals, target) else {
        return Vec::new();
    };
    let canonical = pal.name.as_str();

    let mut cache = state
        .combinations_cache
        .lock()
        .expect("combinations cache lock poisoned");
    if let Some(cached) = cache.get(canonical) {
        return cached.clone();
    }

    let mut results = Vec::new();
    for (i, first) in state.pals.iter().enumerate() {
        for second in state.pals.iter().skip(i) {
            if let Some(calc) = calculate_child(state, &first.name, &second.name) {
                if calc.child.name == canonical {
                    results.push(PairResult {
                        a: first.name.clone(),
                        b: second.name.clone(),
                        method: calc.method,
                    });
                }
            }
        }
    }
    cache.insert(canonical.to_string(), results.clone());
    results
}

fn prewarm_combo_cache(state: &AppState) {
    const POPULAR: &[&str] = &[
        "Frostallion",
        "Anubis",
        "Jetragon",
        "Lamball",
        "Cattiva",
        "Foxparks",
        "Lyleen",
        "Necromus",
        "Paladius",
        "Penking",
        "Relaxaurus",
        "Shadowbeak",
    ];
    for name in POPULAR {
        if find_pal_by_name(&state.pals, name).is_some() {
            let _ = combinations_for_target(state, name);
        }
    }
    eprintln!("Pre-warmed breeding combinations for {} popular Pals", POPULAR.len());
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
    let mut combos = Vec::new();
    for (i, first) in state.pals.iter().enumerate() {
        for second in state.pals.iter().skip(i) {
            if let Some(calc) = calculate_child(state, &first.name, &second.name) {
                if calc.child.name == target_name {
                    combos.push((first.name.clone(), second.name.clone(), calc.method));
                }
            }
        }
    }
    combos
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
    let target = state
        .pals
        .iter()
        .find(|pal| pal.name.eq_ignore_ascii_case(&name))
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

    let mut alternative_pairs = Vec::new();
    for (i, first) in state.pals.iter().enumerate() {
        for second in state.pals.iter().skip(i) {
            if let Some(calc) = calculate_child(&state, &first.name, &second.name) {
                if calc.child.name == result.child.name {
                    alternative_pairs.push((first.name.clone(), second.name.clone(), calc.method));
                }
            }
        }
    }
    alternative_pairs.truncate(10);

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
    <meta property="og:description" content="{description}" />
    <meta property="og:url" content="{page_url}" />
    <meta property="og:image" content="{base_url}/assets/pals/anubis.webp" />
    <meta name="twitter:card" content="summary_large_image" />
    <meta name="twitter:title" content="{title}" />
    <meta name="twitter:description" content="{description}" />
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
        description = page.meta_description,
        app_description = page.page_description,
        page_url = page_url
    )
}

fn query_escape(s: &str) -> String {
    s.replace(' ', "%20").replace('&', "%26")
}

fn combo_table_rows(combos: &[(String, String, String)], limit: usize) -> String {
    combos
        .iter()
        .take(limit)
        .map(|(a, b, method)| {
            let a_slug = pal_slug(a);
            let b_slug = pal_slug(b);
            format!(
                "<tr><td><a href=\"/combo/{a_slug}/{b_slug}\">{a} + {b}</a></td><td>{method}</td></tr>"
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
      th, td {{ border: 1px solid #33465a; padding: 0.45rem 0.52rem; text-align: left; font-size: 0.9rem; }}
      th {{ background: #182230; }}
    </style>
    <script type="application/ld+json">
      {{"@context":"https://schema.org","@type":"HowTo","name":"How to breed {pal_name} in Palworld","description":"{description}","step":[{{"@type":"HowToStep","name":"Unlock Breeding Farm","text":"Research and build a Breeding Farm and Egg Incubator with cake in the feed box."}},{{"@type":"HowToStep","name":"Find parent pairs","text":"Use the reverse calculator to list valid parents for {pal_name}."}},{{"@type":"HowToStep","name":"Breed and incubate","text":"Assign parents, collect the egg, and incubate until {pal_name} hatches."}}]}}
    </script>
    <script type="application/ld+json">
      {{"@context":"https://schema.org","@type":"FAQPage","mainEntity":[{{"@type":"Question","name":"How do you breed {pal_name}?","acceptedAnswer":{{"@type":"Answer","text":"Use valid parent pairs listed on this page or the reverse calculator. Special combinations override power averages when available."}}}},{{"@type":"Question","name":"How many combos produce {pal_name}?","acceptedAnswer":{{"@type":"Answer","text":"This database lists {combo_count} parent pair routes that can produce {pal_name}."}}}}]}}
    </script>
  </head>
  <body>
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
      th, td {{ border: 1px solid #33465a; padding: 0.45rem 0.52rem; text-align: left; font-size: 0.9rem; }}
      th {{ background: #182230; }}
    </style>
    <script type="application/ld+json">
      {{"@context":"https://schema.org","@type":"ItemList","name":"{pal_name} breeding combinations","numberOfItems":{combo_count},"itemListElement":[]}}
    </script>
  </head>
  <body>
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
  </body>
</html>"#
        ,
        title = page.title,
        description = page.description,
        heading = page.heading,
        body_html = page.body_html,
        page_url = page_url
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
        "{} Breeding Guide – Best Combos, Parents & Location | Palworld",
        target.name
    );
    let description = format!(
        "Find every {} breeding combination in Palworld: parent pairs, reverse lookup routes, map location, capture tips, and legendary combo paths instantly.",
        target.name
    );
    let combo_rows = combos
        .iter()
        .map(|(a, b, method)| {
            let a_slug = a.to_lowercase().replace(' ', "-");
            let b_slug = b.to_lowercase().replace(' ', "-");
            format!(
                "<tr><td><a href=\"/combo/{a_slug}/{b_slug}\">{a}</a></td><td><a href=\"/combo/{a_slug}/{b_slug}\">{b}</a></td><td>{method}</td></tr>"
            )
        })
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
      th, td {{ border: 1px solid #33465a; padding: 0.45rem 0.52rem; text-align: left; font-size: 0.9rem; }}
      th {{ background: #182230; }}
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
    <main class="wrap">
      <div class="crumbs"><a href="/">Home</a> / <a href="/pals">Pals</a> / {pal_name}</div>
      <div class="card">
        <h1>{pal_name} in Palworld</h1>
        <p><strong>Breeding Power:</strong> {power}</p>
        <p><strong>Map Area:</strong> {area} | <strong>Coordinates:</strong> {coords}</p>
        <p>{dynamic_summary}</p>
        <p><a href="/how-to-breed/{slug}">How to breed {pal_name}</a> • <a href="/combos/{slug}">All {pal_name} combos ({combo_count})</a> • <a href="{calc_link}">Reverse calculator</a> • <a href="/palworld-breeding-calculator">Breeding tool</a></p>
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
        calc_link = calc_link
    )
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
        .map(|(a, b, method)| {
            let a_slug = a.to_lowercase().replace(' ', "-");
            let b_slug = b.to_lowercase().replace(' ', "-");
            format!(
                "<tr><td><a href=\"/combo/{a_slug}/{b_slug}\">{a}</a></td><td><a href=\"/combo/{a_slug}/{b_slug}\">{b}</a></td><td>{method}</td></tr>"
            )
        })
        .collect::<Vec<_>>()
        .join("");
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
      th, td {{ border: 1px solid #33465a; padding: 0.45rem 0.52rem; text-align: left; font-size: 0.9rem; }}
      th {{ background: #182230; }}
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
    <main class="wrap">
      <div class="crumbs"><a href="/">Home</a> / <a href="{calc_path}">Breeding Calculator</a> / {parent_a_name} + {parent_b_name}</div>
      <div class="card">
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
        related_combo_links = related_combo_links,
        related_guides = related_guides,
        calc_path = PRIMARY_CALCULATOR_PATH
    )
}

fn find_pal_by_slug<'a>(pals: &'a [Pal], slug: &str) -> Option<&'a Pal> {
    pals.iter()
        .find(|pal| pal.name.to_lowercase().replace(' ', "-") == slug.to_lowercase())
}

fn find_pal_by_name<'a>(pals: &'a [Pal], name: &str) -> Option<&'a Pal> {
    let trimmed = name.trim();
    pals
        .iter()
        .find(|pal| pal.name.eq_ignore_ascii_case(trimmed))
}

fn calculate_child(state: &AppState, parent_a: &str, parent_b: &str) -> Option<CalculateResponse> {
    let first = find_pal_by_name(&state.pals, parent_a)?;
    let second = find_pal_by_name(&state.pals, parent_b)?;

    let key = combo_key(&first.name, &second.name);
    if let Some(exact_child_name) = state.special_combos.get(&key) {
        let child = find_pal_by_name(&state.pals, exact_child_name)?;
        return Some(CalculateResponse {
            child: child.clone(),
            method: "Special combination".to_string(),
            distance: None,
        });
    }
    let target_power = (first.power + second.power) / 2;

    let mut nearest = state.pals.first()?.clone();
    let mut best_distance = (nearest.power - target_power).abs();

    for pal in &state.pals {
        let distance = (pal.power - target_power).abs();
        if distance < best_distance {
            best_distance = distance;
            nearest = pal.clone();
        }
    }

    Some(CalculateResponse {
        child: nearest,
        method: format!("Power average ({target_power})"),
        distance: Some(best_distance),
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
