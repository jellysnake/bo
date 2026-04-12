#!/usr/bin/env bash
# check-rust.sh — run cargo fmt --check and cargo clippy on a changed Rust file.
#
# Designed for use as a pi PostToolUse hook on Edit|Write.
# Reads the pi hook JSON from stdin to determine the changed file path.
# Exits 0 (silent) for non-Rust files or a clean file.
# Exits non-zero with error output on fmt or clippy failures.
#
# Can also be run standalone (no stdin) to check the whole project:
#   bash scripts/check-rust.sh

set -euo pipefail

# ── resolve file path from hook input (if any) ────────────────────────────────
if [ -t 0 ]; then
  # No stdin (standalone run) — check the whole project
  FILE_PATH=""
else
  INPUT=$(cat)
  FILE_PATH=$(echo "$INPUT" | python3 -c "
import json, sys
try:
    data = json.load(sys.stdin)
    # pi hook: tool_input.file_path
    path = data.get('tool_input', {}).get('file_path', '')
    print(path)
except Exception:
    print('')
" 2>/dev/null || true)
fi

# ── skip non-Rust files ───────────────────────────────────────────────────────
if [ -n "$FILE_PATH" ] && [[ "$FILE_PATH" != *.rs ]]; then
  exit 0
fi

# ── move to repo root ─────────────────────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

# ── cargo fmt --check ─────────────────────────────────────────────────────────
FMT_OUTPUT=$(cargo fmt --check 2>&1) || {
  echo "cargo fmt failed — run 'cargo fmt' to fix:"
  echo "$FMT_OUTPUT"
  exit 2
}

# ── cargo clippy ─────────────────────────────────────────────────────────────
CLIPPY_OUTPUT=$(cargo clippy -- -D warnings 2>&1) || {
  echo "cargo clippy failed:"
  echo "$CLIPPY_OUTPUT"
  exit 2
}

# Silent success — zero context pollution on clean files
exit 0
