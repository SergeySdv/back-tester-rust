# EPIC-002: deterministic lifecycle, accounting and result buffers

- Status: `READY_AFTER_EPIC-001`
- Brief version: `1.0`
- Dependency: `EPIC-001` accepted
- External blocker: no; synthetic fixtures are used

## Result

The Rust core runs a causal scenario backtest of a 24-hour synthetic short BTC
ATM straddle, implements the 70/30 reserve rule and cash/liability/margin accounting, and
returns all typed columnar buffers defined by the architecture
contract.

## In scope

- validation of minute arrays and mandatory `DatasetMetadata`;
- non-overlapping 24h lifecycle and exact same-timestamp event order;
- baseline/2x/3x IV state, one reserve attempt, and full repricing;
- integer quantity step counts, 70/30 budgets and free-margin cap;
- cash, liability, locked/available margin, settlement, PnL and drawdown;
- incomplete tail, typed short-history/initial-capital errors and
  `capital_exhausted` terminal result;
- equity, completed-trades, executed-tranches, reserve-attempts, summary and run
  metadata buffers with canonical order and validity masks;
- deterministic fixtures and million-minute benchmark harness.

## Out of scope

- PyO3/pandas conversion over scaffold, CSV/Parquet loader and real OKX mapping;
- bid/ask, fees, slippage, liquidation and exchange margin;
- historical option replay, IV surface, overlapping positions and live trading.

## Acceptance criteria

- `E2-01`: data shorter than 1,441 points gives `InsufficientHistory`; gaps,
  duplicates, disorder, length mismatch, cadence not 60 seconds, metadata
  timezone not UTC and invalid price are rejected without partial results.
- `E2-02`: entry uses only the current close, `K=S_entry`, with expiry exactly
  24 elapsed hours later; the reserve retains the strike and expiry.
- `E2-03`: at a shared boundary, settlement/realized PnL/margin release occur first,
  then a new entry, then one post-event row; 2,881 points give exactly two
  completed trades.
- `E2-04`: exactly 1,441 points do not create an incomplete tail; a subsequent incomplete tail
  is excluded from the equity series and is counted once.
- `E2-05`: baseline is constant; the 2x/3x shock is causal, does not affect pre-shock rows,
  and fully reprices both legs; the shock clock restarts at each trade entry,
  and the new boundary entry again uses base IV.
- `E2-06`: quantities are stored as `u64` step counts; values directly
  below/on/above boundaries `quantity_step=0.1` do not exceed budget over
  documented tolerance, and overflow/non-finite products give typed
  errors.
- `E2-07`: the first zero-sized initial entry gives `InsufficientInitialCapital`;
  after a completed trade, if a full next window exists, the engine returns a
  valid `capital_exhausted` result.
- `E2-08`: the reserve trigger is inclusive at 1.5x and fires once; full,
  reduced, and rejected attempts are explicit, and a rejected attempt does not create a tranche.
- `E2-09`: `equity=cash-liability` and
  `available_margin=equity-locked_margin`; entry itself does not change equity,
  margin is released at expiry.
- `E2-10`: PnL, return and running-peak drawdown are the same as manual fixtures;
  negative equity allows drawdown above 100%.
- `E2-11`: all tables, dtypes, nullability, enum values, row ordering, counts and
  metadata correspond exactly to section 10 of the architecture document.
- `E2-12`: a repeated run reproduces integer/result ordering and bitwise-equal
  floating arrays; all quality/coverage gates pass, and a benchmark for 1 million
  minutes records the environment and the result without hard performance threshold.

## Required fixtures

- 1,440, 1,441, 1,442, 2,880 and 2,881 minute points;
- flat, rising, falling and negative-equity price paths;
- shocks before/on reserve boundary, plus 2x and 3x;
- full/reduced/rejected reserve and later capital exhaustion;
- decimal-looking quantity boundaries and hand-calculated ledger path.

## Handoff in EPIC-003

Pass one Rust run entry point, typed column buffers/validity masks and
verified schemas. The Python layer must not repeat this state machine.
