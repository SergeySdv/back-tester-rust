# AGENTS.md

## Purpose of the project

This repository is intended for deterministic research of a strategy that
sells a synthetic 24-hour BTC ATM straddle on OKX BTCUSDT perpetual minute
history.

The first-stage goal is to test the hypothesis in terms of PnL, equity, maximum
drawdown, and margin usage. This is a scenario-based Black–Scholes backtest,
not historical option-market replay or a live-trading system.

## Mandatory reading order

Before changing code or documentation, the agent must:

1. read this file;
2. read the compact [`docs/README.md`](docs/README.md) index and
   [`docs/architecture/02_system_overview.md`](docs/architecture/02_system_overview.md);
3. read `docs/architecture/01_btc_24h_rust_python_mvp.md`;
4. read the canonical input-data contract
   [`docs/data/okx_btcusdt_swap_1m.md`](docs/data/okx_btcusdt_swap_1m.md);
5. read [`docs/epics/README.md`](docs/epics/README.md), the canonical brief for
   the current epic/feature, and the applicable prompt under `docs/prompts/`;
6. check the actual state of the code, tests, configuration and Git working tree.

`docs/archive/` contains historical requirements, superseded analysis, and
future research. It is reference material, not the active Rust implementation
contract.

## Architectural boundaries

### Rust core

Rust is responsible for:

- Black–Scholes and payoff on expiry;
- validation of numerical inputs at the native boundary;
- causal minute-by-minute lifecycle;
- IV scenarios and 70/30 reserve rule;
- positions, cash, option liability, locked/available margin;
- equity, PnL, drawdown and deterministic result contract.

`backtest-core` must not depend on Python, pandas, or exchange SDKs.

### Python orchestration

Python is responsible for:

- loading and displaying CSV/Parquet schema;
- preparation of contiguous arrays;
- user configuration and scenario launch;
- one bulk Rust call to backtest;
- pandas/reporting and future integration with the universal exchange API.

Per-minute Python callbacks during a backtest are prohibited.

## Immutable rules of the MVP model

- Underlying — minute close BTCUSDT perpetual.
- Synthetic option: European call plus put, both positions short.
- Expiry occurs exactly after 24 hours elapsed time.
- `K = S_entry`; ATM is guaranteed only at initial entry.
- Strike and expiry reserve tranches coincide with the initial ones.
- Black–Scholes accepts finite configurable `r` and `q`; the baseline uses
  explicit defaults `r=0`, `q=0`, but pricing must not hard-code these zeros.
- IV is set as annualized decimal; baseline constant, stress after entry
  fully reprices both legs at `2x` or `3x` IV.
- The public scenario set is a non-empty unique subset of only
  `baseline`, `stress_2x`, `stress_3x` in canonical order.
- Linear vega approximation does not replace full Black–Scholes repricing.
- 70% and 30% are margin budgets, not premium and not notional.
- The reserve trigger fires no more than once when
  `current_iv >= 1.5 * entry_iv`.
- Reserve size is limited by margin actually available after the equity change;
  reduction or rejection must be visible in the result.
- 24h trades do not overlap in MVP.
- At a shared daily boundary the order is fixed: settle the old trade, realize
  PnL and release margin, enter the new trade, then emit one post-event row.
- `1.0` quantity means a call plus a put on one BTC; internally quantity is
  stored as an integer number of steps, not accumulated `f64`.
- Look-ahead, hidden data repair, and nondeterministic decisions are prohibited.
- Accounting uses model USD and does not claim exact conformity with any
  exchange's margin rules.

Changing these rules requires an explicit user decision and an update to the
architecture document before implementation changes.

## Design and code rules

### Correctness first

- Financial and time correctness are more important than micro-optimizations.
- Formulas, units, timezone, currency and the moment of the event must be explicit.
- First fix invariants and test cases, then optimize the hot path.
- Do not pass off a synthetic option mark as a real bid/ask, deal or fill.

### KISS

- Choose the simplest solution that fully fulfills acceptance
  criteria.
- Prefer explicit types and small functions over hidden magic and complex frameworks.
- Do not add distributed services, a plugin system, a database, or an async
  runtime unless the epic explicitly requires it.

### YAGNI

- Do not implement future exchange connectors, historical option replay, IV
  surface, order book, liquidation engine or live trading in advance.
- Do not add configuration knobs without the current usage scenario.
- Do not create a universal abstraction for a single implementation.

### DRY

- Do not duplicate formulas, validation rules and accounting transitions.
- Shared logic must have one source of truth and dedicated tests.
- DRY does not justify premature abstraction: a small amount of obvious
  repetition is better than an incorrect general model.

### Separation of concerns

- Pricing, strategy state, portfolio accounting, margin, data loading, and
  reporting must remain separate responsibilities.
- The domain core must not import orchestration or presentation layers.
- Use dependency inversion only on the real external boundary.

## Rust rules

- Use typed errors for invalid data; do not use `panic!`,
  `unwrap()`, or `expect()` on a user-facing path.
- Check all floating-point inputs for `is_finite()` and the acceptable range.
- Do not compare a calculated `f64` for exact equality without mathematical
  justification; tolerance must be named and tested.
- Avoid `unsafe`; if you can’t do without it, document the invariants and add
  narrow tests.
- Public types and errors must be stable and meaningful.
- Avoid allocations and dynamic dispatch in the minute loop without a measured
  necessity.
- The code must pass formatting, Clippy with warnings-as-errors and all tests.

## Python rules

- The Python layer remains thin, typed, and bulk I/O oriented.
- Do not implement the financial state machine a second time in Python.
- Check schema, dtype, timezone, monotonicity, and contiguous layout before the
  native call; Rust validates the numeric boundary again.
- Do not suppress native exceptions or return partial results as success.
- Do not add a dependency unless its necessity is established.
- Public functions must have type hints and clear errors.

## Data and determinism

The canonical approved OKX minute-input format, mapping rules, and raw-trade
boundary are defined in
[`docs/data/okx_btcusdt_swap_1m.md`](docs/data/okx_btcusdt_swap_1m.md).

- Source, symbol, interval, timezone and dataset ID must be stored in
  result metadata.
- Gaps, duplicates, out-of-order timestamps, NaN, infinity and non-positive
  prices are explicitly rejected.
- Do not sort, forward-fill or repair the input silently.
- Time to expiry is derived from timestamps, not merely from a row number.
- The same input and config must give bitwise identical result arrays,
  other than a pre-documented platform limitation.
- If randomness appears, the seed becomes a required part of the config and
  result.

## Testing and quality gates

The practical procedure for installation, adding data, running a backtest, and
checking the trading hypothesis is described in
[`docs/guides/run_backtest_and_validate_strategy.md`](docs/guides/run_backtest_and_validate_strategy.md).

Coverage percentage is a minimum indicator, not a substitute for verifying requirements.

- Rust core: line coverage at least 90%, branch coverage at least 85%.
- Python orchestration/reporting: line coverage at least 85%, branch coverage at
  least 80%.
- Changed executable code: line coverage at least 90% if diff coverage is
  supported by project tools.
- For pricing, time causality, 70/30 sizing, margin ledger, settlement, PnL and
  drawdown, there must be tests for each acceptance rule and boundary
  condition regardless of the final percentage.

Minimum checks when relevant parts of the project exist:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
pytest
```

Coverage policy uses `cargo-llvm-cov` for Rust lines, separate pinned
nightly `cargo-llvm-cov --branch` job for Rust branches, `pytest-cov` with
`--cov-branch` for Python and `scripts/check_coverage.py` as a single threshold
gate. The canonical commands are in section 14.2 of the architecture document;
tool versions and the nightly toolchain are pinned in the repository by the scaffold
epic. Coverage exclusions are allowed only through a reviewed path rule with
justification. An agent must not invent a percentage if the JSON report was not
actually generated.

## Git and the working tree

- Before work, record the branch, base commit, and initial `git status`.
- Preserve unrelated user changes.
- Do not use destructive Git commands without express permission.
- Do not commit `.codex/`, `.idea/`, secrets, credentials, datasets and temporary
  build/cache artifacts.
- Commit and push are performed only when explicitly requested by the user or epic.
- Before handoff, check diff, untracked files and real command results.

## Agent workflow

For an epic/feature, use the sequential process from
[`docs/prompts/README.md`](docs/prompts/README.md):

1. the master agent selects the canonical immutable epic brief and records the
   base commit, user-owned changes, and iteration context without changing its
   criteria;
2. developer implements and writes a mini-report;
3. QA checks quality, tests and coverage and writes a mini-report;
4. reviewer independently checks compliance with epic and writes findings;
5. master accepts the result or begins the next iteration.

It is prohibited for multiple agents to modify the same work tree at the same time.
At most three full `developer → QA → reviewer` iterations are allowed per epic.

## Requirements for handoff

Each performer reports:

- task and iteration number;
- base commit and changed files;
- actually implemented behavior;
- connection with acceptance criteria;
- exact commands and actual test results;
- coverage or honest `not measured`;
- assumptions, risks, blockers and the next recommended step.

Do not write “done,” “all tests passed,” or “meets epic” without
verifiable evidence from the current working tree.
