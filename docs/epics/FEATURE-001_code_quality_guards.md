# FEATURE-001: Code quality guards

## Goal

Keep executable Rust and Python code reviewable as the project grows by making
file size, function complexity, and nesting limits reproducible local and CI
gates. The change must preserve all EPIC-001 and EPIC-002 behavior.

## Scope

In scope:

- refactor existing code only where required to satisfy the guards;
- deterministic file-length checks for tracked Rust and Python code;
- Rust Clippy guards for long functions, cognitive complexity, and excessive
  nesting;
- Python lint guards for syntax/style, complexity, and excessive nesting;
- one documented local command and the same gate in CI;
- tests for the custom file-length checker.

Out of scope:

- changes to pricing, lifecycle, accounting, or public result contracts;
- new architecture layers or generic plugin/framework abstractions;
- generated files, vendored code, build output, virtual environments, and data;
- a blanket inheritance abstraction. Rust has no class inheritance; Python
  production code should prefer composition and must not introduce inheritance
  hierarchies for this feature.

## Immutable acceptance criteria

- **QG-01**: Every tracked executable source file under `crates/`, `python/`,
  and `scripts/` is checked. Production Rust/Python files are at most 500
  physical lines; test, benchmark, and quality-tool files are at most 500
  physical lines. A violation exits non-zero and identifies the path, actual
  count, and limit.
- **QG-02**: Rust quality linting fails on functions longer than 80 lines,
  cognitive complexity above 15, or blocks nested deeper than 4. Thresholds
  are repository configuration, not undocumented command-line knowledge.
- **QG-03**: Python quality linting is pinned and fails on ordinary correctness
  issues, cyclomatic complexity above 15, and excessive nested blocks. Its
  configuration is stored in `pyproject.toml`.
- **QG-04**: A single repository command runs file-length checks plus Rust and
  Python quality linting. CI runs that command, and contributor documentation
  names it.
- **QG-05**: Exclusions are explicit, narrow, and documented. New files are
  discovered automatically; there is no per-file allowlist that can silently
  exempt oversized production code.
- **QG-06**: Existing formatting, Clippy warnings-as-errors, Rust/Python tests,
  and coverage gates continue to pass with unchanged thresholds and contracts.
- **QG-07**: Automated tests cover the file checker at the exact limit, above
  the limit, extension filtering, excluded directories, and actionable error
  output.

## Required verification

```bash
./scripts/quality.sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
uv run python -m pytest
uv run bash scripts/coverage.sh
```

The brief is immutable during the developer -> QA -> reviewer cycle.
