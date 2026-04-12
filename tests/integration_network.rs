// Integration tests that require network access.
// Run with: cargo test --test integration_network -- --ignored --nocapture
//
// Each test seeds bo with a temp output dir, exercises the binary, then razes.
// Tests are sequential (all #[ignore]) — do not run in parallel.

// ── test URLs ────────────────────────────────────────────────────────────────

/// Standard articles — happy path
const ARTICLE_WIKIPEDIA_1: &str = "https://en.wikipedia.org/wiki/Jaya_Sri_Maha_Bodhi";
const ARTICLE_WIKIPEDIA_2: &str = "https://en.wikipedia.org/wiki/Rust_(programming_language)";
const ARTICLE_BLOG: &str = "https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/";

/// Link-heavy page
const LINK_HEAVY: &str = "https://en.wikipedia.org/wiki/Hyperlink";

/// Very long page (100KB+ body)
const VERY_LONG: &str = "https://en.wikipedia.org/wiki/United_States";

/// Slug collision pair — two pages that will produce similar slugs
const SLUG_COLLISION_1: &str = "https://en.wikipedia.org/wiki/Introduction";
const SLUG_COLLISION_2: &str = "https://en.wiktionary.org/wiki/introduction";

/// Near-duplicate URLs (same base, different query params)
const NEAR_DUP_BASE: &str = "https://en.wikipedia.org/wiki/Rust_(programming_language)";
const NEAR_DUP_VARIANT: &str =
    "https://en.wikipedia.org/wiki/Rust_(programming_language)?ref=twitter";

/// Paywalled / auth-gated
const PAYWALLED: &str = "https://www.wsj.com/articles/some-premium-article-that-requires-login";

/// JS-rendered SPA (React app)
const JS_SPA: &str = "https://react.dev/learn";

/// Dead URLs
const DEAD_404: &str = "https://httpbin.org/status/404";
const DEAD_500: &str = "https://httpbin.org/status/500";

/// Non-HTML content
const NON_HTML_PDF: &str =
    "https://www.w3.org/WAI/ER/tests/xhtml/testfiles/resources/pdf/dummy.pdf";
const NON_HTML_BINARY: &str = "https://httpbin.org/bytes/1024";

use std::fs;
use std::path::Path;
use std::process::{Command, Output};
use tempfile::TempDir;

fn bo() -> Command {
    Command::new(env!("CARGO_BIN_EXE_bo"))
}

fn seed(dir: &Path) {
    let out = bo()
        .args(["seed", dir.to_str().unwrap()])
        .output()
        .expect("failed to run bo seed");
    assert!(
        out.status.success(),
        "seed failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn add(url: &str) -> Output {
    bo().args(["add", url])
        .output()
        .expect("failed to run bo add")
}

fn raze() {
    let _ = bo().arg("raze").output();
}

// ── tests ────────────────────────────────────────────────────────────────────

#[test]
#[ignore]
fn network_happy_path_wikipedia() {
    let dir = TempDir::new().unwrap();
    seed(dir.path());

    let out = add(ARTICLE_WIKIPEDIA_2);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // At least one .md file exists
    let md_files: Vec<_> = fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .collect();
    assert_eq!(md_files.len(), 1);

    // Ledger has one entry
    let ledger = fs::read_to_string(dir.path().join("ledger.jsonl")).unwrap();
    assert_eq!(ledger.lines().count(), 1);

    raze();
}

#[test]
#[ignore]
fn network_happy_path_blog() {
    let dir = TempDir::new().unwrap();
    seed(dir.path());

    let out = add(ARTICLE_BLOG);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    raze();
}

#[test]
#[ignore]
fn network_very_long_page() {
    let dir = TempDir::new().unwrap();
    seed(dir.path());

    let out = add(VERY_LONG);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let md_files: Vec<_> = fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .collect();
    let content = fs::read_to_string(md_files[0].path()).unwrap();
    assert!(
        content.len() > 50_000,
        "expected large file, got {} bytes",
        content.len()
    );

    raze();
}

#[test]
#[ignore]
fn network_404_fails_gracefully() {
    let dir = TempDir::new().unwrap();
    seed(dir.path());

    let out = add(DEAD_404);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("404"), "stderr: {stderr}");

    // No markdown file, no ledger
    let md_files: Vec<_> = fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .collect();
    assert!(md_files.is_empty());
    assert!(!dir.path().join("ledger.jsonl").exists());

    raze();
}

#[test]
#[ignore]
fn network_pdf_fails_gracefully() {
    let dir = TempDir::new().unwrap();
    seed(dir.path());

    let out = add(NON_HTML_PDF);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("not HTML"), "stderr: {stderr}");

    raze();
}

#[test]
#[ignore]
fn network_duplicate_rejected() {
    let dir = TempDir::new().unwrap();
    seed(dir.path());

    let out1 = add(ARTICLE_WIKIPEDIA_2);
    assert!(out1.status.success());

    let out2 = add(ARTICLE_WIKIPEDIA_2);
    assert!(!out2.status.success());
    let stderr = String::from_utf8_lossy(&out2.stderr);
    assert!(stderr.contains("already stashed"), "stderr: {stderr}");

    raze();
}

#[test]
#[ignore]
fn network_near_duplicate_urls_both_stored() {
    let dir = TempDir::new().unwrap();
    seed(dir.path());

    let out1 = add(NEAR_DUP_BASE);
    assert!(out1.status.success());

    let out2 = add(NEAR_DUP_VARIANT);
    assert!(out2.status.success());

    let ledger = fs::read_to_string(dir.path().join("ledger.jsonl")).unwrap();
    assert_eq!(ledger.lines().count(), 2);

    raze();
}

#[test]
#[ignore]
fn network_link_heavy_page() {
    let dir = TempDir::new().unwrap();
    seed(dir.path());

    let out = add(LINK_HEAVY);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let md_file = fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .find(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .unwrap();
    let content = fs::read_to_string(md_file.path()).unwrap();
    assert!(
        !content.contains("](http"),
        "output still contains markdown links"
    );

    raze();
}

#[test]
#[ignore]
fn network_slug_collision_pair() {
    let dir = TempDir::new().unwrap();
    seed(dir.path());

    let out1 = add(SLUG_COLLISION_1);
    assert!(
        out1.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out1.stderr)
    );

    let out2 = add(SLUG_COLLISION_2);
    assert!(
        out2.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out2.stderr)
    );

    let ledger = fs::read_to_string(dir.path().join("ledger.jsonl")).unwrap();
    assert_eq!(ledger.lines().count(), 2);

    let md_files: Vec<_> = fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .collect();
    assert_eq!(md_files.len(), 2);

    raze();
}

#[test]
#[ignore]
fn network_500_retries_then_fails() {
    let dir = TempDir::new().unwrap();
    seed(dir.path());

    let out = add(DEAD_500);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("500") || stderr.contains("retry"),
        "stderr: {stderr}"
    );

    raze();
}

#[test]
#[ignore]
fn network_binary_content_fails() {
    let dir = TempDir::new().unwrap();
    seed(dir.path());

    let out = add(NON_HTML_BINARY);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("not HTML"), "stderr: {stderr}");

    raze();
}

#[test]
#[ignore]
fn network_paywalled_degrades_gracefully() {
    let dir = TempDir::new().unwrap();
    seed(dir.path());

    let out = add(PAYWALLED);
    // May succeed with partial content or fail — either is acceptable
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!stderr.contains("panic"), "binary panicked: {stderr}");

    raze();
}

#[test]
#[ignore]
fn network_js_spa_degrades_gracefully() {
    let dir = TempDir::new().unwrap();
    seed(dir.path());

    let out = add(JS_SPA);
    // SPA may return some server-rendered content or fail extraction
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!stderr.contains("panic"), "binary panicked: {stderr}");

    raze();
}
