//! # bo — engine library
//!
//! This crate provides the core engine for `bo`, a tool that fetches web pages
//! and stores them as local markdown files.
//!
//! ## Public API
//!
//! The primary entry point for any consumer (CLI, web server, etc.) is the
//! [`pipeline`] module:
//!
//! ```ignore
//! // Full pipeline including network fetch:
//! bo::pipeline::stash_url(url, output_dir)
//!
//! // Extract-write-ledger pipeline with pre-fetched HTML (useful for testing):
//! bo::pipeline::stash_html(url, html, output_dir)
//! ```
//!
//! ## Module dependency direction
//!
//! Dependencies flow inward. The CLI layer (`main.rs`) calls into `pipeline`;
//! `pipeline` orchestrates the engine modules; engine modules have no
//! dependencies on each other or on CLI code.
//!
//! ```text
//! main.rs  (CLI: arg parsing, output, dispatch)
//!   ├── config  (read directly by main; not used by pipeline)
//!   └── pipeline  (orchestration — the engine's public API)
//!         ├── fetch
//!         ├── extract
//!         ├── slug
//!         ├── markdown
//!         └── ledger
//! ```
//!
//! See `adrs/adr-001.md` for the full rationale behind these structural decisions.

pub mod config;
pub mod extract;
pub mod fetch;
pub mod ledger;
pub mod markdown;
pub mod pipeline;
pub mod slug;
