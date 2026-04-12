// Top-level pipeline — the engine's public API.
//
// This is the only module a future consumer of the `bo` library needs to import
// to stash content. It owns the orchestration of the underlying engine modules
// (fetch, extract, slug, markdown, ledger) and exposes two entry points:
//
//   stash_url(url, output_dir)        — full pipeline including network fetch
//   stash_html(url, html, output_dir) — same, but accepts pre-fetched HTML
//
// `stash_html` is the testable core; `stash_url` is a thin wrapper that fetches first.
//
// Dependency direction: pipeline → fetch, extract, slug, markdown, ledger.

use chrono::Utc;
use std::fmt;
use std::path::Path;

use crate::{extract, fetch, ledger, markdown, slug};

// ── types ────────────────────────────────────────────────────────────────────

/// Successful result of stashing a page.
#[derive(Debug)]
pub struct StashedPage {
    /// Normalised URL that was stashed and recorded in the ledger.
    pub url: String,
    /// Filename (including `.md` extension) written inside `output_dir`.
    pub filename: String,
}

/// Unified error type for the stash pipeline.
#[derive(Debug)]
pub enum StashError {
    /// The URL has already been stashed; contains the existing filename.
    DuplicateUrl {
        existing_file: String,
    },
    Fetch(fetch::FetchError),
    Extract(extract::ExtractError),
    Io(std::io::Error),
}

impl fmt::Display for StashError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StashError::DuplicateUrl { existing_file } => {
                write!(f, "already stashed → {}", existing_file)
            }
            StashError::Fetch(e) => write!(f, "{}", e),
            StashError::Extract(e) => write!(f, "{}", e),
            StashError::Io(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl From<fetch::FetchError> for StashError {
    fn from(e: fetch::FetchError) -> Self {
        StashError::Fetch(e)
    }
}

impl From<extract::ExtractError> for StashError {
    fn from(e: extract::ExtractError) -> Self {
        StashError::Extract(e)
    }
}

impl From<std::io::Error> for StashError {
    fn from(e: std::io::Error) -> Self {
        StashError::Io(e)
    }
}

// ── pipeline ─────────────────────────────────────────────────────────────────

/// Full pipeline: validate URL, fetch HTML, then run the extract-write-ledger pipeline.
///
/// The `url` passed to the underlying `stash_html` call is the normalised form
/// returned by `fetch_url`, preserving the canonicalisation that was previously
/// done in `main.rs`.
pub fn stash_url(url: &str, output_dir: &Path) -> Result<StashedPage, StashError> {
    let fetched = fetch::fetch_url(url)?;
    stash_html(&fetched.url, &fetched.html, output_dir)
}

/// Extract-write-ledger pipeline without network access. Accepts pre-fetched HTML.
///
/// `url` is used for duplicate detection, slug generation, and the ledger entry.
/// It must be a valid, normalised URL string (e.g. as returned by `fetch_url`).
///
/// This is the testable core of the pipeline: integration tests call it directly
/// with fixture HTML to avoid network dependencies.
pub fn stash_html(url: &str, html: &str, output_dir: &Path) -> Result<StashedPage, StashError> {
    let ledger_path = output_dir.join("ledger.jsonl");

    // Duplicate check
    let entries = ledger::read_ledger(&ledger_path)?;
    if let Some(existing) = ledger::is_duplicate(&entries, url) {
        return Err(StashError::DuplicateUrl {
            existing_file: existing.file.clone(),
        });
    }

    // Extract
    let content = extract::extract_content(html)?;

    // Slug
    let title_ref = content.title.as_deref().unwrap_or("");
    let base_slug = slug::slugify(title_ref, url);
    let filename = slug::resolve_slug(&base_slug, url, output_dir);

    // Write markdown
    // `write_document` calls `create_dir_all` internally, ensuring `output_dir` exists
    // before `append_entry` below requires the directory.
    let now = Utc::now();
    let now_str = now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let doc = markdown::format_document(
        content.title.as_deref(),
        url,
        &now_str,
        &content.body_markdown,
    );
    markdown::write_document(output_dir, &filename, &doc)?;

    // Ledger
    let entry = ledger::LedgerEntry {
        url: url.to_string(),
        fetched_at: now,
        file: format!("{}.md", filename),
    };
    ledger::append_entry(&ledger_path, &entry)?;

    Ok(StashedPage {
        url: url.to_string(),
        filename: format!("{}.md", filename),
    })
}
