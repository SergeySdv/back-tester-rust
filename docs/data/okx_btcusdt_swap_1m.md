# OKX BTC-USDT-SWAP one-minute data contract

## Purpose and boundary

This is the canonical input format for the MVP underlying path. Each accepted
row is one completed UTC minute for the `BTC-USDT-SWAP` perpetual instrument;
only its `close` enters the Rust backtest as the underlying price. The dataset
does not contain option quotes, trades, fills or historical IV. Options remain
synthetic Black–Scholes instruments.

The source-backed audit and scenario-run evidence are recorded separately in
[`../research/okx_1m_integration_evidence.md`](../research/okx_1m_integration_evidence.md).

## Approved runtime CSV

The header must contain these nine columns in this exact order:

```text
timestamp_ms,open,high,low,close,volume_contracts,volume_base,volume_quote,confirm
```

The approved sample has the following logical and pandas representation:

| Column | pandas dtype | Unit / meaning |
|---|---|---|
| `timestamp_ms` | `int64` | Unix timestamp in milliseconds, interpreted as UTC |
| `open` | `float64` | opening BTC-USDT-SWAP price in USDT per BTC |
| `high` | `float64` | highest price in USDT per BTC |
| `low` | `float64` | lowest price in USDT per BTC |
| `close` | `float64` | closing price in USDT per BTC; the only price used by the model |
| `volume_contracts` | `float64` | traded contracts reported for the candle |
| `volume_base` | `float64` | reported base volume in BTC |
| `volume_quote` | `float64` | reported quote volume in USDT |
| `confirm` | `int64` | OKX completion flag; must equal `1` on every row |

`load_okx_history_candles` requires the exact header, dtypes and completion
flag. It maps `timestamp_ms * 1_000_000` to contiguous `int64 timestamps_ns`
and passes `close` as contiguous `float64`. Nanoseconds remain Unix UTC; no
timezone conversion is inferred from a filename.

## Provenance and preparation

The recorded source for the approved sample is the official OKX public REST v5
`GET /api/v5/market/history-candles` endpoint with:

- `instId=BTC-USDT-SWAP`;
- `bar=1m`;
- `limit=300`, with pagination.

The recorded API pages were newest-first. Preparation reversed the complete
rows once into ascending source-timestamp order without changing values.
Runtime loading never sorts, forward-fills, deduplicates, aggregates or repairs
the file.

## Approved sample identity and coverage

- local filename:
  `data/okx_btc-usdt-swap_1m_2026-08-24_2026-08-27.csv`;
- SHA256:
  `87e0c1b86fa8c34f18ca916d18f69e6e5b21da27d40fdbc839cb74ee4306d4c5`;
- rows: 4,321;
- inclusive coverage: `2026-08-24T00:00:00Z` through
  `2026-08-27T00:00:00Z`;
- source timestamps: `1787529600000` through `1787788800000`;
- quality: exact 60,000 ms cadence, zero gaps, zero duplicates, finite positive
  closes and all 4,321 `confirm` values equal to `1`.

The loader stores this deterministic identity in result metadata:

```text
dataset_id = okx-rest-v5:BTC-USDT-SWAP:1m:sha256:<verified_sha256>
source = OKX public REST v5 GET /api/v5/market/history-candles
symbol = BTC-USDT-SWAP
interval_seconds = 60
timezone = UTC
```

## Validation and rejection rules

An approved minute series must contain at least 1,441 rows. The loader rejects:

- a checksum mismatch or any missing, additional or reordered runtime column;
- a dtype or timestamp-unit mismatch;
- `confirm != 1`, including incomplete final candles;
- fewer than 1,441 rows;
- gaps, duplicates, out-of-order timestamps or any step other than exactly
  60 seconds;
- timestamp conversion outside the `int64` nanosecond range;
- NaN, infinity, zero or negative `close` values;
- missing/empty dataset identity, source or symbol, a non-60-second interval,
  or a timezone other than UTC.

Invalid input produces an error and no partial result. The loader never repairs
the input silently.

## Tardis raw trades are not minute input

The read-only gzip sample
`data/okex-swap_trades_2023-01-01_BTC-USDT-SWAP.csv.gz` has this exact raw-trade
schema:

```text
exchange,symbol,timestamp,local_timestamp,id,side,price,amount
```

| Column | sample pandas dtype | Logical content |
|---|---|---|
| `exchange` | `object` | recorded exchange label |
| `symbol` | `object` | recorded instrument symbol |
| `timestamp` | `int64` | trade-event Unix timestamp in microseconds |
| `local_timestamp` | `int64` | recorded local Unix timestamp in microseconds |
| `id` | `int64` | trade identifier |
| `side` | `object` | recorded trade side |
| `price` | `float64` | trade price |
| `amount` | `int64` | recorded trade amount |

Its SHA256 is
`dc5491d023d224ee7eb5db5e11a0fd9a04af106f119e58573b18f04c8087c6bf`.
The 71,130 raw trades occupy 1,437 local-time minute buckets with three gaps.
There may be many or no trades in a minute, and the file has no completed
one-row-per-minute candle contract. It cannot enter the minute loader directly.
The MVP does not aggregate it and never forward-fills its missing buckets.

## Loader example

Local datasets remain untracked and uncommitted under `data/`; they are not
package fixtures or repository artifacts and must never be added to Git.

```python
from back_tester import load_okx_history_candles

minutes = load_okx_history_candles(
    "data/okx_btc-usdt-swap_1m_2026-08-24_2026-08-27.csv",
    expected_sha256=(
        "87e0c1b86fa8c34f18ca916d18f69e6"
        "5b21da27d40fdbc839cb74ee4306d4c5"
    ),
)
```

## Model limitation and future datasets

The integration evidence uses illustrative `base_iv=0.55` and simplified
`margin_per_straddle_usd=100`. These are neither observed historical IV nor an
OKX margin formula. A successful load or scenario run validates integration,
not strategy profitability, historical option execution, exchange fidelity or
live readiness.

Before another dataset is approved, its handoff must provide:

- exact source and acquisition query, symbol, interval and UTC semantics;
- immutable filename or dataset reference plus SHA256 identity;
- exact ordered schema, pandas dtypes, timestamp/price/volume units and mapping;
- inclusive coverage, row count, completion rule and any documented preparation;
- measured gaps, duplicates, ordering violations and invalid-price counts;
- evidence that it passes the same no-repair loader checks;
- explicit IV, rate, margin and proxy assumptions for any associated run.
