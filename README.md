# bo

Stash web pages as local markdown.

## Run locally

```bash
cargo run -- <command>
```

Or install the binary:

```bash
cargo install --path .
```

## Commands

```
bo seed <output-dir>   # Initialise a stash
bo collect <url>       # Fetch a URL and stash it
OPENAI_API_KEY=sk-... bo compile  # Build linked knowledge graph from stashed docs
bo raze                # Delete all bo-managed files and config
```
