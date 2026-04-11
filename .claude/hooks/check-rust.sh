#!/bin/bash
# check-rust.sh — PostToolUse hook for Edit|Write
# Only runs cargo checks when a .rs file was modified.
# Trims output to signal-only: silent on success, errors on failure.

# Read hook input from stdin
INPUT=$(cat)

# Extract the file path from the tool input
FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty')

# Skip if not a Rust file
if [[ "$FILE_PATH" != *.rs ]]; then
  exit 0
fi

ERRORS=""

# cargo check
CHECK_OUTPUT=$(cargo check --quiet 2>&1)
if [ $? -ne 0 ]; then
  ERRORS+="cargo check failed:\n${CHECK_OUTPUT}\n\n"
fi

# cargo fmt
FMT_OUTPUT=$(cargo fmt --check --quiet 2>&1)
if [ $? -ne 0 ]; then
  ERRORS+="cargo fmt check failed:\n${FMT_OUTPUT}\n\n"
fi

# cargo clippy
CLIPPY_OUTPUT=$(cargo clippy --quiet -- -D warnings 2>&1)
if [ $? -ne 0 ]; then
  ERRORS+="cargo clippy failed:\n${CLIPPY_OUTPUT}\n\n"
fi

if [ -n "$ERRORS" ]; then
  echo -e "$ERRORS" >&2
  exit 2
fi

# All green — silent
exit 0
