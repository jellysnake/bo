// Integration tests that require network access.
// Run with: cargo test --test integration_network -- --ignored --nocapture
//
// Each test seeds bo with a temp output dir, exercises the binary, then razes.
// Tests are sequential (all #[ignore]) — do not run in parallel.

mod test_urls;

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

    let out = add(test_urls::ARTICLE_WIKIPEDIA_2);
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

    let out = add(test_urls::ARTICLE_BLOG);
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

    let out = add(test_urls::VERY_LONG);
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

    let out = add(test_urls::DEAD_404);
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

    let out = add(test_urls::NON_HTML_PDF);
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

    let out1 = add(test_urls::ARTICLE_WIKIPEDIA_2);
    assert!(out1.status.success());

    let out2 = add(test_urls::ARTICLE_WIKIPEDIA_2);
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

    let out1 = add(test_urls::NEAR_DUP_BASE);
    assert!(out1.status.success());

    let out2 = add(test_urls::NEAR_DUP_VARIANT);
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

    let out = add(test_urls::LINK_HEAVY);
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

    let out1 = add(test_urls::SLUG_COLLISION_1);
    assert!(
        out1.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out1.stderr)
    );

    let out2 = add(test_urls::SLUG_COLLISION_2);
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

    let out = add(test_urls::DEAD_500);
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

    let out = add(test_urls::NON_HTML_BINARY);
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

    let out = add(test_urls::PAYWALLED);
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

    let out = add(test_urls::JS_SPA);
    // SPA may return some server-rendered content or fail extraction
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!stderr.contains("panic"), "binary panicked: {stderr}");

    raze();
}
