# BTC 24h Black–Scholes: Rust MVP fit-gap

- Date of inspection: 2026-08-26
- Repository: current repository

## Conclusion

The audit was completed and the architecture decision was settled: the
computational core would be built in Rust, while data loading, scenario
orchestration, and reporting would remain in Python.

At the time of this audit, the repository contained no `Cargo.toml`, Rust code,
Python package, tests, or confirmed OKX BTCUSDT minute data. Earlier conclusions
about a completed C++ replay/PnL engine and passing CTest/Pytest runs applied to
the separate `back-tester-2026` repository, not to this project.

Approved implementation brief:
[`../../architecture/01_btc_24h_rust_python_mvp.md`](../../architecture/01_btc_24h_rust_python_mvp.md).
Canonical implementation stages:
[`../../epics/README.md`](../../epics/README.md).

## What can be reused

The design reused proven architectural principles from `back-tester-2026`, but
not a runtime dependency on it or a direct copy of its C++ code:

- causal processing of events in a strict temporal order;
- numeric types on the hot path;
- fail-fast validation of input data;
- separation of native core and Python orchestration;
- explicit positions, PnL and immutable results;
- testable reproducibility and absence of a Python call every minute.

Files under `docs/archive/source` are preserved as references to the original
Homework 4 assignment, not as the Rust MVP implementation contract.

## Readiness matrix

| Area | Status | Next step |
|---|---|---|
| Rust workspace | No | Create a workspace and `backtest-core` |
| Immutable epic briefs | Ready | Implement EPIC-001..003 sequentially |
| Coverage tooling contract | Documented | Configure and pin versions in EPIC-001 |
| PyO3/maturin boundary | No | Create a separate binding crate |
| OKX minute loader | No | Document the actual CSV/Parquet schema |
| Black–Scholes | No | Implement call/put pricing, expiry, and tests |
| 24h ATM lifecycle | No | Implement fixed strike and expiry |
| IV baseline/2x/3x | No | Implement a causal jump after trade entry |
| 70/30 reserve rule | No | Implement one trigger per trade |
| Simplified margin | No | Add a constant margin-per-straddle ledger |
| Equity/PnL/drawdown | No | Add minute series and summary |
| Python reporting | No | Return pandas equity/trades/summary |
| Historical option data | Deferred | Consider only after the MVP result |
| Live integration | Outside MVP | Separate stage after validation |

## Model boundary

- The minute perpetual close defines the underlying path, but not historical
  option bid/ask, liquidity, IV surface, or execution.
- Applying a current average IV to older history produces a scenario calculation,
  not a historical estimate of observed IV.
- Both option legs are ATM only at entry; the strike is then fixed.
- Stress `2x/3x` is a full Black–Scholes repricing, not a linear vega
  adjustment.
- 70% and 30% are locked-margin budgets, not premium or notional allocations.
- The first margin model is linear in model USD and does not claim conformity
  with Deribit, Bybit, Binance, or OKX formulas.

## Data blocker at the time of the audit

A real run required a representative OKX minute-history file with confirmed:

- names and types of columns;
- timestamp and timezone units;
- symbol and market;
- coverage period;
- policy of omissions and duplicates.

The missing file blocked only exact OKX loader mapping, a data-quality audit of
the real dataset, and the final scenario run. It did not block scaffold,
pricing, lifecycle, accounting, or a generic Python boundary using synthetic
fixtures. Those fixtures could validate the model implementation, but could not
by themselves confirm the trading hypothesis.

## Research required before drawing a conclusion about the hypothesis

This did not block `EPIC-001` or `EPIC-002`, but was mandatory before making a
meaningful decision from real-data backtest results:

- obtain a representative OKX minute file and verify the actual schema,
  timestamp units, timezone, gaps and symbol;
- record the current-IV source, snapshot time, expiry/moneyness universe, and
  rule used to calculate the average `base_iv`;
- justify the `margin_per_straddle_usd` range and run sensitivity analysis,
  without calling the simplified value a Deribit/Bybit/Binance/OKX initial
  margin requirement;
- separately evaluate the effect of using `perpetual close` as a proxy instead
  of spot/index/forward if the synthetic MVP produces an attractive result;
- collect or buy option history only if this scenario stage justifies the
  additional accuracy and cost.

## Testing this change

This was a documentation-only change. A Rust workspace and executable tests did
not yet exist, so build/test was not applicable to that audit. Criteria for
future builds, tests, and handoffs were recorded in the implementation brief.
