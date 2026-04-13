// End-to-end integration tests using fixtures (no network).
//
// Tests exercise the full extract → write → index pipeline by calling
// `bo::add::collect_html` directly with fixture HTML. This avoids network
// dependencies while covering the same code paths as `bo add <url>`.

use std::fs;
use tempfile::TempDir;

const SAMPLE_HTML: &str = r#"
<html><head><title>Test Article</title></head>
<body><article>
<h1>Test Article</h1>
<p>This is a test article with substantial content that exceeds the minimum threshold for content extraction quality checks.</p>
<h2>Section One</h2>
<p>More detailed content about the first section of this test article, providing enough text for a meaningful extraction.</p>
</article></body></html>
"#;

const COLLISION_HTML_1: &str = r#"
<html><head><title>Introduction</title></head>
<body><article>
<h1>Introduction</h1>
<p>This is the first introduction page with enough content to pass the extraction threshold for quality filtering.</p>
</article></body></html>
"#;

const COLLISION_HTML_2: &str = r#"
<html><head><title>Introduction</title></head>
<body><article>
<h1>Introduction</h1>
<p>This is the second introduction page, from a completely different source, also with enough content for extraction.</p>
</article></body></html>
"#;

#[test]
fn full_pipeline_happy_path() {
    let dir = TempDir::new().unwrap();
    let page =
        bo::add::collect_html("https://example.com/article", SAMPLE_HTML, dir.path()).unwrap();

    // Markdown file exists
    assert!(dir.path().join(&page.filename).exists());

    // Content is correct
    let content = fs::read_to_string(dir.path().join(&page.filename)).unwrap();
    assert!(content.contains("title: \"Test Article\""));
    assert!(content.contains("url: https://example.com/article"));
    assert!(content.contains("collected_at:"));
    assert!(content.contains("updated_at:"));
    assert!(!content.contains("fetched:"));
    assert!(content.contains("# Test Article"));
    assert!(content.contains("Section One"));

    // Index has one entry
    let entries = bo::index::read_index(&dir.path().join("index.jsonl")).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].url, "https://example.com/article");
}

#[test]
fn duplicate_rejected() {
    let dir = TempDir::new().unwrap();
    bo::add::collect_html("https://example.com/article", SAMPLE_HTML, dir.path()).unwrap();

    // Second attempt with same URL should fail
    let result = bo::add::collect_html("https://example.com/article", SAMPLE_HTML, dir.path());
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("already collected"));

    // Index still has only one entry
    let entries = bo::index::read_index(&dir.path().join("index.jsonl")).unwrap();
    assert_eq!(entries.len(), 1);
}

#[test]
fn slug_collision_disambiguated() {
    let dir = TempDir::new().unwrap();

    let page1 =
        bo::add::collect_html("https://example.com/intro1", COLLISION_HTML_1, dir.path())
            .unwrap();
    let page2 =
        bo::add::collect_html("https://example.com/intro2", COLLISION_HTML_2, dir.path())
            .unwrap();

    // Both files exist
    assert!(dir.path().join(&page1.filename).exists());
    assert!(dir.path().join(&page2.filename).exists());

    // Filenames are different
    assert_ne!(page1.filename, page2.filename);

    // Both start with "introduction"
    assert!(page1.filename.starts_with("introduction"));
    assert!(page2.filename.starts_with("introduction"));

    // Second has hash suffix
    assert!(
        page2.filename.contains('-') && page2.filename.len() > page1.filename.len(),
        "second file should have hash suffix: {} vs {}",
        page1.filename,
        page2.filename
    );

    // Index has two entries
    let entries = bo::index::read_index(&dir.path().join("index.jsonl")).unwrap();
    assert_eq!(entries.len(), 2);
}

#[test]
fn empty_extraction_no_artifacts() {
    let dir = TempDir::new().unwrap();
    let empty_html = "<html><body></body></html>";

    let result = bo::add::collect_html("https://example.com/empty", empty_html, dir.path());
    assert!(result.is_err());

    // No markdown file
    let md_files: Vec<_> = fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .collect();
    assert!(md_files.is_empty());

    // No index entry
    let index_path = dir.path().join("index.jsonl");
    assert!(!index_path.exists() || fs::read_to_string(&index_path).unwrap().is_empty());
}

#[test]
fn failed_url_can_be_resubmitted() {
    let dir = TempDir::new().unwrap();
    let empty_html = "<html><body></body></html>";

    // First attempt fails (empty content)
    let result = bo::add::collect_html("https://example.com/flaky", empty_html, dir.path());
    assert!(result.is_err());

    // Second attempt with good content succeeds — not blocked by index
    let result = bo::add::collect_html("https://example.com/flaky", SAMPLE_HTML, dir.path());
    assert!(result.is_ok());
}

#[test]
fn near_duplicate_urls_both_stored() {
    let dir = TempDir::new().unwrap();

    bo::add::collect_html("https://example.com/article", SAMPLE_HTML, dir.path()).unwrap();
    bo::add::collect_html(
        "https://example.com/article?ref=twitter",
        SAMPLE_HTML,
        dir.path(),
    )
    .unwrap();

    let entries = bo::index::read_index(&dir.path().join("index.jsonl")).unwrap();
    assert_eq!(entries.len(), 2);
}
