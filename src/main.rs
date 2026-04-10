use link_stash::{fetch, extract, slug, ledger, markdown};

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
    // Validate URL
    if !cli.url.starts_with("http://") && !cli.url.starts_with("https://") {
        return Err(format!("invalid URL (must start with http:// or https://): {}", cli.url));
    }

    let output_dir = cli.output_dir.unwrap_or_else(default_output_dir);

    // Ensure output dir exists
    std::fs::create_dir_all(&output_dir)
        .map_err(|e| format!("failed to create output directory: {}", e))?;

    let ledger_path = output_dir.join("ledger.jsonl");

    // Check for duplicate
    let entries = ledger::read_ledger(&ledger_path)
        .map_err(|e| format!("failed to read ledger: {}", e))?;

    if let Some(existing) = ledger::is_duplicate(&entries, &cli.url) {
        return Err(format!("already stashed: {} → {}", cli.url, existing.file));
    }

    // Fetch
    eprintln!("fetching {}...", cli.url);
    let fetch_result = fetch::fetch_url(&cli.url)
        .map_err(|e| format!("fetch failed: {}", e))?;

    // Extract
    let content = extract::extract_content(&fetch_result.html)
        .map_err(|e| format!("extraction failed: {}", e))?;

    // Generate slug
    let title_ref = content.title.as_deref().unwrap_or("");
    let base_slug = slug::slugify(title_ref, &cli.url);
    let filename = slug::resolve_slug(&base_slug, &cli.url, &output_dir);

    // Format and write markdown
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let doc = markdown::format_document(
        content.title.as_deref(),
        &cli.url,
        &now,
        &content.body_markdown,
    );

    markdown::write_document(&output_dir, &filename, &doc)
        .map_err(|e| format!("failed to write markdown: {}", e))?;

    // Append to ledger
    let entry = ledger::LedgerEntry {
        url: cli.url.clone(),
        fetched_at: now,
        file: format!("{}.md", filename),
    };
    ledger::append_entry(&ledger_path, &entry)
        .map_err(|e| format!("failed to update ledger: {}", e))?;

    println!("✓ stashed: {} → {}.md", cli.url, filename);
    Ok(())
}

fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli) {
        eprintln!("error: {}", e);
        process::exit(1);
    }
}
