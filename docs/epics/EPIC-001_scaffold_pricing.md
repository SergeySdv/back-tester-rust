# EPIC-001: workspace, pricing and coverage tooling

- Status: `READY`
- Brief version: `1.0`
- Dependencies: no
- External blocker: no; OKX file is not required

## Result

Create a minimal Rust workspace with an independent `backtest-core`, a
PyO3/maturin package scaffold, and one verified shared Rust implementation of
Black–Scholes call/put pricing and expiry payoff. The repository generates machine-readable line/branch coverage
reports and automatically checks thresholds.

## In scope

- Cargo workspace, `backtest-core`, binding crate and thin Python package;
- typed domain errors and minimal config/model types required by the pricing API;
- `DatasetMetadata` and closed set `baseline/stress_2x/stress_3x` with
  validation, without backtest loop;
- normal CDF, Black–Scholes with finite configurable `r` and `q`, `T=0`
  payoff and a single `SECONDS_PER_YEAR`;
- Rust/Python packaging smoke test;
- `cargo-llvm-cov`, pinned nightly branch job, `pytest-cov` and a single threshold
  checker;
- unit, boundary and property-style tests required by this scope.

## Out of scope

- minute lifecycle, positions, reserve, margin, PnL and drawdown;
- final result tables;
- OKX loader and assumptions about its columns;
- exchange API, historical options and live trading.

## Acceptance criteria

- `E1-01`: `cargo build --workspace` passes; `backtest-core` does not depend on
  Python, pandas or exchange SDK.
- `E1-02`: `maturin develop` in a pinned project environment creates an
  importable Python package; a smoke test calls the native bulk-pricing API
  without Python callback.
- `E1-03`: call/put matches at least three independent reference cases in
  documented tolerance; at least one case has non-zero `r` and `q`.
- `E1-04`: put-call parity, exact intrinsic payoff with `T=0`, ATM and
  near-expiry boundaries are covered by direct tests.
- `E1-05`: invalid/infinite `S`, `K`, `T`, `sigma`, `r` and `q` return
  typed error, not panic/NaN; negative `T` is prohibited.
- `E1-06`: `DatasetMetadata` rejects empty identity fields, an interval other
  than 60 seconds, and a timezone other than UTC; scenario collection rejects empty, duplicate, and
  custom variants.
- `E1-07`: the same inputs give the same outputs; pricing formula has
  one source of truth in Rust.
- `E1-08`: the repository fixes compatible versions of `cargo-llvm-cov`, nightly
  toolchain and `pytest-cov`; commands from the architecture create JSON reports.
- `E1-09`: `scripts/check_coverage.py` rejects missing/invalid report
  and enforces thresholds of Rust lines 90%, Rust branches 85%, Python lines 85%, and Python
  branches 80%; actual reports pass these thresholds.
- `E1-10`: format, Clippy warnings-as-errors, Rust/Python tests and documented
  coverage commands exit with exit code 0.

## Mandatory tests and evidence

- named Rust tests for each pricing/error criterion;
- Python import/native exception smoke tests;
- threshold checker test for pass, below-threshold and malformed/missing input;
- commands, test counts, percentages and versions of tools saved in the mini-report;
- review of the lack of state-machine/exchange scope creep.

## Handoff in EPIC-002

Pass stable pricing/error/config types, `quantity_step`-compatible
numerical policy and green quality gates. Pricing semantics must not change in
`EPIC-002` without a separate contract change.
