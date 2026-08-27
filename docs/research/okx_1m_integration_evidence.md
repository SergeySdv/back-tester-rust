# OKX BTC-USDT-SWAP one-minute integration evidence

## Approved sample and mapping

The approved local sample is
`data/okx_btc-usdt-swap_1m_2026-08-24_2026-08-27.csv`. It was prepared
read-only from the official OKX public REST v5
`GET /api/v5/market/history-candles` endpoint with `instId=BTC-USDT-SWAP`,
`bar=1m`, `limit=300` pagination. The endpoint returns newest-first; preparation
reversed complete rows into ascending source timestamps without altering values.

- SHA256: `87e0c1b86fa8c34f18ca916d18f69e6e5b21da27d40fdbc839cb74ee4306d4c5`
- schema, in exact order: `timestamp_ms`, `open`, `high`, `low`, `close`,
  `volume_contracts`, `volume_base`, `volume_quote`, `confirm`;
- mapping: `timestamp_ms` integer milliseconds -> `timestamps_ns`; `close`
  `float64` -> underlying close;
- source/symbol/interval/timezone: official OKX public REST v5,
  `BTC-USDT-SWAP`, 60 seconds, UTC;
- coverage: 4,321 rows from `2026-08-24T00:00:00Z` through
  `2026-08-27T00:00:00Z`, inclusive;
- quality: exact 60,000 ms cadence, zero gaps, zero duplicate timestamps, all
  closes finite and positive, and all 4,321 `confirm` values equal `1`.

The result dataset ID is deterministic and contains the checksum:
`okx-rest-v5:BTC-USDT-SWAP:1m:sha256:87e0c1b86fa8c34f18ca916d18f69e6e5b21da27d40fdbc839cb74ee4306d4c5`.
`load_okx_history_candles` verifies the checksum, exact schema, completed-candle
flag and generic minute-series invariants. It does not sort, fill or repair.

## Rejected raw trades sample

`data/okex-swap_trades_2023-01-01_BTC-USDT-SWAP.csv.gz` is an official Tardis
downloadable raw trades file with SHA256
`dc5491d023d224ee7eb5db5e11a0fd9a04af106f119e58573b18f04c8087c6bf`.
Its schema is `exchange,symbol,timestamp,local_timestamp,id,side,price,amount`.
The 71,130 trades occupy only 1,437 local-time minute buckets and have three
gaps. Raw trades are not one row per minute and are rejected rather than
aggregated, sorted or filled by the minute loader.

## Reproducible scenario run

The integration run uses illustrative model assumptions: initial capital
1,000 model USD, annualized `base_iv=0.55`, `r=q=0`, simplified
`margin_per_straddle_usd=100`, `quantity_step=0.1`, and IV shocks after 720
minutes. The IV is not observed historical IV and the margin value is not OKX
margin. Consequently this is a `synthetic Black–Scholes scenario backtest`
that validates data/boundary/report integration, not the trading hypothesis,
historical option execution, exchange-margin fidelity or live readiness.

Run and export the auditable tables under ignored `target/`:

```bash
uv run python scripts/run_okx_sample.py \
  data/okx_btc-usdt-swap_1m_2026-08-24_2026-08-27.csv \
  --sha256 87e0c1b86fa8c34f18ca916d18f69e6e5b21da27d40fdbc839cb74ee4306d4c5 \
  --output target/okx-sample-run
```

The checked run processes 4,321 rows and completes three non-overlapping
24-hour trades in each scenario. Trade PnL sums, terminal equity, maximum
locked margin, completed-trade counts and reserve-attempt counts reconcile
exactly to the Rust summary:

| Scenario | Final equity / total PnL | Max drawdown USD / pct | Min equity | Max locked margin | Reserve attempts |
|---|---:|---:|---:|---:|---:|
| baseline | 536420.582192 / 535420.582192 | 117976.544479 / 3.974484 | -20965.589666 | 36220.0 | 0 |
| stress_2x | 536420.582192 / 535420.582192 | 461471.713984 / 4.454898 | -218821.714026 | 36220.0 | 3 rejected |
| stress_3x | 536420.582192 / 535420.582192 | 923007.578226 / 4.573518 | -680357.578268 | 36220.0 | 3 rejected |

The large, identical terminal PnL and negative intratrade equity are outputs of
the illustrative assumptions: the stress scenarios change synthetic marks but
their reserve sales are rejected, and settlement payoff is IV-independent.
They are not evidence that the assumptions or strategy are economically valid.
Generated CSV/JSON artifacts remain under `target/` and are not committed.
