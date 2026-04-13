// Integration tests for `bo compile`.
//
// Tests that require a live OpenAI API key are marked `#[ignore]` so they do
// not run in CI without credentials.  Run them explicitly with:
//
//   OPENAI_API_KEY=sk-... cargo test --test integration_compile -- --ignored

use std::fs;
use std::path::PathBuf;

use bo::compile;
use bo::config::Config;
use bo::index;

/// Copy the fixture collection into a temp directory and return the path.
fn setup_fixture_collection() -> tempfile::TempDir {
    let dir = tempfile::TempDir::new().unwrap();
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/compile");

    for entry in fs::read_dir(&fixtures).unwrap() {
        let entry = entry.unwrap();
        let dest = dir.path().join(entry.file_name());
        fs::copy(entry.path(), &dest).unwrap();
    }

    dir
}

fn make_config(output_dir: &std::path::Path) -> Config {
    Config {
        output_dir: output_dir.to_path_buf(),
        compile_model: Some("gpt-4o-mini".to_string()), // cheaper model for tests
    }
}

// ── live API tests (require OPENAI_API_KEY) ───────────────────────────────────

#[test]
#[ignore = "requires OPENAI_API_KEY"]
fn compile_creates_branches_directory() {
    let dir = setup_fixture_collection();
    let cfg = make_config(dir.path());

    let result = compile::cmd_compile(&cfg);
    assert!(result.is_ok(), "compile failed: {:?}", result.err());

    assert!(
        dir.path().join("branches").exists(),
        "branches/ directory was not created"
    );
}

#[test]
#[ignore = "requires OPENAI_API_KEY"]
fn compile_produces_at_least_one_branch_file() {
    let dir = setup_fixture_collection();
    let cfg = make_config(dir.path());

    compile::cmd_compile(&cfg).unwrap();

    let branches_dir = dir.path().join("branches");
    let branch_files: Vec<_> = fs::read_dir(&branches_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
        .collect();

    assert!(
        !branch_files.is_empty(),
        "no branch files were written to branches/"
    );

    // Validate the first branch file has correct frontmatter
    let first_path = branch_files[0].path();
    let content = fs::read_to_string(&first_path).unwrap();
    let (mapping, body) = bo::frontmatter::parse(&content).unwrap();
    assert!(
        mapping.get("title").and_then(|v| v.as_str()).is_some(),
        "branch missing 'title' in frontmatter"
    );
    assert!(
        mapping.get("compiled_at").and_then(|v| v.as_str()).is_some(),
        "branch missing 'compiled_at' in frontmatter"
    );
    assert!(
        mapping.get("updated_at").and_then(|v| v.as_str()).is_some(),
        "branch missing 'updated_at' in frontmatter"
    );
    assert!(
        mapping.get("leaves").is_some(),
        "branch missing 'leaves' in frontmatter"
    );
    assert!(!body.trim().is_empty(), "branch body is empty");
}

#[test]
#[ignore = "requires OPENAI_API_KEY"]
fn compile_gives_every_leaf_a_branches_field() {
    let dir = setup_fixture_collection();
    let cfg = make_config(dir.path());

    compile::cmd_compile(&cfg).unwrap();

    let index_path = dir.path().join("index.jsonl");
    let entries = index::read_index(&index_path).unwrap();

    for entry in &entries {
        let leaf_path = dir.path().join(&entry.file);
        let content = fs::read_to_string(&leaf_path).unwrap();
        let (mapping, _) = bo::frontmatter::parse(&content).unwrap();
        assert!(
            mapping.get("branches").is_some(),
            "leaf {} missing 'branches' field after compile",
            entry.file
        );
    }
}

#[test]
#[ignore = "requires OPENAI_API_KEY"]
fn compile_does_not_modify_index_jsonl() {
    let dir = setup_fixture_collection();
    let cfg = make_config(dir.path());

    let index_before =
        fs::read_to_string(dir.path().join("index.jsonl")).unwrap();

    compile::cmd_compile(&cfg).unwrap();

    let index_after =
        fs::read_to_string(dir.path().join("index.jsonl")).unwrap();
    assert_eq!(
        index_before, index_after,
        "index.jsonl was modified by bo compile"
    );
}

#[test]
#[ignore = "requires OPENAI_API_KEY"]
fn compile_rerun_preserves_compiled_at() {
    let dir = setup_fixture_collection();
    let cfg = make_config(dir.path());

    // First compile
    compile::cmd_compile(&cfg).unwrap();

    let branches_dir = dir.path().join("branches");
    let first_branch = fs::read_dir(&branches_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
        .next()
        .expect("no branch files after first compile")
        .path();

    let content1 = fs::read_to_string(&first_branch).unwrap();
    let (m1, _) = bo::frontmatter::parse(&content1).unwrap();
    let compiled_at_1 = m1
        .get("compiled_at")
        .and_then(|v| v.as_str())
        .unwrap()
        .to_string();

    // Brief sleep to ensure timestamp differs if updated_at changes
    std::thread::sleep(std::time::Duration::from_secs(1));

    // Second compile
    compile::cmd_compile(&cfg).unwrap();

    // Find the same branch (by slug/filename)
    let content2 = fs::read_to_string(&first_branch).unwrap();
    let (m2, _) = bo::frontmatter::parse(&content2).unwrap();
    let compiled_at_2 = m2
        .get("compiled_at")
        .and_then(|v| v.as_str())
        .unwrap()
        .to_string();

    assert_eq!(
        compiled_at_1, compiled_at_2,
        "compiled_at changed on second compile run"
    );
}

// ── offline unit checks ───────────────────────────────────────────────────────

#[test]
fn compile_exits_cleanly_on_empty_collection() {
    let dir = tempfile::TempDir::new().unwrap();
    // No index.jsonl — read_index returns empty vec
    let cfg = make_config(dir.path());
    // Should print "bo is empty!" and return Ok(())
    // (no OPENAI_API_KEY needed — guard fires first)
    std::env::remove_var("OPENAI_API_KEY");
    // With 0 leaves the guard fires before the API key check
    // We need an empty index file to test this
    fs::write(dir.path().join("index.jsonl"), "").unwrap();
    let result = compile::cmd_compile(&cfg);
    assert!(result.is_ok());
}

#[test]
fn compile_exits_cleanly_on_single_leaf() {
    let dir = tempfile::TempDir::new().unwrap();
    fs::write(
        dir.path().join("index.jsonl"),
        r#"{"file":"only.md","title":"Only","url":"https://example.com"}"#,
    )
    .unwrap();
    std::env::remove_var("OPENAI_API_KEY");
    let cfg = make_config(dir.path());
    let result = compile::cmd_compile(&cfg);
    assert!(result.is_ok());
}

#[test]
fn compile_errors_without_api_key() {
    let dir = setup_fixture_collection();
    let cfg = make_config(dir.path());
    std::env::remove_var("OPENAI_API_KEY");
    let result = compile::cmd_compile(&cfg);
    assert!(result.is_err());
    let msg = result.unwrap_err();
    assert!(
        msg.contains("OPENAI_API_KEY"),
        "error message should mention OPENAI_API_KEY, got: {}",
        msg
    );
}
