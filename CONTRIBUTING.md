# Contributing

Run the repository quality gate before the full test and coverage suites:

```bash
./scripts/quality.sh
```

This single command checks physical file length, Rust Clippy guards, and the
pinned Python Ruff rules across `crates/`, `python/`, and `scripts/`. The
configured limits are:

- 500 physical lines for executable `.rs`, `.py`, and `.sh` files under
  `crates/`, `python/`, and `scripts/`;
- 80 lines per Rust function, cognitive complexity 15, and nesting depth 4;
- Python cyclomatic complexity 15 and nesting depth 4, alongside ordinary
  correctness and import-order checks.

The file checker uses Git discovery, so tracked files and non-ignored new files
are checked automatically without a per-file allowlist. Its only directory
exclusions are generated/build/cache or vendored trees named `target`, `.venv`,
`vendor`, `build`, `dist`, and `__pycache__`. Files outside the three source
roots and extensions other than `.rs`, `.py`, and `.sh` are outside this gate.

The thresholds live in `clippy.toml` and `pyproject.toml`; the Rust lint policy
is enabled for every workspace crate through `Cargo.toml`. The full verification
still includes formatting, tests, and unchanged coverage thresholds:

```bash
cargo fmt --all -- --check
cargo test --workspace --all-features
uv run python -m pytest
uv run bash scripts/coverage.sh
```
