# EPIC-003: Python boundary, data integration and reporting

- Status: `ACCEPTED`
- Brief version: `1.0`
- Dependency: `EPIC-002` accepted
- External blocker: resolved; an approved representative OKX CSV supplies
  evidence for `E3-08..E3-10`

## Result

Python loads and validates minute data, launches scenarios in Rust through one
bulk call, and provides stable pandas tables and metadata. A real OKX sample has
an exact documented mapping and produces an auditable baseline/2x/3x scenario
run.

## In scope

- typed Python config, dataset metadata and fixed scenario API;
- contiguous NumPy input and one PyO3 call without per-minute callback;
- translation of Rust buffers/validity masks into pandas with contract dtypes/order;
- CSV/Parquet loader with explicit, source-backed mapping;
- saving run config, model ID/version and dataset identity;
- synthetic end-to-end tests and real-data data-quality/run handoff;
- minimum comparison tables/export required for PnL/drawdown review.

## Out of scope

- duplication of pricing/state machine in Python;
- guessing OKX schema by filename;
- download service, exchange connector, live orders and credentials;
- historical option quotes/IV, dashboard and generic plugin framework.

## Acceptance criteria

- `E3-01`: public Python API accepts contiguous `int64 timestamps_ns`,
  `float64 close`, `DatasetMetadata`, config and fixed scenarios with one run call.
- `E3-02`: dtype/length/contiguity errors and native typed errors become
  actionable Python exceptions without partial result.
- `E3-03`: equity, trades, tranches, reserve attempts and summary DataFrames have
  exact column order/dtypes/nullability; nulls are not represented by NaN/negative-ID
  sentinels.
- `E3-04`: scenario/table ordering and repeated-call results are deterministic;
  Python does not recalculate the financial state machine.
- `E3-05`: synthetic end-to-end test at 2,881 points completes two trades and
  checks table counts, final equity, PnL, margin and metadata with native result.
- `E3-06`: the loader rejects missing columns, invalid dtype/unit/timezone,
  gaps/duplicates/out-of-order data, NaN/infinity, and non-positive close; it
  does not sort, fill, or silently repair the data.
- `E3-07`: each output is marked as
  `synthetic Black–Scholes scenario backtest` and does not claim historical
  option execution or exchange-margin fidelity.
- `E3-08`: the representative OKX sample documents the exact source, symbol,
  interval, timezone, timestamp unit, price column, coverage, and mapping; these
  values are not inferred from unchecked filename.
- `E3-09`: approved sample passes data-quality checks; its dataset ID and
  checksum/identity are saved with the result.
- `E3-10`: baseline, 2x, and 3x runs on the approved sample create auditable PnL,
  drawdown, margin, and reserve outputs, and the summaries reconcile.
- `E3-11`: format/lint/tests and Rust/Python coverage gates pass; packaging
  reproduced from clean project environment.

## External blocker rule and resolution

Work through `E3-07` may be accepted as a separate synthetic-fixture feature. Full
`EPIC-003` and the full MVP cannot be accepted without `E3-08..E3-10`. The absence
of a sample does not permit inventing expected column names, timestamp units, symbol,
or timezone; manager returns `BLOCKED_EXTERNAL` only for the real-data part.

That historical blocker is now resolved. The approved representative OKX
history-candles sample has a source-backed schema, mapping, checksum-derived
dataset identity, strict data-quality evidence, and reconciled baseline/2x/3x
run evidence for `E3-08..E3-10`. Epic acceptance still requires the normal
developer, QA, and reviewer gates; resolving the external blocker does not by
itself declare the epic accepted.

## Required handoff evidence

- exact install/build/test/coverage commands and environment versions;
- schema/dtype assertions and synthetic E2E results;
- after receiving sample: mapping document, data-quality summary, dataset
  identity and reconciled scenario summaries;
- an explicit list of scenario-model limitations before any decision about a live stage.
