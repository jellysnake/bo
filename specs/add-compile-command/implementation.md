# `bo compile` — Implementation

## Status: Complete

All 16 tasks from `tasks.md` implemented and verified.
91 tests passing (76 unit, 8 integration offline, 5 integration live, 2 other).

---

## What Was Built

`bo compile` is a tool-calling LLM agent that reads a bo collection, identifies
recurring concepts across documents, and produces **branch** files — one per
concept — while backwriting each leaf's frontmatter with the branches it belongs
to. The result is a bidirectionally linked knowledge graph.

Alongside the feature, three refactoring commits landed on the same branch:
- `leaf.rs` introduced as the domain entity module for leaves (analogous to `branch.rs`)
- `markdown.rs` deleted (subsumed by `leaf.rs`)
- `pipeline.rs` renamed to `collect.rs`, `bo add` renamed to `bo collect`

---

## New Modules

### `src/frontmatter.rs`

YAML frontmatter parsing and patching. The key design decision here was using
**two separate code paths** after discovering that `serde_yaml_ng` re-quotes
string values on round-trip (e.g. `title: "Simple Title"` → `title: Simple
Title`). Forcing every leaf through a parse→serialize cycle would dirty every
leaf's title field on each compile run.

- `parse()` + `render()` + `set_field()` — for **creating** branch files from
  scratch. Round-trip quoting changes are fine here since there is no original
  to compare against.
- `patch_fields()` — for **updating** existing leaf frontmatter. Surgical
  string-based replacement: only the target fields (`branches:`, `updated_at:`)
  are touched; all other fields, including their original quoting style, are
  preserved byte-for-byte.

`patch_fields` was the most non-obvious piece of the implementation. It works
by splitting the document into its raw YAML string and body, operating on lines
directly rather than serialising back through serde, and rejoining with the
original suffix so the body is guaranteed identical.

### `src/branch.rs`

Domain entity I/O for branch files. Mirrors `leaf.rs` in scope and structure.

- `write()` — builds a Mapping via `frontmatter::set_field`, serialises with
  `frontmatter::render`. Auto-prepends `# {title}` heading if the agent's body
  omits it (robustness guard).
- `read_compiled_at()` — reads the `compiled_at` timestamp from an existing
  branch file so that recompile runs preserve the first-write timestamp. Returns
  `None` for all failure cases (missing, corrupted, absent field) so the caller
  uses first-write semantics uniformly.

### `src/agent.rs`

Provider-agnostic LLM loop. All `async-openai` types are private to this module;
nothing leaks across the boundary. The public surface is two traits, a config
struct, and a `run()` function.

**`Tool` and `LlmProvider` traits** mirror the `mini-agent` crate's interface
design for forward compatibility. `mini-agent` itself was not taken as a direct
dependency — its OpenAI provider hardcodes stdout output that would corrupt bo's
output model, among other issues (see `research.md`).

**`OpenAiProvider`** uses `async-openai 0.34`. Two non-obvious API details for
that version:
- Tool calls are wrapped in a `ChatCompletionMessageToolCalls::Function(tc)`
  enum variant — must be destructured, not accessed directly.
- `tc.function.arguments` is a `String` (not `serde_json::Value`); must be
  parsed via `.parse()`.
- The builder pattern is `mutable` — setters return `&mut Self`, not `Self`,
  so assignments like `builder = builder.tool_calls(...)` fail to compile.
- Tool specs must be `Vec<ChatCompletionTools>` (enum), not `Vec<ChatCompletionTool>`.

**`run()` loop** dispatches tool calls sequentially. Each tool holds
`Arc<Mutex<CompileContext>>`; the lock/copy/unlock pattern is applied consistently
(lock briefly to copy needed data, drop the guard before file I/O, never hold a
`MutexGuard` across an `.await` point to avoid compiler rejection).

**Async strategy**: a `tokio::runtime::Builder::new_current_thread()` runtime
is created inside `cmd_compile` and destroyed on return. Existing commands use
`reqwest::blocking`, which panics if called from within a multi-threaded tokio
runtime; the single-threaded local runtime sidesteps this entirely.

### `src/compile.rs`

The compile command, tools, system prompt, and output formatting.

**Four tools** each hold `Arc<Mutex<CompileContext>>`:

| Tool | Key behaviour |
|---|---|
| `ListIndexTool` | Serialises valid leaves to JSON; emits stderr progress |
| `ReadLeafTool` | Path-traversal guard; returns error string (not `Err`) on failure |
| `WriteBranchTool` | Validates `leaves` parameter against known index (filters invented filenames); preserves `compiled_at`; emits stderr progress |
| `UpdateLeafFrontmatterTool` | Uses `patch_fields` so unrelated frontmatter fields are untouched |

**`cmd_compile` ordering**: the leaf-count guard (0 → `bo is empty!`, 1 → `bo
only has 1 leaf!`) fires before the `OPENAI_API_KEY` check. You should not need
an API key to be told your collection is empty.

**`COMPILE_SYSTEM_PROMPT`** is a named constant in this module, tracked in
version control alongside the code. It includes explicit step ordering (list →
read → branch → backlink → summarise), quality guidance, and guards against
known failure modes (over-branching, skipping `update_leaf_frontmatter`,
re-reading already-read documents, inventing leaf filenames).

### `src/leaf.rs`

Domain entity I/O for leaf documents. Introduced as a refactor on the same
branch. Replaced `markdown.rs`, which was a generically-named module doing a
domain-specific thing.

- `write()` — formats the YAML frontmatter block with double-quoted title
  (preserving the canonical format established by the original `markdown.rs`)
  and writes to disk.
- `read_frontmatter()` — reads and parses a leaf's frontmatter, unifying I/O
  errors and parse errors under a single `LeafError` type. Used by `cmd_compile`
  to validate leaves before the agent run; both failure modes are treated
  identically (skip the leaf).

---

## Modified Modules

### `src/collect.rs` (was `src/pipeline.rs`, was `src/add.rs`)

Two calls to `markdown::format_document` + `markdown::write_document` replaced
by a single `leaf::write` call. The module was renamed twice: first to `add.rs`
for semantic consistency with `compile.rs`, then to `collect.rs` because the
internal vocabulary throughout the codebase (`collect_url`, `collect_html`,
`CollectError`, `collected_at`, `✓ collected:`) already used "collect" — "add"
was a provisional name from early CLI development.

### `src/config.rs`

`compile_model: Option<String>` added with `#[serde(default)]`. Backward
compatible: existing `config.json` files without the field parse cleanly.
`effective_compile_model()` returns the stored value or `"gpt-4o"` as fallback.
Users set it by editing `~/.bo/config.json` directly.

### `src/main.rs`

`Commands::Collect` (was `Commands::Add`). `cmd_compile` wired via
`require_config().and_then(|cfg| bo::compile::cmd_compile(&cfg))` — `?` cannot
be used directly in a `main()` match arm since `main()` returns `()`.

### `src/lib.rs`

`markdown` removed; `agent`, `branch`, `compile`, `frontmatter`, `leaf` added.
Dependency diagram updated to show both `collect` and `compile` command branches.

---

## Deleted

- `src/markdown.rs` — subsumed by `leaf.rs`

---

## New Dependencies

```toml
async-openai = { version = "0.34", features = ["chat-completion"] }
async-trait  = "0.1"
tokio        = { version = "1", features = ["rt", "macros"] }
serde_yaml_ng = "0.10"   # serde_yaml 0.9 is deprecated; this is the maintained fork
```

`serde_yaml 0.9` was the original plan but is marked deprecated on crates.io
(`0.9.34+deprecated`). `serde_yaml_ng 0.10` is the maintained fork with an
identical API.

`async-openai` was initially planned at `0.27` (from the research phase) but
the actual current version is `0.34`, with a changed API. The analysis step
caught this before implementation started.

---

## Test Coverage

| Suite | Count | Notes |
|---|---|---|
| `frontmatter` unit tests | 17 | round-trip, patch, parse/render |
| `branch` unit tests | 7 | write, read_compiled_at, heading injection |
| `leaf` unit tests | 10 | write, read_frontmatter, error types |
| `compile` unit tests | 18 | all four tools, context, path traversal |
| `integration_compile` offline | 3 | empty/single-leaf guards, missing API key |
| `integration_compile` live (`#[ignore]`) | 5 | require `OPENAI_API_KEY` |

Live tests run with `cargo test --test integration_compile -- --ignored`.
All 5 passed against the fixture collection (6 Rust-themed documents).

---

## Real-World Run

A full lifecycle was run against 47 URLs from a real browsing session.
42 collected successfully (5× Medium 403, 1× goose 404, 1× milvus redirect).
`bo compile` produced 5 branches:

| Branch | Leaves |
|---|---|
| AI Coding Agents and Frameworks | 11 |
| Rust Web Frameworks and Tools | 6 |
| Agentic Engineering and Harness Concepts | 6 |
| Subagents and Multistep Planning | 5 |
| Machine Learning and LLMs | 4 |

The clustering was accurate. Known issues observed:

**3 leaves were not backlinked** — the agent referenced them in branch `leaves:`
arrays but did not call `update_leaf_frontmatter` for all of them. Bidirectional
consistency was not fully achieved. The system prompt instructs this explicitly
but enforcement is not structural. A future improvement: after the agent loop
ends, bo could compare `ctx.leaves_updated` against `ctx.valid_leaves` and
emit a warning (or patch them directly) for any leaf that was referenced in a
branch but not updated.

**Branch bodies have a redundant subheading** — the agent wrote
`### {Title}` at the top of the body, and `branch::write` prepended `# {Title}`
since it didn't detect the `###` as the required heading. The prompt should
instruct the agent to begin the body with content directly after the heading.

**`bo raze` does not clean up `branches/`** — branches are not tracked in
`index.jsonl` (by design), so `raze` leaves the `branches/` directory behind.
This is noted in the scratchpad as a future improvement.
