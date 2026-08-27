#!/usr/bin/env bash
set -euo pipefail

readonly CARGO_LLVM_COV_VERSION="0.9.0"
readonly NIGHTLY_TOOLCHAIN="nightly-2026-08-01"
readonly REPOSITORY_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly INITIAL_GIT_STATUS="$(git -C "${REPOSITORY_ROOT}" status --porcelain=v1 --untracked-files=all)"

cd "${REPOSITORY_ROOT}"

assert_no_external_profiles() {
  local leaked_profiles
  leaked_profiles="$(find . -path ./target -prune -o -name '*.profraw' -print)"
  if [[ -n "${leaked_profiles}" ]]; then
    printf 'coverage profile escaped target/:\n%s\n' "${leaked_profiles}" >&2
    return 1
  fi
}

mkdir -p target/coverage
cargo clean --target-dir target/llvm-cov-stable
(
  eval "$(cargo llvm-cov show-env --sh)"
  export CARGO_TARGET_DIR="${REPOSITORY_ROOT}/target/llvm-cov-stable"
  export CARGO_LLVM_COV_TARGET_DIR="${CARGO_TARGET_DIR}"
  export CARGO_LLVM_COV_BUILD_DIR="${CARGO_TARGET_DIR}"
  export LLVM_PROFILE_FILE="${CARGO_TARGET_DIR}/back-tester-%p-%12m.profraw"
  cargo test --workspace --all-features
  uv run maturin develop
  uv run pytest -q
  cargo llvm-cov report --fail-under-lines 90 --json \
    --output-path "${REPOSITORY_ROOT}/target/coverage/rust-lines.json"
)

cargo clean --target-dir target/llvm-cov-nightly
(
  eval "$(cargo +${NIGHTLY_TOOLCHAIN} llvm-cov show-env --branch --sh)"
  export RUSTUP_TOOLCHAIN="${NIGHTLY_TOOLCHAIN}"
  export CARGO_TARGET_DIR="${REPOSITORY_ROOT}/target/llvm-cov-nightly"
  export CARGO_LLVM_COV_TARGET_DIR="${CARGO_TARGET_DIR}"
  export CARGO_LLVM_COV_BUILD_DIR="${CARGO_TARGET_DIR}"
  export LLVM_PROFILE_FILE="${CARGO_TARGET_DIR}/back-tester-%p-%12m.profraw"
  cargo test --workspace --all-features
  uv run maturin develop
  uv run pytest -q
  cargo llvm-cov report --branch --json \
    --output-path "${REPOSITORY_ROOT}/target/coverage/rust-branches.json"
)

# Restore the ordinary editable extension after the instrumented nightly build.
cargo clean --target-dir target/native-normal
CARGO_TARGET_DIR="${REPOSITORY_ROOT}/target/native-normal" uv run maturin develop
uv run python -m pytest \
  --cov=back_tester --cov-branch \
  --cov-report=term-missing \
  --cov-report=json:target/coverage/python.json

uv run python scripts/check_coverage.py \
  --rust-lines target/coverage/rust-lines.json --min-rust-lines 90 \
  --rust-branches target/coverage/rust-branches.json --min-rust-branches 85 \
  --python target/coverage/python.json \
  --min-python-lines 85 --min-python-branches 80

assert_no_external_profiles
readonly FINAL_GIT_STATUS="$(git status --porcelain=v1 --untracked-files=all)"
if [[ "${FINAL_GIT_STATUS}" != "${INITIAL_GIT_STATUS}" ]]; then
  printf 'coverage command changed git status\n' >&2
  diff <(printf '%s\n' "${INITIAL_GIT_STATUS}") <(printf '%s\n' "${FINAL_GIT_STATUS}") || true
  exit 1
fi

printf 'coverage tools: cargo-llvm-cov %s, %s, pytest-cov 7.0.0\n' \
  "${CARGO_LLVM_COV_VERSION}" "${NIGHTLY_TOOLCHAIN}"
