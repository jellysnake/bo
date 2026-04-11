use chrono::Utc;
use link_stash::{extract, fetch, ledger, markdown, slug};
use url::Url;

use clap::Parser;
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "link-stash", about = "Stash web pages as local markdown")]
struct Cli {
    /// URL to fetch and stash
    url: String,

    /// Output directory (default: ~/.link-stash/)
    #[arg(long, short = 'o')]
    output_dir: Option<PathBuf>,
}

fn default_output_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".link-stash")
}

fn run(cli: Cli) -> Result<(), String> {
    // Validate URL — must be HTTP/HTTPS and parseable
    let parsed_url = Url::parse(&cli.url).map_err(|e| format!("invalid URL: {}", e))?;
    if parsed_url.scheme() != "http" && parsed_url.scheme() != "https" {
        return Err(format!(
            "invalid URL scheme '{}': must be http or https",
            parsed_url.scheme()
        ));
    }
    let url_str = parsed_url.as_str().to_string();

    let output_dir = cli.output_dir.unwrap_or_else(default_output_dir);

    // Ensure output dir exists
    std::fs::create_dir_all(&output_dir)
        .map_err(|e| format!("failed to create output directory: {}", e))?;

    let ledger_path = output_dir.join("ledger.jsonl");

    // Check for duplicate
    let entries =
        ledger::read_ledger(&ledger_path).map_err(|e| format!("failed to read ledger: {}", e))?;

    if let Some(existing) = ledger::is_duplicate(&entries, &url_str) {
        return Err(format!("already stashed: {} → {}", url_str, existing.file));
    }

    // Fetch
    eprintln!("fetching {}...", url_str);
    let fetch_result = fetch::fetch_url(&url_str).map_err(|e| format!("fetch failed: {}", e))?;

    // Extract
    let content = extract::extract_content(&fetch_result.html)
        .map_err(|e| format!("extraction failed: {}", e))?;

    // Generate slug
    let title_ref = content.title.as_deref().unwrap_or("");
    let base_slug = slug::slugify(title_ref, &url_str);
    let filename = slug::resolve_slug(&base_slug, &url_str, &output_dir);

    // Format and write markdown
    let now = Utc::now();
    let now_str = now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let doc = markdown::format_document(
        content.title.as_deref(),
        &cli.url,
        &now_str,
        &content.body_markdown,
    );

    markdown::write_document(&output_dir, &filename, &doc)
        .map_err(|e| format!("failed to write markdown: {}", e))?;

    // Append to ledger
    let entry = ledger::LedgerEntry {
        url: url_str,
        fetched_at: now,
        file: format!("{}.md", filename),
    };
    ledger::append_entry(&ledger_path, &entry)
        .map_err(|e| format!("failed to update ledger: {}", e))?;

    println!("✓ stashed: {} → {}.md", entry.url, filename);
    Ok(())
}

fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli) {
        eprintln!("error: {}", e);
        process::exit(1);
    }
}
