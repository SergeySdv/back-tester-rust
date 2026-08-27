# BTC 24h ATM short straddle: Rust core and Python orchestration MVP

- Status: approved implementation brief
- Date: 2026-08-26
- Target repository: this repository

## 1. Goal

Build a deterministic research backtester that tests a short 24-hour BTC ATM
call-and-put strategy on one-minute OKX BTCUSDT perpetual prices.

The MVP does not require historical option quotes. It creates synthetic option
marks with Black–Scholes from:

- the minute close of the underlying;
- the strike fixed at entry;
- the exact remaining time to expiry;
- a configured base IV and deterministic IV-shock scenario.

The main output is an equity curve with PnL, maximum drawdown, margin usage and
a complete trade/tranche audit. A favorable result is a reason to improve data
and execution realism; it is not by itself permission to trade live.

## 2. Research boundary

This is a **scenario backtest**, not a historical option-market replay.

- Minute BTCUSDT perpetual data supplies only the underlying path.
- Black–Scholes supplies synthetic option marks.
- Current mean IV applied to old underlying history is an explicit scenario
  assumption, not an estimate of the IV that was observable then.
- The MVP must never label a synthetic mark as an observed option bid, ask,
  trade, fill or exchange mark.
- Historical option data becomes a separate follow-up only if the synthetic
  tests justify the added effort.

## 3. Frozen MVP decisions

| Area | Decision |
|---|---|
| Native core | Rust |
| Data loading, run orchestration and reporting | Python |
| Boundary | PyO3 extension built with maturin |
| Underlying input | One-minute OKX BTCUSDT perpetual bars |
| Underlying price | Minute `close` |
| Option | Synthetic European call plus put |
| Horizon | Exactly 24 elapsed hours |
| Strike | `K = S_entry`, without exchange strike-grid rounding |
| Position | Short call and short put with equal quantity |
| Rates | Black–Scholes accepts finite `risk_free_rate` and `carry_rate`; the baseline defaults both to zero |
| Base volatility | Required annualized decimal `base_iv`, for example `0.55` |
| Scenario set | A non-empty unique subset of `baseline`, `stress_2x`, `stress_3x` |
| Baseline | IV remains equal to `base_iv` and has no shock time |
| Stress | IV jumps after entry to exactly `2x` or `3x` and stays there until expiry |
| Initial allocation | 70% of each trade's entry equity as margin budget |
| Reserve allocation | 30% of that trade's entry equity as margin budget for one additional sale |
| Reserve trigger | `current_iv >= 1.5 * entry_iv` |
| Reserve instrument | Same call and put, strike and expiry as the initial sale |
| Margin | Constant configured margin per one complete straddle |
| Quantity unit | `1.0` means call plus put exposure on one BTC; internal quantities are integer step counts |
| Execution | Synthetic Black–Scholes mark; no book, spread or slippage |
| Positions | Non-overlapping 24-hour trades |
| Shared daily boundary | Settle the old trade, realize PnL and release margin, then enter the new trade, then emit one post-event row |
| Settlement currency | Model USD; linear PnL only |

ATM is guaranteed only at the initial entry. The strike stays fixed afterwards,
so the options can move ITM or OTM. The reserve tranche averages the entry
premium in the same contracts; it does not select a new ATM strike.

## 4. Architecture and ownership

```text
OKX CSV/Parquet
      |
      v
Python loader and validation
      |
      | contiguous timestamps_ns[] and close[] plus typed config
      v
PyO3 boundary (one call per run, no callback per minute)
      |
      v
Rust deterministic engine
  - Black–Scholes
  - option lifecycle
  - IV scenario
  - 70/30 strategy
  - cash, liability and margin ledger
  - equity, PnL and drawdown
      |
      | columnar result arrays and summary
      v
Python pandas/reporting layer
```

### Rust owns

- validation of all numerical values received across the native boundary;
- Black–Scholes pricing and expiry payoff;
- the minute-by-minute causal loop;
- scenario state and the one-time reserve trigger;
- contract sizing and quantity-step rounding;
- tranche, cash, liability, equity and margin state;
- PnL and drawdown calculations;
- deterministic typed result buffers.

### Python owns

- reading OKX CSV or Parquet;
- mapping the actual source schema to `timestamps_ns` and `close`;
- user-facing configuration and scenario selection;
- invoking the Rust engine once per run;
- conversion of result arrays to pandas objects;
- plots, comparison tables and report export;
- later integration with the user's universal exchange API.

Python must not execute a callback on every minute in this MVP. The strategy is
configured and launched in Python but its state machine runs in Rust.

## 5. Proposed repository layout

```text
AGENTS.md
Cargo.toml
pyproject.toml
crates/
  backtest-core/
    Cargo.toml
    src/
      lib.rs
      black_scholes.rs
      config.rs
      engine.rs
      error.rs
      iv_scenario.rs
      margin.rs
      option.rs
      portfolio.rs
      result.rs
      strategy.rs
  backtest-python/
    Cargo.toml
    src/lib.rs
python/
  back_tester/
    __init__.py
    config.py
    data.py
    reporting.py
    runner.py
  tests/
    test_data.py
    test_end_to_end.py
scripts/
  check_coverage.py
docs/
  architecture/
  epics/
  prompts/
  research/
  source/
```

`backtest-core` must not depend on Python, pandas, Arrow or exchange SDKs.

## 6. Input contracts

### 6.1 Minute series passed to Rust

```text
timestamps_ns: contiguous int64 Unix timestamps in UTC
close:         contiguous float64 BTCUSDT prices
```

Required invariants:

- arrays have equal length and contain at least 1,441 points;
- timestamps are strictly increasing;
- adjacent timestamps differ by exactly 60 seconds;
- prices are finite and strictly positive;
- no duplicate timestamps;
- the first complete 24-hour window contains 1,441 boundary points, including
  both entry and expiry timestamps;
- fewer than 1,441 points fail with `InsufficientHistory` and produce no
  result;
- after at least one completed trade, a trailing partial window is excluded
  from the equity series and counted once in result metadata.

Python may load `open`, `high`, `low` and `volume`, but the MVP passes only
`timestamp` and `close` to Rust. The final OKX column mapping cannot be frozen
until a representative source file is available.

### 6.2 Dataset metadata

Metadata crosses the Rust boundary with the price arrays and is returned
unchanged with every scenario result:

```text
dataset_id:       non-empty UTF-8 dataset identity
source:           non-empty UTF-8 source, for example "okx"
symbol:           non-empty exact source instrument identifier
interval_seconds: uint32, exactly 60 in the MVP
timezone:         UTF-8, exactly "UTC" in the MVP
```

The loader must obtain these values from the actual file/manifest or explicit
user input. It must not infer an unverified venue or symbol from a filename.

### 6.3 Backtest and scenario configuration

```text
initial_capital_usd:       finite, > 0
base_iv:                   annualized decimal, finite, > 0
risk_free_rate:            annualized decimal, finite; default 0.0
carry_rate:                annualized decimal, finite; default 0.0
initial_allocation:        exactly 0.70 in MVP
reserve_allocation:        exactly 0.30 in MVP
reserve_trigger_multiple:  exactly 1.50 in MVP
margin_per_straddle_usd:   finite, > 0
quantity_step:             finite, > 0
```

The scenario collection must be a non-empty subset of these fixed variants:

```text
baseline:  scenario_id="baseline",  multiplier=1.0, shock_after_minutes=null
stress_2x: scenario_id="stress_2x", multiplier=2.0, shock_after_minutes=1..1439
stress_3x: scenario_id="stress_3x", multiplier=3.0, shock_after_minutes=1..1439
```

Duplicate variants, custom multipliers, an empty collection, a baseline shock
time or a missing/invalid stress shock time fail before the run starts. Results
use canonical scenario order `baseline`, `stress_2x`, `stress_3x`, regardless of
the caller's input order.

## 7. Pricing contract

For `T > 0`, `S > 0`, `K > 0` and `sigma > 0`:

```text
d1 = (ln(S / K) + (r - q + 0.5 * sigma^2) * T)
     / (sigma * sqrt(T))
d2 = d1 - sigma * sqrt(T)

call = S * exp(-q * T) * N(d1) - K * exp(-r * T) * N(d2)
put  = K * exp(-r * T) * N(-d2) - S * exp(-q * T) * N(-d1)
```

`r` is `risk_free_rate` and `q` is `carry_rate`. The formula and unit tests must
support non-zero values. The initial scenario suite uses `r = 0` and `q = 0` as
an explicit short-horizon assumption, not as hard-coded pricing behavior.
Those defaults isolate the spot/IV/time hypothesis and avoid inventing
historical rate, funding or carry series that have not been supplied; they are
not claimed to be observed OKX or Deribit values. Sensitivity runs may pass
non-zero `r/q` without changing the pricing implementation.
BTCUSDT perpetual close is treated as a documented proxy for spot/index in this
MVP; funding, basis and Black-76 are deferred realism work.

`T` is measured as exact nanoseconds remaining divided by the chosen constant
number of seconds per year. The implementation must define and test that
constant once; the MVP uses `365 * 24 * 60 * 60` seconds.

At `T = 0`:

```text
call_payoff = max(S - K, 0)
put_payoff  = max(K - S, 0)
```

The engine always performs a full Black–Scholes revaluation. It must not use a
linear vega approximation for `2x` or `3x` stress.

## 8. Strategy timeline

For each complete, non-overlapping 24-hour window:

1. Read `S_entry` from the current minute without looking ahead.
2. Set `K = S_entry` and `expiry = entry + 24h`.
3. Price the call and put at `base_iv`.
4. Calculate initial quantity from 70% of equity available at this trade's
   entry.
5. Sell equal quantities of the call and put as one logical straddle tranche.
6. Mark both short legs on every subsequent minute with the active scenario IV
   and exact remaining time.
7. If IV reaches the 1.5x trigger and the reserve has not been used, attempt one
   reserve sale in the same strike and expiry, capped by margin still available
   at that minute.
8. At expiry, settle both legs at intrinsic value.
9. Carry resulting cash/equity into the next 24-hour window and size that next
   trade from its own entry equity.

Scenario IV is:

```text
baseline: IV(t) = base_iv

stress before shock: IV(t) = base_iv
stress_2x at/after shock: IV(t) = base_iv * 2
stress_3x at/after shock: IV(t) = base_iv * 3
```

Every stress run opens at `base_iv`. Starting a run at `2x` or `3x` would also
increase the premium received and would not test the loss caused by a later IV
jump.

The shock clock is per trade, not per whole dataset:
`shock_timestamp = trade_entry_timestamp + shock_after_minutes * 60s`. Every
new daily trade resets to `base_iv`, including a new trade opened on the same
timestamp at which the previous stressed trade expires.

### 8.1 Minute and same-timestamp event order

There is exactly one equity row per processed input timestamp per scenario. It
is always a post-event row. The causal order is:

1. read the current timestamp and spot;
2. if the active trade expires now, calculate intrinsic payoff from this spot,
   settle cash, realize trade PnL, clear liability and release its margin;
3. if a non-expiring trade remains active, activate this minute's scenario IV,
   reprice its tranches, update equity/free margin and perform the one allowed
   reserve attempt at the same mark;
4. if no trade is active, open the first or next trade at this timestamp only
   when a full following 24-hour window exists and initial quantity is non-zero;
5. emit one row containing the final post-event state.

At a shared daily boundary the old trade is therefore never marked alongside
the new one: settlement and margin release happen first. The new trade uses the
same timestamp's close, its own `K = S_entry`, `base_iv` and equity after the old
settlement. Opening it changes cash and liability equally, so boundary equity
does not jump solely because of entry.

Complete trade start indices are `0, 1440, 2880, ...`; adjacent trades share
one boundary timestamp. A 2-day synthetic fixture therefore has 2,881 points
and exactly two completed trades.

### 8.2 Incomplete tail and emitted prefix

- Fewer than 1,441 total points return `InsufficientHistory` with no result.
- Exactly 1,441 points produce one complete trade and no skipped tail.
- If strictly later timestamps exist after the last completed expiry but do not
  complete another 24-hour window, they are ignored, no flat equity rows are
  emitted for them, and `skipped_incomplete_window_count = 1`.
- The last emitted row is the completed expiry boundary. With no new complete
  window, it has no active trade and nullable `active_trade_id`/`active_iv`.
- Metadata reports total, processed and ignored input-row counts.

## 9. Quantity and margin model

Capital allocations are margin budgets, not option premium or notional.
`margin_per_straddle_usd` is the margin for `1.0` quantity, meaning one call
plus one put on one BTC. Call/put prices and expiry payoff are model USD per
`1.0` quantity and are multiplied by executed quantity.

Quantities are stored internally as integer `u64` step counts:

```text
entry_equity = equity at the start of the current 24-hour trade
initial_budget = entry_equity * 0.70
reserve_budget = entry_equity * 0.30

margin_per_step = margin_per_straddle_usd * quantity_step
initial_step_count = floor_steps(initial_budget / margin_per_step)
initial_quantity = initial_step_count * quantity_step

reserve_margin_available_at_trigger = max(
    min(reserve_budget, available_margin_before_reserve),
    0,
)

reserve_step_count = floor_steps(
    reserve_margin_available_at_trigger / margin_per_step
)
reserve_quantity = reserve_step_count * quantity_step
```

`floor_steps` is one shared deterministic helper with this algorithm:

```text
raw_steps = budget / margin_per_step
nearest_integer = round(raw_steps)
step_ratio_tolerance = 8 * f64::EPSILON * max(1, abs(raw_steps))

normalized_steps = nearest_integer
    if abs(raw_steps - nearest_integer) <= step_ratio_tolerance
    else raw_steps

step_count = floor(normalized_steps), converted to u64 with range checks
```

Non-finite intermediates or a value outside `u64` range return a typed numeric
error. After conversion, the helper verifies `step_margin <= budget +
money_tolerance(budget, step_margin)`, where `step_margin = step_count *
margin_per_step`, and decrements until the postcondition is met. Tests
cover values immediately below, on and above boundaries such as a `0.1`
quantity step and a `u64` overflow case.

If the first trade has `initial_step_count == 0`, input/configuration is not
usable and the run fails with `InsufficientInitialCapital` without a result. If
the same condition occurs only after at least one completed trade and a full
next window exists, the engine does not open another trade: it returns the
valid accumulated result with `terminal_status = capital_exhausted` and the
current post-settlement boundary as its last row. Remaining input rows are
reported as ignored. If no full next window exists, normal incomplete-tail
rules apply and the status remains `completed`.

A zero reserve step count is allowed. Every triggered attempt is logged as
full, reduced or rejected. If losses before the IV trigger consumed part of the
free collateral, the reserve sale is reduced to the number of steps that still
fits available margin. It is never allowed to create a margin breach at the
instant of submission, except for the documented floating-point comparison
tolerance.

For each sale:

- received call and put premiums increase cash;
- the marked option liability increases by the same amount at the sale instant;
- equity therefore does not jump solely because a position was opened;
- locked margin increases by `quantity * margin_per_straddle_usd`.

At every minute:

```text
equity = cash - total_short_option_liability
pnl = equity - initial_capital_usd
available_margin = equity - locked_margin
margin_breach = available_margin < -money_tolerance
```

The comparison tolerance is fixed as:

```text
money_tolerance(a, b) =
    8 * f64::EPSILON * max(1, abs(a), abs(b))
```

It is used only for floating-point classification and invariant assertions; it
does not repair balances or change quantities. The engine still reports the
unmodified `available_margin` value.

At expiry, payoff is paid from cash, the expired liability and its locked
margin are released, and the accounting invariant must still hold.

### 9.1 Drawdown

The running peak starts at the positive `initial_capital_usd` before the first
row. For every emitted post-event row `t`:

```text
running_peak_t = max(running_peak_(t-1), equity_t)
drawdown_usd_t = max(running_peak_t - equity_t, 0)
drawdown_pct_t = drawdown_usd_t / running_peak_t
```

`maximum_drawdown_usd` and `maximum_drawdown_pct` are the independent maxima of
their respective row series. Percentage drawdown is not capped at 100%; it can
exceed 100% when equity is negative. The denominator remains positive because
the running peak begins at positive initial capital.

`total_pnl = final_equity - initial_capital_usd` and
`return_pct = total_pnl / initial_capital_usd`; return is not clamped when
equity is negative.

This is intentionally not a Deribit, Bybit, Binance or OKX margin formula. It
does not model IM/MM changes, liquidation or coin-margined conversion.

## 10. Result contract

Rust returns typed columnar buffers suitable for zero-copy or one bulk-copy
conversion at the Python boundary. It does not build a Python object per row.
All tables use canonical scenario order `baseline`, `stress_2x`, `stress_3x`
and ascending timestamp/order within a scenario.

All columns are non-nullable unless a schema explicitly marks them nullable;
in the MVP only `active_trade_id` and `active_iv` in the equity series are
nullable. Trade rows are ordered by `trade_id`, tranche rows by `tranche_id`,
and reserve attempts by `attempt_timestamp_ns` within each scenario.

Logical `string` fields are closed enums or metadata strings. The Rust boundary
may transport enums as `uint8` codes, but Python must expose the documented
string values. Nullable numeric columns cross the boundary as a value buffer
plus validity mask; NaN and negative-ID sentinels are not null representations.

### 10.1 Equity series

One post-event row exists for every processed timestamp and scenario:

| Column | Logical dtype | Nullable | Meaning |
|---|---|---:|---|
| `timestamp_ns` | `int64` | no | UTC Unix nanoseconds |
| `scenario_id` | `string` | no | fixed scenario identifier |
| `spot` | `float64` | no | current underlying close |
| `active_trade_id` | `uint64` | yes | null after final settlement without re-entry |
| `active_iv` | `float64` | yes | null when no trade is active |
| `cash` | `float64` | no | model USD cash balance |
| `option_liability` | `float64` | no | non-negative marked short-option liability |
| `locked_margin` | `float64` | no | configured locked margin |
| `available_margin` | `float64` | no | `equity - locked_margin` |
| `equity` | `float64` | no | `cash - option_liability` |
| `pnl` | `float64` | no | `equity - initial_capital_usd` |
| `running_peak` | `float64` | no | drawdown running peak |
| `drawdown_usd` | `float64` | no | non-negative absolute drawdown |
| `drawdown_pct` | `float64` | no | drawdown divided by positive running peak |
| `margin_breached` | `bool` | no | available margin is below zero beyond tolerance |

Python exposes ordinary `int64`/`float64`/`bool` for non-null columns and pandas
nullable `UInt64`/`Float64` for the two nullable columns.

### 10.2 Completed trades

One non-null row exists per completed trade:

```text
scenario_id:                    string
trade_id:                       uint64
entry_timestamp_ns:             int64
expiry_timestamp_ns:            int64
strike:                         float64
entry_equity:                   float64
initial_quantity_steps:         uint64
initial_quantity:               float64
reserve_attempted:              bool
reserve_executed_quantity_steps:uint64
reserve_executed_quantity:      float64
total_received_premium:         float64
settlement_spot:                float64
total_expiry_payoff:            float64
realized_pnl:                   float64
margin_breached_during_trade:   bool
```

Rows are ordered by `scenario_id`, then `trade_id`. A trade is not created for
a rejected next initial entry after capital exhaustion.

### 10.3 Executed tranches

One row exists for each positive-quantity initial or reserve execution. A
rejected zero-quantity reserve attempt does not create a fake tranche:

```text
scenario_id:             string
trade_id:                uint64
tranche_id:              uint64
tranche_kind:            string enum {initial, reserve}
execution_timestamp_ns:  int64
expiry_timestamp_ns:     int64
strike:                  float64
quantity_steps:          uint64
quantity:                float64
active_iv:               float64
call_premium_per_unit:   float64
put_premium_per_unit:    float64
total_premium_per_unit:  float64
received_premium:        float64
locked_margin:           float64
```

### 10.4 Reserve attempts

Every reserve trigger creates exactly one row, including a full rejection:

```text
scenario_id:              string
trade_id:                 uint64
attempt_timestamp_ns:     int64
requested_quantity_steps: uint64
executed_quantity_steps:  uint64
requested_quantity:       float64
executed_quantity:        float64
available_margin_before:  float64
reserve_budget_remaining: float64
outcome:                  string enum {full, reduced, rejected}
reason:                   string enum {
    none,
    limited_by_available_margin,
    below_quantity_step,
    no_available_margin
}
```

`requested_quantity_steps` is calculated from the original reserve budget;
`executed_quantity_steps` is capped by current free margin. `outcome=full`
requires equality of the two counts.

### 10.5 Scenario summary and run metadata

One summary row exists per requested scenario:

```text
scenario_id:                     string
terminal_status:                 string enum {completed, capital_exhausted}
terminal_timestamp_ns:           int64
initial_equity:                  float64
final_equity:                    float64
total_pnl:                       float64
return_pct:                      float64
maximum_drawdown_usd:            float64
maximum_drawdown_pct:            float64
minimum_equity:                  float64
minimum_available_margin:        float64
maximum_locked_margin:           float64
completed_trade_count:           uint64
reserve_attempt_count:           uint64
full_reserve_count:              uint64
reduced_reserve_count:           uint64
rejected_reserve_count:          uint64
skipped_incomplete_window_count: uint8
input_row_count:                 uint64
processed_row_count:             uint64
ignored_input_row_count:         uint64
any_margin_breach:               bool
```

Run metadata contains the complete `DatasetMetadata`, all effective config and
scenario values, pricing-model identifier, seconds-per-year constant and
software version/commit when available. All table column names, order, logical
dtypes and nullability are public-contract tests.

## 11. Python API target

The exact class names may change during implementation, but the public surface
must remain one orchestration call per scenario or scenario set:

```python
from back_tester import (
    BacktestConfig,
    IvScenario,
    load_okx_minutes,
    run,
)

minutes = load_okx_minutes("okx_btcusdt_1m.parquet")

result = run(
    timestamps_ns=minutes.timestamps_ns,
    close=minutes.close,
    dataset=minutes.metadata,
    config=BacktestConfig(
        initial_capital_usd=1_000.0,
        base_iv=0.55,
        risk_free_rate=0.0,
        carry_rate=0.0,
        margin_per_straddle_usd=100.0,
        quantity_step=0.1,
    ),
    scenarios=[
        IvScenario.baseline(),
        IvScenario.stress_2x(after_minutes=720),
        IvScenario.stress_3x(after_minutes=720),
    ],
)

equity = result.equity_df
trades = result.trades_df
tranches = result.tranches_df
reserve_attempts = result.reserve_attempts_df
summary = result.summary_df
```

## 12. Error and determinism rules

- Invalid configuration or market data, insufficient history and inability to
  size the first trade fail before producing a result.
- Inability to size a later trade is not an error: it returns the accumulated
  valid result with `terminal_status=capital_exhausted`.
- No NaN, infinity, non-positive price or negative liability is silently
  accepted.
- Input is never implicitly sorted, deduplicated, forward-filled or repaired.
- Same-timestamp ordering is settlement/release, optional new entry, then one
  post-event equity row as defined in section 8.1.
- Reserve execution occurs after the minute's IV shock becomes active and at
  that minute's Black–Scholes mark.
- Repeated runs with identical binary, inputs and configuration produce
  bit-identical integer fields and equal floating-point arrays.
- All user-facing errors identify the invalid field or timestamp.

## 13. Explicitly out of scope

- historical option quotes, trades, order books or IV surface;
- strike smile/skew and term structure;
- stochastic or inferred IV;
- Greeks as a public subsystem;
- bid/ask spread, slippage, fees and market impact;
- exchange-specific margin, liquidation and portfolio margin;
- coin-margined accounting and collateral conversion;
- overlapping or rolling positions;
- real exchange schedules and strike grids;
- direct exchange connectors, credentials and live orders;
- a generic strategy plugin framework;
- per-minute Python strategy callbacks;
- porting the complete C++ Homework 4 matching engine.

## 14. Implementation sequence

### Phase 1 — scaffold and pricing

- create the Cargo workspace and two crates;
- configure maturin/PyO3 packaging;
- define typed errors and core configuration;
- implement normal CDF, Black–Scholes and expiry payoff;
- add pricing unit tests;
- configure Rust/Python coverage commands and the threshold checker.

### Phase 2 — deterministic scenario engine

- validate minute arrays and configuration;
- implement non-overlapping 24-hour windows;
- implement baseline and deterministic IV jump;
- implement initial and reserve tranches;
- add lifecycle and causality tests.

### Phase 3 — accounting and metrics

- implement cash, liability and locked-margin ledger;
- implement expiry settlement;
- implement equity, PnL and drawdown;
- add accounting-invariant and known-path tests.

### Phase 4 — Python boundary and data loader

- expose configuration and `run()` through PyO3;
- accept contiguous NumPy arrays without per-row Python calls;
- return bulk columnar results;
- implement and test the generic array/result boundary on synthetic fixtures;
- implement the final OKX CSV/Parquet mapping only after receiving a
  representative source file;
- add Python and end-to-end tests.

### Phase 5 — real-data validation

- add the representative OKX file schema as a documented fixture;
- run data-quality checks before a backtest;
- record base IV and margin assumptions with every result;
- run baseline, 2x and 3x scenarios;
- review PnL, drawdown and margin-breach evidence.

### 14.1 Canonical epic briefs

The implementation sequence is split into immutable briefs indexed in
[`../epics/README.md`](../epics/README.md):

1. `EPIC-001` — workspace scaffold, pricing and coverage tooling;
2. `EPIC-002` — deterministic lifecycle, accounting and result buffers;
3. `EPIC-003` — Python boundary, reporting, OKX loader and real-data handoff.

`EPIC-001` and `EPIC-002` require only synthetic fixtures. The generic boundary
and reporting part of `EPIC-003` can also use synthetic fixtures, but its final
OKX mapping and real-data acceptance criteria remain blocked until a
representative file is supplied.

### 14.2 Coverage tooling decision

The scaffold epic must configure:

- [`cargo-llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov/blob/main/README.md)
  for Rust line coverage on stable Rust;
- a separately pinned nightly Rust job using `cargo-llvm-cov --branch`, because
  upstream still marks Rust branch coverage as unstable;
- [`pytest-cov`](https://pytest-cov.readthedocs.io/en/stable/config.html) with
  branch measurement for Python;
- `scripts/check_coverage.py` to parse JSON reports and enforce distinct line
  and branch thresholds without inventing results.

Target commands after scaffold:

```bash
cargo llvm-cov --workspace --all-features \
  --fail-under-lines 90 \
  --json --output-path target/coverage/rust-lines.json

cargo +<pinned-nightly> llvm-cov --workspace --all-features \
  --branch --json --output-path target/coverage/rust-branches.json

python -m pytest \
  --cov=back_tester --cov-branch \
  --cov-report=term-missing \
  --cov-report=json:target/coverage/python.json

python scripts/check_coverage.py \
  --rust-lines target/coverage/rust-lines.json --min-rust-lines 90 \
  --rust-branches target/coverage/rust-branches.json --min-rust-branches 85 \
  --python target/coverage/python.json \
  --min-python-lines 85 --min-python-branches 80
```

The exact `cargo-llvm-cov`, `pytest-cov` and nightly versions are pinned by
`EPIC-001` after a successful local/CI run. Generated bindings may be excluded
only by a reviewed path rule; production core and Python orchestration code may
not be excluded.

## 15. Acceptance criteria

### AC-01: Rust workspace and Python package

- `cargo build --workspace` succeeds.
- `maturin develop` installs an importable Python package in the project
  environment.
- `backtest-core` has no Python or exchange-SDK dependency.
- The pinned coverage commands from section 14.2 generate machine-readable
  reports and enforce the documented thresholds.

### AC-02: Black–Scholes correctness

- Call and put match at least three independent reference cases within a
  documented tolerance.
- Put-call parity holds within the same tolerance.
- At least one reference case uses non-zero `risk_free_rate` and `carry_rate`;
  zero is a default scenario value, not hard-coded pricing behavior.
- `T = 0` returns exact intrinsic payoff.
- Invalid `S`, `K`, `T` or IV returns a typed error, not NaN.
- A full repricing is used for all IV stresses.

### AC-03: Minute-data validation

- Valid one-minute UTC arrays are accepted.
- Fewer than 1,441 points return `InsufficientHistory` with no partial result.
- Different lengths, empty input, duplicate/out-of-order timestamps, gaps,
  NaN/infinite prices and non-positive prices are rejected.
- Rejected data does not produce a partial result.
- `DatasetMetadata` rejects empty identity fields, interval other than 60 or
  timezone other than UTC.

### AC-04: 24-hour lifecycle

- Entry uses only the current minute's close.
- Strike equals entry spot and does not change later.
- Time to expiry decreases monotonically to zero.
- Settlement at 24 hours equals call plus put intrinsic payoff.
- On a shared boundary, settlement/release precedes the next entry and exactly
  one post-event row is emitted.
- A 2,881-point fixture completes exactly two trades.
- An incomplete trailing window emits no flat tail rows and is counted once;
  exactly 1,441 points have no skipped tail.

### AC-05: IV scenarios and causality

- Baseline keeps IV constant and never triggers the 1.5x reserve rule.
- A 2x or 3x scenario enters at base IV, jumps only at the configured minute
  and keeps the stressed IV through expiry.
- The shock schedule restarts for every new 24-hour trade; a new entry on a
  shared boundary uses base IV.
- No pre-shock equity point uses post-shock IV.
- Empty, duplicate, custom-multiplier and invalid shock-time scenario inputs
  fail validation; output scenario order is canonical.

### AC-06: 70/30 strategy

- Initial and reserve quantities use integer step counts and pass immediately
  below/on/above boundary tests for `quantity_step=0.1`.
- Step rounding follows the exact section 9 algorithm; non-finite products and
  `u64` overflow return typed errors.
- The reserve sale happens at most once per 24-hour trade.
- It uses the original strike and expiry.
- The trigger is inclusive at exactly `1.5 * entry_iv`.
- Reserve quantity is capped by actual available margin at the trigger; a
  reduced or rejected reserve order is explicit in results.
- Each new 24-hour trade sizes its 70/30 budgets from that trade's entry equity.
- Zero initial steps on the first trade return `InsufficientInitialCapital`;
  after a completed trade they return a valid `capital_exhausted` result.

### AC-07: Accounting and margin

- `equity = cash - liability` holds at every result point within tolerance.
- Opening a fairly marked tranche does not create instant PnL.
- Locked margin increases on sale and is released at expiry.
- `available_margin = equity - locked_margin` is reported every minute.
- A negative available margin sets `margin_breached` without silently
  liquidating the position.

### AC-08: PnL and drawdown

- Expiry PnL equals received premiums minus intrinsic payoff.
- Row and maximum drawdown follow the exact running-peak formulas in section
  9.1 and match a hand-calculated fixture.
- A negative-equity fixture can report drawdown above 100% without clamping.
- Summary values reconcile with the first and last equity points and trade log.

### AC-09: Native/Python boundary

- One Python call runs a complete scenario; no per-minute Python callback is
  used.
- Returned arrays have stable documented dtypes and equal column lengths.
- Nulls use validity masks/native nullable columns, not NaN or negative-ID
  sentinels.
- Python exposes equity, trades, executed tranches, reserve attempts and summary
  as pandas objects with the documented column order and dtypes.
- Every reserve trigger has an attempt row, including a full rejection, while
  only positive-quantity executions create tranche rows.
- Native errors surface as actionable Python exceptions.

### AC-10: Determinism and tests

- Two identical runs return equal result arrays.
- A synthetic two-day fixture completes two trades and reconciles all summary
  counts.
- `cargo fmt --all -- --check`, Clippy with all targets/features and
  warnings-as-errors, workspace tests and Python tests all pass.
- Rust/Python line and branch reports satisfy section 14.2; if the pinned branch
  job cannot run, acceptance is blocked rather than silently waived.
- A benchmark records throughput for at least one million minute points; no
  hard machine-specific performance threshold is imposed in the MVP.

### AC-11: Honest reporting

- Every result records the complete `DatasetMetadata`, base IV, effective `r/q`,
  scenario/multiplier/shock time, quantity unit/step and margin per straddle.
- Reports are labeled `synthetic Black–Scholes scenario backtest`.
- No result claims historical option execution or exchange margin fidelity.

## 16. Implementation-agent checklist

### Before coding

- [ ] Read this document completely.
- [ ] Read `docs/research/btc_24h_black_scholes_fit_gap.md`.
- [ ] Read the current immutable brief under `docs/epics/`.
- [ ] Inspect the current working tree and preserve unrelated user changes.
- [ ] Record the starting commit, or explicitly record that the repository has
      no initial commit.
- [ ] For the final loader/real-data work in Phase 4/5, confirm the actual OKX
      input schema from a representative file; do not guess column names, units
      or timezone. This is not a blocker for Phases 1–3 on synthetic fixtures.
- [ ] Convert each acceptance criterion into one or more named tests.
- [ ] Keep every phase inside the frozen MVP scope.

### While implementing

- [ ] Implement behavior test-first in small vertical slices.
- [ ] Keep `backtest-core` independent from Python and pandas.
- [ ] Validate both sides of the PyO3 boundary.
- [ ] Use typed Rust errors; never panic on user data.
- [ ] Do not sort, repair or forward-fill invalid minute data silently.
- [ ] Derive `T` from timestamps, not row count.
- [ ] Apply the documented same-timestamp settlement/entry order and emit only
      post-event rows.
- [ ] Use the current minute only when opening or triggering the reserve.
- [ ] Reprice both legs with full Black–Scholes after an IV shock.
- [ ] Keep the original strike and expiry for the reserve tranche.
- [ ] Check accounting invariants at every state transition in tests.
- [ ] Store quantity as integer step counts and test decimal-looking boundaries.
- [ ] Preserve complete dataset/config metadata in every scenario result.
- [ ] Keep result generation columnar; avoid a Python object per minute.
- [ ] Do not add exchange clients, async services, databases or plugin systems.
- [ ] Do not present synthetic marks as observed market data.

### Verification and handoff

- [ ] Run `cargo fmt --all -- --check`.
- [ ] Run `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- [ ] Run `cargo test --workspace --all-features`.
- [ ] Build/install with maturin in a clean project environment.
- [ ] Run the complete Python test suite.
- [ ] Generate Rust/Python line and branch reports and run the threshold checker
      from section 14.2.
- [ ] Run the deterministic repeat test.
- [ ] Run the million-minute benchmark and record environment plus result.
- [ ] Run baseline, 2x and 3x on the approved sample data.
- [ ] Reconcile final equity, trade PnL, locked margin and summary counts.
- [ ] Update this document if any public contract changes.
- [ ] Report files changed, commands and exact results, unsupported cases and
      remaining assumptions.
- [ ] Do not claim live readiness or historical option validation.

## 17. Definition of done

The full MVP is done only when all acceptance criteria pass, the OKX input
mapping is documented from a real sample, results are reproducible, and
baseline/2x/3x runs produce auditable PnL, equity, drawdown and margin outputs.
Absence of that sample does not block acceptance of the earlier scaffold,
pricing, lifecycle and accounting epics on synthetic fixtures.

The next decision is then evidence-based:

- reject or revise the hypothesis;
- run additional underlying periods and parameter robustness checks; or
- invest in historical option quotes, IV and exchange-specific margin data.

Live trading remains a separate later milestone.
