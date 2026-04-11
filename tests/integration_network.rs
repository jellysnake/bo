// Integration tests that require network access.
// Run with: cargo test --test integration_network -- --ignored --nocapture

mod test_urls;

use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn run_binary(url: &str, output_dir: &std::path::Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_bo"))
        .arg(url)
        .arg("--output-dir")
        .arg(output_dir)
        .output()
        .expect("failed to run bo binary")
}

#[test]
#[ignore]
fn network_happy_path_wikipedia() {
    let dir = TempDir::new().unwrap();
    let out = run_binary(test_urls::ARTICLE_WIKIPEDIA, dir.path());
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
}

#[test]
#[ignore]
fn network_happy_path_blog() {
    let dir = TempDir::new().unwrap();
    let out = run_binary(test_urls::ARTICLE_BLOG, dir.path());
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
#[ignore]
fn network_very_long_page() {
    let dir = TempDir::new().unwrap();
    let out = run_binary(test_urls::VERY_LONG, dir.path());
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // File should be substantial
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
}

#[test]
#[ignore]
fn network_404_fails_gracefully() {
    let dir = TempDir::new().unwrap();
    let out = run_binary(test_urls::DEAD_404, dir.path());
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("404"), "stderr: {stderr}");

    // No markdown file
    let md_files: Vec<_> = fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .collect();
    assert!(md_files.is_empty());

    // No ledger
    assert!(!dir.path().join("ledger.jsonl").exists());
}

#[test]
#[ignore]
fn network_pdf_fails_gracefully() {
    let dir = TempDir::new().unwrap();
    let out = run_binary(test_urls::NON_HTML_PDF, dir.path());
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("not HTML"), "stderr: {stderr}");
}

#[test]
#[ignore]
fn network_duplicate_rejected() {
    let dir = TempDir::new().unwrap();

    // First attempt succeeds
    let out1 = run_binary(test_urls::ARTICLE_WIKIPEDIA, dir.path());
    assert!(out1.status.success());

    // Second attempt rejected
    let out2 = run_binary(test_urls::ARTICLE_WIKIPEDIA, dir.path());
    assert!(!out2.status.success());
    let stderr = String::from_utf8_lossy(&out2.stderr);
    assert!(stderr.contains("already stashed"), "stderr: {stderr}");
}

#[test]
#[ignore]
fn network_near_duplicate_urls_both_stored() {
    let dir = TempDir::new().unwrap();

    let out1 = run_binary(test_urls::NEAR_DUP_BASE, dir.path());
    assert!(out1.status.success());

    let out2 = run_binary(test_urls::NEAR_DUP_VARIANT, dir.path());
    assert!(out2.status.success());

    let ledger = fs::read_to_string(dir.path().join("ledger.jsonl")).unwrap();
    assert_eq!(ledger.lines().count(), 2);
}

#[test]
#[ignore]
fn network_link_heavy_page() {
    let dir = TempDir::new().unwrap();
    let out = run_binary(test_urls::LINK_HEAVY, dir.path());
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // Verify no markdown links in output
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
}

#[test]
#[ignore]
fn network_slug_collision_pair() {
    let dir = TempDir::new().unwrap();

    let out1 = run_binary(test_urls::SLUG_COLLISION_1, dir.path());
    assert!(
        out1.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out1.stderr)
    );

    let out2 = run_binary(test_urls::SLUG_COLLISION_2, dir.path());
    assert!(
        out2.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out2.stderr)
    );

    // Both stashed, two ledger entries
    let ledger = fs::read_to_string(dir.path().join("ledger.jsonl")).unwrap();
    assert_eq!(ledger.lines().count(), 2);

    // Two distinct .md files
    let md_files: Vec<_> = fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .collect();
    assert_eq!(md_files.len(), 2);
}

#[test]
#[ignore]
fn network_500_retries_then_fails() {
    let dir = TempDir::new().unwrap();
    let out = run_binary(test_urls::DEAD_500, dir.path());
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("500") || stderr.contains("retry"),
        "stderr: {stderr}"
    );
}

#[test]
#[ignore]
fn network_binary_content_fails() {
    let dir = TempDir::new().unwrap();
    let out = run_binary(test_urls::NON_HTML_BINARY, dir.path());
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("not HTML"), "stderr: {stderr}");
}

#[test]
#[ignore]
fn network_paywalled_degrades_gracefully() {
    let dir = TempDir::new().unwrap();
    let out = run_binary(test_urls::PAYWALLED, dir.path());
    // May succeed with partial content or fail — either is acceptable
    // What matters: no panic, clean exit
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!stderr.contains("panic"), "binary panicked: {stderr}");
}

#[test]
#[ignore]
fn network_js_spa_degrades_gracefully() {
    let dir = TempDir::new().unwrap();
    let out = run_binary(test_urls::JS_SPA, dir.path());
    // SPA may return some server-rendered content or fail extraction
    // What matters: no panic, clean exit
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!stderr.contains("panic"), "binary panicked: {stderr}");
}
