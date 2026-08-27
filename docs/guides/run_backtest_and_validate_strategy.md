# Running the backtester and validating the strategy

## What this project can verify

The current project runs a deterministic synthetic Black–Scholes scenario
backtest for a 24-hour BTC ATM short straddle. It can verify the Rust/Python
implementation, data integration, scenario PnL, equity, drawdown and simplified
margin usage.

It does not replay historical option quotes or fills and does not implement an
exchange margin or liquidation engine. A technically successful run is not a
signal to trade live.

The canonical input contract is
[`../data/okx_btcusdt_swap_1m.md`](../data/okx_btcusdt_swap_1m.md). The latest
technical acceptance and performance evidence is in
[`../reports/final_mvp_validation_performance.md`](../reports/final_mvp_validation_performance.md).

## 1. Install the development environment

Run all commands from the repository root:

```bash
cd /path/to/back-tester-rust
```

Required tools:

- Rust stable with `rustfmt`, `clippy` and `llvm-tools-preview`;
- pinned nightly `nightly-2026-08-01` for Rust branch coverage;
- `uv` 0.7.13;
- `cargo-llvm-cov` 0.9.0.

One-time setup:

```bash
rustup component add rustfmt clippy llvm-tools-preview
rustup toolchain install nightly-2026-08-01 \
  --profile minimal --component llvm-tools-preview
cargo install cargo-llvm-cov --version 0.9.0 --locked
uv sync --locked --group dev
uv run maturin develop --locked
```

Confirm that the native package imports:

```bash
uv run python -c "import back_tester; print(back_tester.__file__)"
```

Re-run `uv run maturin develop --locked` after changing Rust code so Python
uses the current native extension.

## 2. Verify the project before a research run

Fast quality and test checks:

```bash
./scripts/quality.sh
cargo fmt --all -- --check
cargo test --workspace --all-features
uv run pytest -q
```

Full coverage acceptance:

```bash
uv run bash scripts/coverage.sh
```

The full command is slower because it builds instrumented stable and nightly
Rust binaries. It must finish with all four thresholds passing:

- Rust lines at least 90%;
- Rust branches at least 85%;
- Python lines at least 85%;
- Python branches at least 80%.

Performance smoke test:

```bash
cargo bench -p backtest-core --bench million
```

This must process at least one million points. Throughput is recorded but has
no machine-independent acceptance threshold.

## 3. Add a dataset

Keep local market data under `data/`. The directory is intentionally untracked;
never commit datasets, credentials or download tokens.

### Approved OKX candle format

The strict ready-to-run path accepts CSV or Parquet with exactly these columns
and order:

```text
timestamp_ms,open,high,low,close,volume_contracts,volume_base,volume_quote,confirm
```

Requirements:

- `timestamp_ms` and `confirm` are `int64`;
- all price and volume columns are `float64`;
- every `confirm` equals `1`;
- rows are ascending UTC minutes with an exact 60-second step;
- `close` is finite and strictly positive;
- there are at least 1,441 rows, including entry and expiry boundaries;
- there are no gaps, duplicates or out-of-order timestamps.

The loader never sorts, fills, deduplicates or repairs input. Prepare a source
artifact explicitly, audit the preparation outside the runtime loader, and
record its origin, query, coverage, schema and checksum.

Calculate its identity:

```bash
shasum -a 256 data/YOUR_OKX_FILE.csv
```

Then use that exact hash in every run. A changed file must have a new dataset
identity and be reviewed again.

### Tardis data

The current Tardis sample is raw trades, not minute candles. It has multiple
events per minute, only 1,437 occupied minute buckets and three gaps. Passing it
directly to the backtester must fail.

Do not forward-fill or silently aggregate it. To use Tardis in a future data
pipeline, obtain a source-backed completed 1-minute candle product or create a
separately reviewed conversion specification covering:

- event timestamp used for bucketing and UTC boundary semantics;
- OHLC construction and volume units;
- policy for minutes without trades;
- partial first/last minutes;
- duplicate and out-of-order events;
- deterministic output checksum and a quality report.

That conversion is not implemented by this MVP. A filled synthetic minute
series must not be described as unchanged historical Tardis candles.

### A different reviewed CSV or Parquet schema

Use the generic loader only when the mapping and metadata are independently
known, not inferred from the filename:

```python
from back_tester import ColumnMapping, DatasetMetadata, load_minutes

minutes = load_minutes(
    "data/reviewed_minutes.parquet",
    mapping=ColumnMapping(
        timestamp_column="timestamp",
        close_column="close",
        timestamp_unit="ms",
    ),
    metadata=DatasetMetadata(
        dataset_id="provider:symbol:1m:sha256:FULL_HASH",
        source="exact provider and endpoint/product",
        symbol="exact source symbol",
        interval_seconds=60,
        timezone="UTC",
    ),
)
```

The generic loader still enforces dtype, minimum history, exact cadence,
ordering and price validity. It does not verify a checksum itself; the calling
workflow must do so before loading.

## 4. Run the approved real-data example

The repository includes a reproducible runner for the strict OKX format:

```bash
uv run python scripts/run_okx_sample.py \
  data/okx_btc-usdt-swap_1m_2026-08-24_2026-08-27.csv \
  --sha256 87e0c1b86fa8c34f18ca916d18f69e6e5b21da27d40fdbc839cb74ee4306d4c5 \
  --output target/okx-sample-run
```

The runner validates the checksum and source schema, runs `baseline`,
`stress_2x` and `stress_3x`, reconciles the tables and exports:

```text
target/okx-sample-run/equity.csv
target/okx-sample-run/trades.csv
target/okx-sample-run/tranches.csv
target/okx-sample-run/reserve_attempts.csv
target/okx-sample-run/summary.csv
target/okx-sample-run/metadata.json
```

The example uses fixed illustrative assumptions:

```text
initial_capital_usd = 1000
base_iv = 0.55
risk_free_rate = 0
carry_rate = 0
margin_per_straddle_usd = 100
quantity_step = 0.1
shock_after_minutes = 720
```

They validate integration only. They are not observed historical IV, funding,
carry or OKX margin values.

## 5. Run a reviewed configuration from Python

For a different research configuration, create a small orchestration script;
do not reproduce pricing or the strategy state machine in Python:

```python
from pathlib import Path

from back_tester import BacktestConfig, IvScenario, load_okx_history_candles, run

sample = Path("data/reviewed_okx_1m.csv")
checksum = "FULL_64_CHARACTER_SHA256"

minutes = load_okx_history_candles(sample, expected_sha256=checksum)
result = run(
    timestamps_ns=minutes.timestamps_ns,
    close=minutes.close,
    dataset=minutes.metadata,
    config=BacktestConfig(
        initial_capital_usd=1_000.0,
        base_iv=0.55,
        margin_per_straddle_usd=100.0,
        quantity_step=0.1,
        risk_free_rate=0.0,
        carry_rate=0.0,
    ),
    scenarios=[
        IvScenario.baseline(),
        IvScenario.stress_2x(720),
        IvScenario.stress_3x(720),
    ],
)
result.export_csv(Path("target/research-run"))
print(result.summary_df.to_string(index=False))
```

Every comparison run must retain `metadata.json`, effective configuration,
dataset checksum and exported audit tables. Never overwrite evidence from a
previous configuration; use a distinct output directory.

## 6. Read and reconcile the result

Start with `summary.csv`, then investigate the underlying tables.

| Table | What to inspect |
|---|---|
| `summary.csv` | terminal status, final equity, PnL, return, maximum drawdown, minimum equity, margin breach and counts |
| `equity.csv` | minute path, liability, locked/available margin, drawdown timing and breach duration |
| `trades.csv` | entry equity, sizing, premium, expiry payoff and realized PnL per 24-hour trade |
| `tranches.csv` | initial/reserve execution, quantity, IV, premium and locked margin |
| `reserve_attempts.csv` | full/reduced/rejected reserve attempts and limiting reason |
| `metadata.json` | exact dataset, model, configuration, scenarios and software identity |

Mandatory reconciliation checks:

- `final_equity = initial_equity + total_pnl`;
- sum of completed-trade `realized_pnl` equals summary `total_pnl`;
- last equity row equals summary `final_equity`;
- maximum equity-table drawdown and margin values equal summary maxima/minima;
- table counts equal summary counts;
- `equity = cash - option_liability` on every row;
- `available_margin = equity - locked_margin` on every row;
- repeated identical runs return equal tables.

`scripts/run_okx_sample.py` performs the principal summary reconciliations and
fails rather than exporting a successful claim when they disagree.

## 7. Validate the trading hypothesis

Use the following sequence. Do not skip directly from one three-day synthetic
run to live capital.

### Stage A: implementation acceptance

Require all quality, test and coverage gates to pass, exact dataset identity,
deterministic repeated output and zero reconciliation differences. This stage
is complete for the audited MVP.

### Stage B: data and assumption acceptance

Before evaluating profit, obtain a much broader representative history and
record, without look-ahead:

- historical IV source and the method used to select `base_iv` at each test;
- justified ranges for risk-free rate, carry/funding and margin per straddle;
- the effect of perpetual close versus spot, index or forward;
- fees, spread, slippage and executable strike/size constraints;
- periods of high volatility, crashes, rallies and quiet markets.

The current constant-IV engine can be used for a transparent sensitivity grid,
but it cannot claim that the chosen IV was observable historically. Run each
assumption set into a separate output directory and compare robustness, not
only the best result.

### Stage C: predeclare decision criteria

Choose thresholds before inspecting the final out-of-sample period. At a
minimum define:

- maximum tolerable drawdown in USD and percent;
- whether any negative equity or margin breach is an automatic rejection;
- minimum number of independent completed 24-hour trades;
- required out-of-sample PnL/return after realistic costs;
- maximum loss per trade and consecutive-loss tolerance;
- acceptable sensitivity to IV, margin, fees, execution time and data source.

Do not tune these thresholds after seeing results. The current audited example
has negative intratrade equity and margin breaches in every scenario, so it
must not pass a live-readiness criterion that forbids insolvency or requires
realistic liquidation handling.

### Stage D: chronological out-of-sample testing

Split data by time, never randomly:

1. use an earlier calibration period to select assumptions;
2. freeze configuration and decision criteria;
3. run one or more later untouched validation periods;
4. report profitable and losing subperiods, not only aggregate PnL;
5. repeat a walk-forward process without allowing future data into earlier
   decisions.

Because trades do not overlap and last 24 hours, evaluate the number of
completed trades as well as the number of minute rows. Three completed trades
are enough for integration testing and far too few for an economic conclusion.

### Stage E: execution-realism model

If the synthetic hypothesis survives Stage D, implement and test a separate
realism stage before paper trading:

- observed or defensibly reconstructed option IV/quotes;
- bid/ask spread, fees, slippage and order-size limits;
- exchange-specific initial/maintenance margin and liquidation;
- funding, basis, collateral currency and settlement details;
- real strike grid and instrument expiry rules.

Do not relabel the current Black–Scholes marks as historical fills.

### Stage F: paper trading

Paper trading is a separate project stage, not a mode of this repository.
It requires a connector, immutable market snapshots, clock/event audit,
simulated order lifecycle and daily reconciliation against the research model.

Run it long enough to cover multiple entries, expiries and stressed markets.
Investigate every difference between expected and simulated execution. A paper
result is acceptable only under the predeclared drawdown, margin and operational
limits.

### Stage G: controlled live pilot

Only consider live trading after independent review of the historical and paper
evidence. A live system additionally needs credentials isolation, least-privilege
API keys, position/notional limits, kill switch, monitoring, alerts, exchange
reconciliation, idempotent order handling and a tested incident procedure.

Start with capital whose complete loss is acceptable and hard limits materially
below the tested capacity. This MVP must not be connected directly to an
exchange.

## 8. Research-run checklist

Before every run:

- [ ] source, symbol, interval, timezone and preparation are documented;
- [ ] SHA256 is calculated and stored;
- [ ] exact schema/dtypes and `confirm` are verified;
- [ ] cadence, gaps, duplicates, ordering and prices pass without repair;
- [ ] at least one complete 1,441-point window exists;
- [ ] IV, rates, margin, quantity step and shock timing are justified;
- [ ] output directory is new and configuration is retained;
- [ ] project quality/tests pass on the exact code being used.

After every run:

- [ ] summary and detailed tables reconcile;
- [ ] repeated run is deterministic;
- [ ] margin breaches and negative equity are treated as risk failures, not
      hidden by terminal PnL;
- [ ] drawdown and worst individual trades are reviewed;
- [ ] results are compared across scenarios and chronological subperiods;
- [ ] limitations state that outputs are synthetic, not historical fills;
- [ ] no dataset, token, credential or generated report is accidentally staged.

## 9. Common failures

- `minute data path must end in .csv, .parquet, or .pq`: raw gzip/trade input is
  not an accepted minute artifact.
- `dataset SHA256 mismatch`: the file differs from the reviewed artifact; stop
  and re-audit it.
- `at least 1441 minute points required`: no complete entry-to-expiry window.
- `expected an exact 60-second step`: gap, duplicate, disorder or raw events;
  do not sort/fill silently.
- dtype error: preserve exact integer timestamps and `float64` prices instead
  of coercing ambiguous source data at runtime.
- stale native behavior after Rust edits: rerun
  `uv run maturin develop --locked`.

When a failure concerns source data, fix or replace the source preparation and
create a new reviewed artifact. Do not weaken the loader to make a dataset pass.
