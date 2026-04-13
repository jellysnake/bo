use bo::config::{self, Config, ConfigError};
use bo::index;
use bo::collect;

use clap::{Parser, Subcommand};
use std::io::ErrorKind;
use std::path::PathBuf;
use std::process;

const NOT_SEEDED_MSG: &str = "bo hasn't been seeded yet — run: bo seed <output-dir>";

#[derive(Parser)]
#[command(name = "bo", about = "Stash web pages as local markdown")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialise a stash at <output-dir> and save config
    Seed {
        /// Directory to store stashed content
        output_dir: PathBuf,
    },
    /// Fetch a URL and collect it
    Collect {
        /// URL to collect
        url: String,
    },
    /// Compile collected documents into a linked knowledge graph
    Compile,
    /// Delete all bo-managed files and config
    Raze,
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn not_seeded_err() -> String {
    NOT_SEEDED_MSG.to_string()
}

fn require_config() -> Result<Config, String> {
    match config::read_config(&config::config_path()) {
        Ok(cfg) => Ok(cfg),
        Err(ConfigError::NotFound) => Err(not_seeded_err()),
        Err(e) => Err(format!("failed to read config: {}", e)),
    }
}

// ── cmd_seed ─────────────────────────────────────────────────────────────────

fn cmd_seed(output_dir: PathBuf) -> Result<(), String> {
    // Resolve to absolute path
    let output_dir = if output_dir.is_absolute() {
        output_dir
    } else {
        std::env::current_dir()
            .map_err(|e| format!("failed to get current dir: {}", e))?
            .join(&output_dir)
    };

    // Check if already seeded
    match config::read_config(&config::config_path()) {
        Ok(existing) => {
            println!(
                "bo has already been seeded at {}!",
                existing.output_dir.display()
            );
            return Ok(());
        }
        Err(ConfigError::NotFound) => {} // proceed
        Err(e) => return Err(format!("failed to read config: {}", e)),
    }

    std::fs::create_dir_all(&output_dir)
        .map_err(|e| format!("failed to create output directory: {}", e))?;

    config::write_config(
        &Config {
            output_dir: output_dir.clone(),
            compile_model: None,
        },
        &config::config_path(),
    )
    .map_err(|e| format!("failed to write config: {}", e))?;

    println!("seeded bo at {}", output_dir.display());
    Ok(())
}

// ── cmd_add ──────────────────────────────────────────────────────────────────

fn cmd_collect(url: String) -> Result<(), String> {
    let cfg = require_config()?;
    eprintln!("fetching {}...", url);
    let page = collect::collect_url(&url, &cfg.output_dir).map_err(|e| e.to_string())?;
    println!("✓ collected: {} → {}", page.url, page.filename);
    Ok(())
}

// ── cmd_raze ─────────────────────────────────────────────────────────────────

fn cmd_raze() -> Result<(), String> {
    let cfg = require_config()?;
    let output_dir = cfg.output_dir;

    let index_path = output_dir.join("index.jsonl");
    let entries =
        index::read_index(&index_path).map_err(|e| format!("failed to read index: {}", e))?;

    // Delete ledger-tracked markdown files
    let mut deleted = 0usize;
    for entry in &entries {
        let resolved = output_dir.join(&entry.file);

        // Path traversal guard
        if !resolved.starts_with(&output_dir) {
            eprintln!(
                "warning: skipping ledger entry with suspicious path: {}",
                entry.file
            );
            continue;
        }

        match std::fs::remove_file(&resolved) {
            Ok(()) => deleted += 1,
            Err(e) if e.kind() == ErrorKind::NotFound => {} // already gone, fine
            Err(e) => return Err(format!("failed to delete {}: {}", resolved.display(), e)),
        }
    }
    println!("deleted {} markdown file(s)", deleted);

    // Delete index
    match std::fs::remove_file(&index_path) {
        Ok(()) => println!("deleted index"),
        Err(e) if e.kind() == ErrorKind::NotFound => {}
        Err(e) => return Err(format!("failed to delete ledger: {}", e)),
    }

    // Attempt to remove output dir (only succeeds if empty)
    match std::fs::remove_dir(&output_dir) {
        Ok(()) => println!("removed output directory {}", output_dir.display()),
        Err(e) if e.kind() == ErrorKind::DirectoryNotEmpty || e.kind() == ErrorKind::NotFound => {
            println!(
                "output directory left in place (not empty or already absent): {}",
                output_dir.display()
            );
        }
        Err(e) => return Err(format!("failed to remove output directory: {}", e)),
    }

    // Delete config last — so a mid-raze failure doesn't strand the user
    match std::fs::remove_file(config::config_path()) {
        Ok(()) => println!("deleted config"),
        Err(e) if e.kind() == ErrorKind::NotFound => {}
        Err(e) => return Err(format!("failed to delete config: {}", e)),
    }

    Ok(())
}

// ── main ─────────────────────────────────────────────────────────────────────

fn main() {
    // Initialise tracing. WARN+ shown by default; set RUST_LOG=debug for verbose output.
    // Format is message-only (no timestamp/level prefix) to match the CLI's plain output style.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::WARN.into()),
        )
        .with_target(false)
        .without_time()
        .with_level(false)
        .init();

    let cli = Cli::parse();
    let result = match cli.command {
        Commands::Seed { output_dir } => cmd_seed(output_dir),
        Commands::Collect { url } => cmd_collect(url),
        Commands::Compile => require_config().and_then(|cfg| bo::compile::cmd_compile(&cfg)),
        Commands::Raze => cmd_raze(),
    };
    if let Err(e) = result {
        eprintln!("error: {}", e);
        process::exit(1);
    }
}
