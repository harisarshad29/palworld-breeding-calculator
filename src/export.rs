//! Static export for shared hosting (Namecheap cPanel / Apache).
//!
//! Renders every page reachable from the sitemaps and internal links through the
//! real router, so the HTML is byte-for-byte what the Rust server would send.
//! Output layout:
//!   /                     -> index.html
//!   /pal/anubis           -> pal/anubis.html      (served clean by .htaccess)
//!   /sitemap.xml, robots  -> same file name
//!   /api/bootstrap etc.   -> api/....json      (served at the original URL by .htaccess)
//! The chain planner runs in the browser, so no server-side code is needed.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::{Path, PathBuf};

use axum::body::{to_bytes, Body};
use axum::http::{header, Request, StatusCode};
use axum::Router;
use tower::ServiceExt;

/// Routes that are not linked from sitemaps but must exist as files.
const EXTRA_ROUTES: &[&str] = &[
    "/",
    "/robots.txt",
    "/sitemap.xml",
    "/api/bootstrap",
    "/api/locations",
    "/breeding-calculator",
];

/// Static files and folders copied as-is.
const STATIC_FILES: &[&str] = &[
    "app.js",
    "styles.css",
    "favicon.svg",
    "manifest.webmanifest",
    "google0261c5c69eda848e.html",
    "22f4f9e126a34930a0810a0d85f0b755.txt",
];

pub async fn run(router: Router, pal_names: Vec<String>, base_url: &str, out: &str) -> Result<(), String> {
    let out = PathBuf::from(out);
    if out.exists() {
        std::fs::remove_dir_all(&out).map_err(|e| format!("cannot clear {}: {e}", out.display()))?;
    }
    std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;

    let mut queue: VecDeque<String> = EXTRA_ROUTES.iter().map(|r| r.to_string()).collect();
    for name in &pal_names {
        queue.push_back(format!("/api/combinations/{}", encode_segment(name)));
        queue.push_back(format!("/api/capture/{}", encode_segment(name)));
    }

    let mut seen: BTreeSet<String> = queue.iter().cloned().collect();
    let mut redirects: BTreeMap<String, (u16, String)> = BTreeMap::new();
    let mut missing: BTreeSet<String> = BTreeSet::new();
    let mut pages = 0usize;
    let mut bytes = 0usize;

    while let Some(path) = queue.pop_front() {
        let response = router
            .clone()
            .oneshot(Request::get(&path).body(Body::empty()).map_err(|e| e.to_string())?)
            .await
            .map_err(|e| e.to_string())?;
        let status = response.status();
        if status.is_redirection() {
            let location = response
                .headers()
                .get(header::LOCATION)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("/")
                .to_string();
            let target = strip_origin(&location, base_url);
            let target_path = target.split(['?', '#']).next().unwrap_or("").to_string();
            if target_path.starts_with('/') && seen.insert(target_path.clone()) {
                queue.push_back(target_path);
            }
            redirects.insert(path, (status.as_u16(), target));
            continue;
        }
        if status != StatusCode::OK {
            missing.insert(format!("{path} ({status})"));
            continue;
        }
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        let body = to_bytes(response.into_body(), 64 * 1024 * 1024)
            .await
            .map_err(|e| e.to_string())?;

        let is_html = content_type.starts_with("text/html");
        let is_xml = path.ends_with(".xml");
        if is_html || is_xml {
            let text = String::from_utf8_lossy(&body);
            for link in internal_links(&text, base_url) {
                if !link.starts_with("/assets/") && !link.starts_with("/api/") && seen.insert(link.clone()) {
                    queue.push_back(link);
                }
            }
        }

        let file = out.join(file_for(&path, is_html));
        if let Some(parent) = file.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(&file, &body).map_err(|e| format!("{}: {e}", file.display()))?;
        pages += 1;
        bytes += body.len();
    }

    for name in STATIC_FILES {
        if Path::new(name).exists() {
            std::fs::copy(name, out.join(name)).map_err(|e| format!("{name}: {e}"))?;
        }
    }
    copy_dir(Path::new("assets"), &out.join("assets"))?;

    std::fs::write(out.join("404.html"), not_found_html()).map_err(|e| e.to_string())?;
    std::fs::write(out.join(".htaccess"), htaccess(base_url, &redirects)).map_err(|e| e.to_string())?;
    // The same output also runs on Netlify (free static hosting): _redirects/_headers are
    // ignored by Apache, .htaccess is ignored by Netlify.
    std::fs::write(out.join("_redirects"), netlify_redirects(base_url, &redirects)).map_err(|e| e.to_string())?;
    std::fs::write(out.join("_headers"), netlify_headers()).map_err(|e| e.to_string())?;

    println!(
        "Exported {pages} files ({:.1} MB) and {} redirects to {}",
        bytes as f64 / 1_048_576.0,
        redirects.len(),
        out.display()
    );
    if !missing.is_empty() {
        println!("Linked but not exported ({}):", missing.len());
        for item in missing.iter().take(30) {
            println!("  {item}");
        }
    }
    Ok(())
}

fn encode_segment(name: &str) -> String {
    let mut out = String::new();
    for byte in name.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(byte as char),
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

fn decode_path(path: &str) -> String {
    let bytes = path.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(v) = u8::from_str_radix(&path[i + 1..i + 3], 16) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn strip_origin(url: &str, base_url: &str) -> String {
    url.strip_prefix(base_url).unwrap_or(url).to_string()
}

/// Root-relative links in href/src/<loc>, without query string or fragment.
fn internal_links(text: &str, base_url: &str) -> Vec<String> {
    let mut links = Vec::new();
    let candidates = text
        .split("href=\"")
        .skip(1)
        .chain(text.split("<loc>").skip(1))
        .filter_map(|chunk| chunk.split(['"', '<']).next());
    for raw in candidates {
        let raw = raw.replace("&amp;", "&");
        let path = strip_origin(&raw, base_url);
        if !path.starts_with('/') || path.starts_with("//") {
            continue;
        }
        let path = path.split(['?', '#']).next().unwrap_or("").to_string();
        if !path.is_empty() {
            links.push(path);
        }
    }
    links
}

/// Output file for a route. HTML pages get `.html`; everything else keeps its name.
fn file_for(path: &str, is_html: bool) -> PathBuf {
    let decoded = decode_path(path);
    let trimmed = decoded.trim_start_matches('/').trim_end_matches('/');
    if trimmed.is_empty() {
        return PathBuf::from("index.html");
    }
    if is_html {
        PathBuf::from(format!("{trimmed}.html"))
    } else if trimmed.starts_with("api/") {
        // A real extension gives the right Content-Type on Apache and LiteSpeed alike.
        PathBuf::from(format!("{trimmed}.json"))
    } else {
        PathBuf::from(trimmed)
    }
}

fn copy_dir(from: &Path, to: &Path) -> Result<(), String> {
    std::fs::create_dir_all(to).map_err(|e| e.to_string())?;
    for entry in std::fs::read_dir(from).map_err(|e| format!("{}: {e}", from.display()))? {
        let entry = entry.map_err(|e| e.to_string())?;
        let target = to.join(entry.file_name());
        if entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), &target).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn not_found_html() -> &'static str {
    r#"<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><meta name="robots" content="noindex"><title>Page not found | Palworld Breeding Calculator</title><link rel="stylesheet" href="/styles.css"></head><body><main class="container"><h1>Page not found</h1><p>This page does not exist. Try the <a href="/palworld-breeding-calculator">breeding calculator</a>, <a href="/pal-pages">all Pal pages</a> or the <a href="/">home page</a>.</p></main></body></html>"#
}

fn netlify_redirects(base_url: &str, redirects: &BTreeMap<String, (u16, String)>) -> String {
    let host = base_url
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_end_matches('/');
    let apex = host.trim_start_matches("www.");
    let mut out = String::from("# Generated by `palworld-breeding-calculator export` — do not edit by hand.\n");
    if apex != host {
        out.push_str(&format!("https://{apex}/* https://{host}/:splat 301!\n"));
    }
    for (from, (status, to)) in redirects {
        let code = if *status == 301 || *status == 308 { 301 } else { 302 };
        let to = if to.starts_with('/') { format!("https://{host}{to}") } else { to.clone() };
        out.push_str(&format!("{from} {to} {code}!\n"));
    }
    // Existing files always win over the rules below (no `!`), like the Apache setup.
    out.push_str("/api/*  /api/:splat.json  200\n");
    out.push_str("/combo/:a/:b  /palworld-breeding-calculator?parentA=:a&parentB=:b  302\n");
    out
}

fn netlify_headers() -> &'static str {
    "/assets/*\n  Cache-Control: public, max-age=2592000\n/*.css\n  Cache-Control: public, max-age=3600\n/*.js\n  Cache-Control: public, max-age=3600\n/api/*\n  Cache-Control: public, max-age=3600\n"
}

fn htaccess(base_url: &str, redirects: &BTreeMap<String, (u16, String)>) -> String {
    let host = base_url
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_end_matches('/');
    let apex = host.trim_start_matches("www.");
    let mut rules = String::new();
    for (from, (status, to)) in redirects {
        let pattern = regex_escape(from.trim_start_matches('/'));
        let code = if *status == 301 || *status == 308 { 301 } else { 302 };
        // Absolute targets so a redirect never lands on http:// or the apex first.
        let to = if to.starts_with('/') { format!("https://{host}{to}") } else { to.clone() };
        rules.push_str(&format!("RewriteRule ^{pattern}/?$ {to} [R={code},L]\n"));
    }
    format!(
        r#"# Generated by `palworld-breeding-calculator export` — do not edit by hand.
Options -Indexes -MultiViews
DirectoryIndex index.html
ErrorDocument 404 /404.html

<IfModule mod_rewrite.c>
RewriteEngine On

# One canonical origin: https://{host}
RewriteCond %{{HTTPS}} off [OR]
RewriteCond %{{HTTP_HOST}} ^{apex_re}$ [NC]
RewriteRule ^(.*)$ https://{host}/$1 [R=301,L]

# Old .html and trailing-slash URLs -> clean URL
RewriteCond %{{THE_REQUEST}} \s/+([^?\s]+?)\.html[\s?] [NC]
RewriteCond %1 !^google
RewriteRule ^ https://{host}/%1 [R=301,L,NE]
RewriteCond %{{REQUEST_FILENAME}} !-d
RewriteRule ^(.+)/$ https://{host}/$1 [R=301,L]

# Server redirects carried over from the Rust app
{rules}
# Calculator data: /api/bootstrap -> api/bootstrap.json
RewriteCond %{{DOCUMENT_ROOT}}/$1.json -f
RewriteRule ^(api/.+)$ $1.json [L]

# Clean URLs: /pal/anubis -> pal/anubis.html
RewriteCond %{{REQUEST_FILENAME}} !-f
RewriteCond %{{DOCUMENT_ROOT}}/$1.html -f
RewriteRule ^(.+)$ $1.html [L]

# Any other parent pair (44,850 exist; ~11k are pre-rendered) opens in the calculator
RewriteCond %{{REQUEST_FILENAME}} !-f
RewriteRule ^combo/([^/.]+)/([^/.]+)$ https://{host}/palworld-breeding-calculator?parentA=$1&parentB=$2 [R=302,L]
</IfModule>

<IfModule mod_deflate.c>
AddOutputFilterByType DEFLATE text/html text/css application/javascript application/json image/svg+xml application/xml text/plain
</IfModule>

<IfModule mod_expires.c>
ExpiresActive On
ExpiresByType image/webp "access plus 30 days"
ExpiresByType image/svg+xml "access plus 7 days"
ExpiresByType text/css "access plus 1 hour"
ExpiresByType application/javascript "access plus 1 hour"
ExpiresByType application/json "access plus 1 hour"
ExpiresByType text/html "access plus 0 seconds"
</IfModule>
"#,
        host = host,
        apex_re = regex_escape(apex),
        rules = rules
    )
}

fn regex_escape(text: &str) -> String {
    let mut out = String::new();
    for c in text.chars() {
        if ".+*?^$()[]{}|\\".contains(c) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_routes_to_files() {
        assert_eq!(file_for("/", true), PathBuf::from("index.html"));
        assert_eq!(file_for("/pal/anubis", true), PathBuf::from("pal/anubis.html"));
        assert_eq!(file_for("/sitemap.xml", false), PathBuf::from("sitemap.xml"));
        assert_eq!(
            file_for("/api/combinations/Frostallion%20Noct", false),
            PathBuf::from("api/combinations/Frostallion Noct.json")
        );
    }

    #[test]
    fn extracts_internal_links_only() {
        let html = r#"<a href="/pal/anubis?x=1">a</a><a href="https://example.com/x">b</a><a href="//cdn/x">c</a><loc>https://site.test/pals</loc>"#;
        assert_eq!(internal_links(html, "https://site.test"), vec!["/pal/anubis", "/pals"]);
    }
}
