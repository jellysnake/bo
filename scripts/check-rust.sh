#!/usr/bin/env bash
# check-rust.sh — full-project cargo fmt and clippy check.
#
# Run manually to verify the whole project is clean.
# Per-file checks on every agent edit are handled automatically by the
# pi extension at .pi/extensions/check-rust.ts

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

FMT_OUTPUT=$(cargo fmt --check 2>&1) || {
  echo "cargo fmt failed — run 'cargo fmt' to fix:"
  echo "$FMT_OUTPUT"
  exit 2
}

CLIPPY_OUTPUT=$(cargo clippy -- -D warnings 2>&1) || {
  echo "cargo clippy failed:"
  echo "$CLIPPY_OUTPUT"
  exit 2
}

echo "OK"
