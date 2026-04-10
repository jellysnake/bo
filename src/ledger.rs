// JSONL ledger for successful fetches

use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, Write};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub url: String,
    pub fetched_at: String,
    pub file: String,
}

/// Read all ledger entries from a JSONL file.
/// Returns an empty vec if the file doesn't exist.
/// Skips malformed lines with a warning to stderr.
pub fn read_ledger(path: &Path) -> io::Result<Vec<LedgerEntry>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let file = fs::File::open(path)?;
    let reader = io::BufReader::new(file);
    let mut entries = Vec::new();
    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        match serde_json::from_str::<LedgerEntry>(trimmed) {
            Ok(entry) => entries.push(entry),
            Err(e) => {
                eprintln!("warning: skipping malformed ledger line {}: {}", i + 1, e);
            }
        }
    }
    Ok(entries)
}

/// Check if a URL already exists in the ledger (exact string match).
pub fn is_duplicate<'a>(entries: &'a [LedgerEntry], url: &str) -> Option<&'a LedgerEntry> {
    entries.iter().find(|e| e.url == url)
}

/// Append a single entry to the ledger file.
/// Creates the file if it doesn't exist.
pub fn append_entry(path: &Path, entry: &LedgerEntry) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    let json = serde_json::to_string(entry)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    writeln!(file, "{}", json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_entry(url: &str, file: &str) -> LedgerEntry {
        LedgerEntry {
            url: url.to_string(),
            fetched_at: "2025-01-15T09:32:00Z".to_string(),
            file: file.to_string(),
        }
    }

    #[test]
    fn append_creates_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("ledger.jsonl");
        assert!(!path.exists());

        append_entry(&path, &make_entry("https://example.com", "example.md")).unwrap();
        assert!(path.exists());

        let entries = read_ledger(&path).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].url, "https://example.com");
    }

    #[test]
    fn append_multiple_valid_jsonl() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("ledger.jsonl");

        append_entry(&path, &make_entry("https://a.com", "a.md")).unwrap();
        append_entry(&path, &make_entry("https://b.com", "b.md")).unwrap();
        append_entry(&path, &make_entry("https://c.com", "c.md")).unwrap();

        let entries = read_ledger(&path).unwrap();
        assert_eq!(entries.len(), 3);

        // Verify each line is independently valid JSON
        let content = fs::read_to_string(&path).unwrap();
        for line in content.lines() {
            assert!(serde_json::from_str::<LedgerEntry>(line).is_ok(),
                "line is not valid JSON: {line}");
        }
    }

    #[test]
    fn duplicate_detection_exact_match() {
        let entries = vec![
            make_entry("https://example.com/article", "article.md"),
        ];
        assert!(is_duplicate(&entries, "https://example.com/article").is_some());
    }

    #[test]
    fn near_duplicate_not_detected() {
        let entries = vec![
            make_entry("https://example.com/article", "article.md"),
        ];
        assert!(is_duplicate(&entries, "https://example.com/article?ref=twitter").is_none());
    }

    #[test]
    fn read_empty_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("ledger.jsonl");
        fs::write(&path, "").unwrap();

        let entries = read_ledger(&path).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn read_nonexistent_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nope.jsonl");
        let entries = read_ledger(&path).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn skips_malformed_lines() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("ledger.jsonl");
        let content = r#"{"url":"https://good.com","fetched_at":"2025-01-15T09:32:00Z","file":"good.md"}
this is not json
{"url":"https://also-good.com","fetched_at":"2025-01-15T09:33:00Z","file":"also-good.md"}
"#;
        fs::write(&path, content).unwrap();

        let entries = read_ledger(&path).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].url, "https://good.com");
        assert_eq!(entries[1].url, "https://also-good.com");
    }

    #[test]
    fn failed_url_not_in_ledger_can_resubmit() {
        // Only successful fetches go in the ledger.
        // A URL that failed previously should not be blocked.
        let entries = vec![
            make_entry("https://example.com/success", "success.md"),
        ];
        // A different URL that "failed" (never made it to ledger) is not blocked
        assert!(is_duplicate(&entries, "https://example.com/failed").is_none());
    }
}
