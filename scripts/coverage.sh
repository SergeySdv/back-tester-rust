#!/usr/bin/env bash
set -euo pipefail

readonly CARGO_LLVM_COV_VERSION="0.9.0"
readonly NIGHTLY_TOOLCHAIN="nightly-2026-08-01"

mkdir -p target/coverage

cargo llvm-cov --workspace --all-features \
  --fail-under-lines 90 \
  --json --output-path target/coverage/rust-lines.json

cargo +"${NIGHTLY_TOOLCHAIN}" llvm-cov --workspace --all-features \
  --branch --json --output-path target/coverage/rust-branches.json

python -m pytest \
  --cov=back_tester --cov-branch \
  --cov-report=term-missing \
  --cov-report=json:target/coverage/python.json

python scripts/check_coverage.py \
  --rust-lines target/coverage/rust-lines.json --min-rust-lines 90 \
  --rust-branches target/coverage/rust-branches.json --min-rust-branches 85 \
  --python target/coverage/python.json \
  --min-python-lines 85 --min-python-branches 80

printf 'coverage tools: cargo-llvm-cov %s, %s, pytest-cov 7.0.0\n' \
  "${CARGO_LLVM_COV_VERSION}" "${NIGHTLY_TOOLCHAIN}"
