# Final MVP validation and performance report

- Date: 2026-08-27
- Branch: `feature/epic-003-python-data-integration`
- Base commit: `e002e2da4482aba5e2209db0dcec114c3b29289d`
- Current implementation iteration: 3 of 3
- Current decision: `ACCEPTED`

## Scope and verdict

The final audit covers `EPIC-001`, `EPIC-002`, `EPIC-003`,
`FEATURE-001` and architecture criteria `AC-01..AC-11`.

The table below records the acceptance-criteria audit that supported the final
iteration-3 QA `PASS` and reviewer `APPROVED` decisions.

| Contract | Passed | Total | Evidence audit |
|---|---:|---:|---|
| EPIC-001 | 10 | 10 | PASS |
| EPIC-002 | 12 | 12 | PASS |
| EPIC-003 | 11 | 11 | PASS |
| FEATURE-001 | 7 | 7 | PASS |
| Architecture MVP | 11 | 11 | PASS |

Iteration 1 ended with developer `DONE`, QA `PASS`, and reviewer `APPROVED` on
the then-current tree. A subsequent review found checksum TOCTOU in the
specialized OKX loader: it hashed the path and then reopened it for parsing.

Iteration 2 fixed that finding by hashing and parsing one immutable byte
snapshot, protected root `data/` through `.gitignore`, translated repository
documentation to English, and reorganized active versus archived documentation.
Iteration-2 QA then found stale external-blocker wording and an inaccurate
incomplete-tail transition in the compact state diagram.

Iteration 3 updated those documentation defects without changing executable
behavior or acceptance criteria. Iteration-3 QA passed. The reviewer then
requested corrections for semantic drift in the English translation and for
stale workflow status in this report. Those corrections were implemented and
independently rechecked. Iteration-3 QA returned `PASS`, the reviewer returned
`APPROVED`, and the master accepted the current tree.

The system under review is a deterministic synthetic Black–Scholes scenario
backtester. It is not a historical option replay, exchange margin simulator,
live trading service or proof of strategy profitability.

## Developer verification history

The following full implementation commands completed with exit code zero in
iteration 2. Iteration 3 changes documentation only and reruns the documentation
audits plus the repository quality command.

```bash
./scripts/quality.sh
cargo fmt --all -- --check
cargo build --workspace --all-features --locked
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
uv lock --check
uv sync --frozen --all-extras
uv run maturin develop --locked
uv run pytest -q
uv run maturin build --locked --out target/final-audit-dist
uv run bash scripts/coverage.sh
cargo bench -p backtest-core --bench million
git diff --check
```

Test results:

- Rust: 52 passed, 0 failed, 0 ignored;
- Python: 84 passed, 0 failed, 0 skipped;
- focused data/loader suite: 40 passed;
- focused quality-guard suite: 16 passed;
- focused coverage-checker suite: 10 passed.

Coverage was regenerated during this audit from machine-readable JSON reports:

| Runtime | Metric | Covered / total | Result | Required |
|---|---|---:|---:|---:|
| Rust | lines | 1,355 / 1,387 | 97.69% | 90% |
| Rust | branches | 115 / 122 | 94.26% | 85% |
| Python | statements | 230 / 235 | 97.87% | 85% |
| Python | branches | 66 / 72 | 91.67% | 80% |

Separate diff coverage is not available. The changed production loader module
has 130/131 covered statements and 56/58 covered branches.

The checksum/path replacement regression passes for both CSV and Parquet. It
captures source bytes, replaces the path before parsing, and verifies that the
loaded values and SHA-256 identity both come from the captured snapshot.

## Architecture acceptance-criteria audit

The current implementation has evidence for all 11 architecture criteria:

| Criterion | Result | Primary evidence |
|---|---|---|
| AC-01 workspace/package | PASS | workspace build, maturin install/import, coverage gates |
| AC-02 Black–Scholes | PASS | Rust pricing reference/parity/expiry/error tests |
| AC-03 minute validation | PASS | Rust engine-validation and Python loader rejection tests |
| AC-04 24-hour lifecycle | PASS | boundary-order, 1,441/2,881-point, and incomplete-tail tests |
| AC-05 IV causality | PASS | causal shock/full-repricing/restart test |
| AC-06 70/30 strategy | PASS | sizing boundaries and full/reduced/rejected reserve tests |
| AC-07 accounting/margin | PASS | hand-ledger and expiry-breach attribution tests |
| AC-08 PnL/drawdown | PASS | hand-calculated ledger and negative-equity drawdown tests |
| AC-09 native/Python boundary | PASS | one-call, dtype/nullability, and native-error tests |
| AC-10 determinism/tests | PASS | bitwise repeat tests, full gates, coverage, million-point benchmark |
| AC-11 honest reporting | PASS | metadata/model-label contract test and explicit limitations in this report |

This evidence audit supports the final iteration-3 acceptance decision.

## Data validation

### Approved OKX minute dataset

The successful final scenario run used:

```text
data/okx_btc-usdt-swap_1m_2026-08-24_2026-08-27.csv
SHA256 87e0c1b86fa8c34f18ca916d18f69e6e5b21da27d40fdbc839cb74ee4306d4c5
```

The strict loader confirmed 4,321 completed UTC minute rows, exact nine-column
schema and dtypes, exact 60-second cadence, no duplicate timestamps, no gaps,
finite positive closes and `confirm=1` for every row. The dataset covers three
complete non-overlapping 24-hour trades and preserves the checksum in its
dataset identity.

The reproducible command was:

```bash
uv run python scripts/run_okx_sample.py \
  data/okx_btc-usdt-swap_1m_2026-08-24_2026-08-27.csv \
  --sha256 87e0c1b86fa8c34f18ca916d18f69e6e5b21da27d40fdbc839cb74ee4306d4c5 \
  --output target/okx-sample-run
```

Result-table counts were 12,963 equity rows, 9 completed trades, 9 executed
tranches, 6 reserve attempts and 3 scenario summaries. Trade PnL, terminal
equity, locked margin, row counts and reserve counts reconciled to the native
summary with zero delta in every scenario.

| Scenario | Final equity | Total PnL | Maximum drawdown USD | Minimum equity | Reserve attempts |
|---|---:|---:|---:|---:|---:|
| baseline | 536,420.582192 | 535,420.582192 | 117,976.544479 | -20,965.589666 | 0 |
| stress_2x | 536,420.582192 | 535,420.582192 | 461,471.713984 | -218,821.714026 | 3 rejected |
| stress_3x | 536,420.582192 | 535,420.582192 | 923,007.578226 | -680,357.578268 | 3 rejected |

All scenarios breached simplified model margin during the run. The engine
reports a breach but intentionally has no liquidation model. Identical terminal
PnL is expected here: IV changes the intermediate synthetic marks, all reserve
attempts are rejected, and expiry payoff does not depend on IV.

### Tardis raw trades dataset

The available Tardis file is:

```text
data/okex-swap_trades_2023-01-01_BTC-USDT-SWAP.csv.gz
SHA256 dc5491d023d224ee7eb5db5e11a0fd9a04af106f119e58573b18f04c8087c6bf
```

It contains 71,130 raw trades, not completed minute candles. The trades occupy
1,437 minute buckets and contain three minute gaps. The strict OKX minute loader
rejects the gzip/raw-trade format. An independent read-only test of a temporary
decompressed copy through the generic loader also rejects it at the second row
because trade events do not have exact 60-second cadence.

This rejection is the required safe result. Aggregating and filling this file
would change the approved input contract and could introduce hidden repair.
The source file checksum remained unchanged during all audits.

## Performance

Performance was measured locally on macOS/aarch64 with 12 logical CPUs using a
release build. The benchmark is an in-process library benchmark, not an HTTP or
multi-user service load test.

### Rust core benchmark

Each iteration-1 measurement processed 1,000,801 minute points and completed
695 trades in one baseline scenario. The iteration-2 verification run processed
the same 1,000,801 points and 695 trades in 0.109351 seconds, or 9,152,196
points/second.

| Run | Elapsed seconds | Points per second |
|---:|---:|---:|
| 1 | 0.107676 | 9,294,574 |
| 2 | 0.107498 | 9,309,975 |
| 3 | 0.107589 | 9,302,054 |
| 4 | 0.109933 | 9,103,760 |
| 5 | 0.105480 | 9,488,053 |

- mean: 0.107635 seconds and 9,299,683 points/second;
- median throughput: 9,302,054 points/second;
- observed range: 9,103,760 to 9,488,053 points/second.

No machine-specific performance threshold is part of the MVP. These numbers
are a local capacity observation and not a cross-platform guarantee.

### Python-to-Rust real-data run

Five warm local CLI runs loaded and validated all 4,321 rows, constructed
pandas inputs, invoked Rust for all three scenarios, constructed result tables,
performed reconciliation and serialized the JSON summary. Output was redirected
to avoid terminal rendering cost; CSV export was not included.

- wall-clock samples: 0.41, 0.42, 0.42, 0.43 and 0.43 seconds;
- mean: 0.422 seconds;
- median: 0.42 seconds;
- observed range: 0.41 to 0.43 seconds;
- measured maximum resident set size in a separate run: 107,249,664 bytes,
  approximately 102.3 MiB.

For this small dataset, Python startup, CSV parsing, pandas conversion and JSON
serialization dominate; the native minute loop is not the bottleneck.

## Quality and maintainability

The repository quality command enforces automatic source discovery, a maximum
of 500 physical lines per Rust/Python/shell source file, Rust function length
80, cognitive complexity 15, nesting depth 4, and Python complexity/nesting
limits 15/4. The largest audited executable file has 455 lines. The same
quality command is referenced by CI.

## Residual model risks

- `base_iv=0.55`, initial capital 1,000 model USD and
  `margin_per_straddle_usd=100` are illustrative assumptions;
- the input is a three-day perpetual-close proxy, not historical spot/index,
  forward, option quotes or historical IV;
- fees, spread, slippage, liquidity, funding, basis, strike grids, exchange
  margin and liquidation are not modelled;
- the negative intratrade equity and margin breaches make the current PnL
  unsuitable for an economic or live-trading decision;
- remote CI and cross-platform bit equality were not rerun in this local audit.

Iteration 2 ended with the QA findings recorded above. In iteration 3, the
developer corrected those documentation defects, QA passed, and the reviewer
returned `CHANGES_REQUESTED` for translation fidelity and report-status drift.
The developer remediation was independently rechecked and the reviewer returned
`APPROVED`; the master therefore accepted EPIC-003. The trading hypothesis is
not validated by this run and requires a broader dataset plus source-backed IV
and margin assumptions before economic conclusions.

For the complete operational and research workflow, see
[`../guides/run_backtest_and_validate_strategy.md`](../guides/run_backtest_and_validate_strategy.md).

## Repository state

Root `data/` is ignored and must not be committed; the local datasets remain
unchanged. Git integration identifiers are reported in the delivery handoff
rather than embedded in this pre-integration evidence report.
