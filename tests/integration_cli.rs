// CLI integration tests for seed and raze subcommands.
//
// Uses $HOME override to redirect config to a temp dir, avoiding any
// interaction with the real ~/.bo/config.json.

use std::fs;
use std::path::Path;
use std::process::{Command, Output};
use tempfile::TempDir;

fn bo(home: &Path) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bo"));
    cmd.env("HOME", home);
    cmd
}

fn seed(home: &Path, output_dir: &Path) -> Output {
    bo(home)
        .args(["seed", output_dir.to_str().unwrap()])
        .output()
        .expect("failed to run bo seed")
}

fn raze(home: &Path) -> Output {
    bo(home)
        .arg("raze")
        .output()
        .expect("failed to run bo raze")
}

fn config_path(home: &TempDir) -> std::path::PathBuf {
    home.path().join(".bo").join("config.json")
}

// ── seed ─────────────────────────────────────────────────────────────────────

#[test]
fn seed_creates_output_dir_and_config() {
    let home = TempDir::new().unwrap();
    let stash = home.path().join("my-stash");

    let out = seed(home.path(), &stash);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // Output dir created
    assert!(stash.exists());

    // Config written and contains the path
    let cfg_path = config_path(&home);
    assert!(cfg_path.exists());
    let contents = fs::read_to_string(&cfg_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&contents).unwrap();
    assert_eq!(parsed["tree"]["output_dir"], stash.to_str().unwrap());
}

#[test]
fn seed_already_seeded_is_idempotent() {
    let home = TempDir::new().unwrap();
    let stash = home.path().join("my-stash");

    let out1 = seed(home.path(), &stash);
    assert!(out1.status.success());

    // Second seed with same dir: succeeds (no error), prints already-seeded message
    let out2 = seed(home.path(), &stash);
    assert!(out2.status.success());
    let stdout = String::from_utf8_lossy(&out2.stdout);
    assert!(
        stdout.contains("already been seeded"),
        "expected already-seeded message, got: {stdout}"
    );

    // Config still valid after double seed
    let cfg_path = config_path(&home);
    let contents = fs::read_to_string(&cfg_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&contents).unwrap();
    assert_eq!(parsed["tree"]["output_dir"], stash.to_str().unwrap());
}

#[test]
fn collect_without_seed_fails_with_helpful_message() {
    let home = TempDir::new().unwrap();

    let out = bo(home.path())
        .args(["collect", "https://example.com"])
        .output()
        .unwrap();

    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("seed"),
        "expected hint to run seed, got: {stderr}"
    );
}

// ── raze ─────────────────────────────────────────────────────────────────────

#[test]
fn raze_removes_config_and_cleans_stash() {
    let home = TempDir::new().unwrap();
    let stash = home.path().join("my-stash");

    // Seed
    seed(home.path(), &stash);
    assert!(config_path(&home).exists());

    // Manually write a stash file and index entry so raze has something to delete
    fs::create_dir_all(&stash).unwrap();
    fs::write(stash.join("article.md"), "# Article").unwrap();
    let entry = serde_json::json!({
        "file": "article.md",
        "title": "Article",
        "url": "https://example.com/article"
    });
    fs::write(
        stash.join("index.jsonl"),
        serde_json::to_string(&entry).unwrap(),
    )
    .unwrap();

    let out = raze(home.path());
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // Config deleted
    assert!(!config_path(&home).exists());

    // Stash file and index deleted
    assert!(!stash.join("article.md").exists());
    assert!(!stash.join("index.jsonl").exists());
}

#[test]
fn raze_without_seed_fails_with_helpful_message() {
    let home = TempDir::new().unwrap();

    let out = raze(home.path());
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("seed"),
        "expected hint to run seed, got: {stderr}"
    );
}

#[test]
fn raze_tolerates_already_deleted_files() {
    let home = TempDir::new().unwrap();
    let stash = home.path().join("my-stash");

    seed(home.path(), &stash);

    // Ledger references a file that doesn't exist on disk
    fs::create_dir_all(&stash).unwrap();
    let entry = serde_json::json!({
        "file": "gone.md",
        "title": "Gone",
        "url": "https://example.com/gone"
    });
    fs::write(
        stash.join("index.jsonl"),
        serde_json::to_string(&entry).unwrap(),
    )
    .unwrap();

    // Should not error — missing files are silently skipped
    let out = raze(home.path());
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!config_path(&home).exists());
}
