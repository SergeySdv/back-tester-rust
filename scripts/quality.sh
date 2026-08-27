#!/usr/bin/env bash
set -euo pipefail

uv run python scripts/check_file_lengths.py
cargo clippy --workspace --all-targets --all-features -- -D warnings
uv run ruff check crates python scripts
