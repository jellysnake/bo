// End-to-end integration tests using fixtures (no network)

use std::fs;
use tempfile::TempDir;

// We test the full pipeline by calling the modules directly rather than the binary,
// to avoid needing network access. This exercises the same code paths as main().

fn run_pipeline(url: &str, html: &str, output_dir: &std::path::Path) -> Result<String, String> {
    let ledger_path = output_dir.join("ledger.jsonl");

    // Check duplicate
    let entries = link_stash::ledger::read_ledger(&ledger_path).map_err(|e| e.to_string())?;
    if let Some(existing) = link_stash::ledger::is_duplicate(&entries, url) {
        return Err(format!("already stashed: {} → {}", url, existing.file));
    }

    // Extract
    let content = link_stash::extract::extract_content(html).map_err(|e| e.to_string())?;

    // Slug
    let title_ref = content.title.as_deref().unwrap_or("");
    let base_slug = link_stash::slug::slugify(title_ref, url);
    let filename = link_stash::slug::resolve_slug(&base_slug, url, output_dir);

    // Format + write
    let now: chrono::DateTime<chrono::Utc> = "2025-01-15T09:32:00Z".parse().unwrap();
    let now_str = now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let doc = link_stash::markdown::format_document(
        content.title.as_deref(),
        url,
        &now_str,
        &content.body_markdown,
    );
    link_stash::markdown::write_document(output_dir, &filename, &doc).map_err(|e| e.to_string())?;

    // Ledger
    let entry = link_stash::ledger::LedgerEntry {
        url: url.to_string(),
        fetched_at: now,
        file: format!("{}.md", filename),
    };
    link_stash::ledger::append_entry(&ledger_path, &entry).map_err(|e| e.to_string())?;

    Ok(format!("{}.md", filename))
}

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
    let file = run_pipeline("https://example.com/article", SAMPLE_HTML, dir.path()).unwrap();

    // Markdown file exists
    assert!(dir.path().join(&file).exists());

    // Content is correct
    let content = fs::read_to_string(dir.path().join(&file)).unwrap();
    assert!(content.contains("title: \"Test Article\""));
    assert!(content.contains("url: https://example.com/article"));
    assert!(content.contains("# Test Article"));
    assert!(content.contains("Section One"));

    // Ledger has one entry
    let entries = link_stash::ledger::read_ledger(&dir.path().join("ledger.jsonl")).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].url, "https://example.com/article");
}

#[test]
fn duplicate_rejected() {
    let dir = TempDir::new().unwrap();
    run_pipeline("https://example.com/article", SAMPLE_HTML, dir.path()).unwrap();

    // Second attempt with same URL should fail
    let result = run_pipeline("https://example.com/article", SAMPLE_HTML, dir.path());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("already stashed"));

    // Ledger still has only one entry
    let entries = link_stash::ledger::read_ledger(&dir.path().join("ledger.jsonl")).unwrap();
    assert_eq!(entries.len(), 1);
}

#[test]
fn slug_collision_disambiguated() {
    let dir = TempDir::new().unwrap();

    let file1 = run_pipeline("https://example.com/intro1", COLLISION_HTML_1, dir.path()).unwrap();
    let file2 = run_pipeline("https://example.com/intro2", COLLISION_HTML_2, dir.path()).unwrap();

    // Both files exist
    assert!(dir.path().join(&file1).exists());
    assert!(dir.path().join(&file2).exists());

    // Filenames are different
    assert_ne!(file1, file2);

    // Both start with "introduction"
    assert!(file1.starts_with("introduction"));
    assert!(file2.starts_with("introduction"));

    // Second has hash suffix
    assert!(
        file2.contains('-') && file2.len() > file1.len(),
        "second file should have hash suffix: {file1} vs {file2}"
    );

    // Ledger has two entries
    let entries = link_stash::ledger::read_ledger(&dir.path().join("ledger.jsonl")).unwrap();
    assert_eq!(entries.len(), 2);
}

#[test]
fn empty_extraction_no_artifacts() {
    let dir = TempDir::new().unwrap();
    let empty_html = "<html><body></body></html>";

    let result = run_pipeline("https://example.com/empty", empty_html, dir.path());
    assert!(result.is_err());

    // No markdown file
    let md_files: Vec<_> = fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .collect();
    assert!(md_files.is_empty());

    // No ledger entry
    let ledger_path = dir.path().join("ledger.jsonl");
    assert!(!ledger_path.exists() || fs::read_to_string(&ledger_path).unwrap().is_empty());
}

#[test]
fn failed_url_can_be_resubmitted() {
    let dir = TempDir::new().unwrap();
    let empty_html = "<html><body></body></html>";

    // First attempt fails (empty content)
    let result = run_pipeline("https://example.com/flaky", empty_html, dir.path());
    assert!(result.is_err());

    // Second attempt with good content succeeds — not blocked by ledger
    let result = run_pipeline("https://example.com/flaky", SAMPLE_HTML, dir.path());
    assert!(result.is_ok());
}

#[test]
fn near_duplicate_urls_both_stored() {
    let dir = TempDir::new().unwrap();

    let file1 = run_pipeline("https://example.com/article", SAMPLE_HTML, dir.path()).unwrap();
    let file2 = run_pipeline(
        "https://example.com/article?ref=twitter",
        SAMPLE_HTML,
        dir.path(),
    )
    .unwrap();

    assert!(dir.path().join(&file1).exists());
    assert!(dir.path().join(&file2).exists());

    let entries = link_stash::ledger::read_ledger(&dir.path().join("ledger.jsonl")).unwrap();
    assert_eq!(entries.len(), 2);
}
