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
| Rates | Risk-free and carry/dividend rates fixed at zero |
| Base volatility | Required annualized decimal `base_iv`, for example `0.55` |
| Baseline | IV remains equal to `base_iv` |
| Stress | IV jumps after entry to `2x` or `3x` and stays there until expiry |
| Initial allocation | 70% of initial capital as margin budget |
| Reserve allocation | 30% as margin budget for one additional sale |
| Reserve trigger | `current_iv >= 1.5 * entry_iv` |
| Reserve instrument | Same call and put, strike and expiry as the initial sale |
| Margin | Constant configured margin per one complete straddle |
| Execution | Synthetic Black–Scholes mark; no book, spread or slippage |
| Positions | Non-overlapping 24-hour trades |
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
docs/
  architecture/
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

- arrays have equal non-zero length;
- timestamps are strictly increasing;
- adjacent timestamps differ by exactly 60 seconds;
- prices are finite and strictly positive;
- no duplicate timestamps;
- the first complete 24-hour window contains 1,441 boundary points, including
  both entry and expiry timestamps;
- an incomplete final window is skipped and counted in result metadata.

Python may load `open`, `high`, `low` and `volume`, but the MVP passes only
`timestamp` and `close` to Rust. The final OKX column mapping cannot be frozen
until a representative source file is available.

### 6.2 Backtest configuration

```text
initial_capital_usd:       finite, > 0
dataset_id:                non-empty source identity
base_iv:                   annualized decimal, finite, > 0
stress_multiplier:         finite, >= 1
shock_after_minutes:       integer in 1..1439 for stress; absent for baseline
initial_allocation:        exactly 0.70 in MVP
reserve_allocation:        exactly 0.30 in MVP
reserve_trigger_multiple:  exactly 1.50 in MVP
margin_per_straddle_usd:   finite, > 0
quantity_step:             finite, > 0
```

The public API may represent baseline and stress as separate scenario objects,
but invalid combinations must fail before the run starts. In particular, a
stress multiplier greater than one requires `shock_after_minutes`.

## 7. Pricing contract

For `T > 0`, `S > 0`, `K > 0` and `sigma > 0`:

```text
d1 = (ln(S / K) + 0.5 * sigma^2 * T) / (sigma * sqrt(T))
d2 = d1 - sigma * sqrt(T)

call = S * N(d1) - K * N(d2)
put  = K * N(-d2) - S * N(-d1)
```

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
stress at/after shock: IV(t) = base_iv * stress_multiplier
```

Every stress run opens at `base_iv`. Starting a run at `2x` or `3x` would also
increase the premium received and would not test the loss caused by a later IV
jump.

## 9. Quantity and margin model

Capital allocations are margin budgets, not option premium or notional:

```text
entry_equity = equity at the start of the current 24-hour trade
initial_budget = entry_equity * 0.70
reserve_budget = entry_equity * 0.30

initial_quantity = floor_to_step(
    initial_budget / margin_per_straddle_usd,
    quantity_step,
)

reserve_margin_available_at_trigger = max(
    min(reserve_budget, available_margin_before_reserve),
    0,
)

reserve_quantity = floor_to_step(
    reserve_margin_available_at_trigger / margin_per_straddle_usd,
    quantity_step,
)
```

If `initial_quantity` is zero, the run must fail with a typed configuration
error. A zero reserve quantity is allowed but must be reported. If losses before
the IV trigger consumed part of the free collateral, the reserve sale is
reduced to the quantity that still fits available margin. It is never allowed
to create a margin breach at the instant of submission.

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
margin_breach = available_margin < 0
```

At expiry, payoff is paid from cash, the expired liability and its locked
margin are released, and the accounting invariant must still hold.

This is intentionally not a Deribit, Bybit, Binance or OKX margin formula. It
does not model IM/MM changes, liquidation or coin-margined conversion.

## 10. Result contract

Rust returns columnar arrays suitable for zero-copy or one bulk-copy conversion
at the Python boundary.

### Equity series

- `timestamp_ns`;
- `scenario_id`;
- `spot`;
- `active_iv`;
- `cash`;
- `option_liability`;
- `locked_margin`;
- `available_margin`;
- `equity`;
- `pnl`;
- `drawdown_usd`;
- `drawdown_pct`;
- `margin_breached`.

### Tranche/trade log

- trade and tranche IDs;
- entry, reserve-trigger and expiry timestamps;
- strike and expiry;
- quantity;
- call premium, put premium and total premium;
- entry IV and active scenario;
- realized expiry payoff and PnL;
- whether the tranche is initial or reserve;
- requested and executed reserve quantity plus a rejection/reduction reason.

### Summary

- initial and final equity;
- total PnL and return;
- maximum drawdown in USD and percent;
- minimum equity and minimum available margin;
- maximum locked margin;
- number of completed trades;
- number of reserve triggers;
- number of full, reduced and rejected reserve executions;
- number of skipped incomplete windows;
- whether any margin breach occurred;
- all effective configuration values.

## 11. Python API target

The exact class names may change during implementation, but the public surface
must remain one orchestration call per scenario or scenario set:

```python
from back_tester import BacktestConfig, IvScenario, load_okx_minutes, run

minutes = load_okx_minutes("okx_btcusdt_1m.parquet")

result = run(
    timestamps_ns=minutes.timestamps_ns,
    close=minutes.close,
    config=BacktestConfig(
        dataset_id="okx-btcusdt-swap-1m-example",
        initial_capital_usd=1_000.0,
        base_iv=0.55,
        margin_per_straddle_usd=100.0,
        quantity_step=0.1,
    ),
    scenarios=[
        IvScenario.baseline(),
        IvScenario.jump(multiplier=2.0, after_minutes=720),
        IvScenario.jump(multiplier=3.0, after_minutes=720),
    ],
)

equity = result.equity_df
trades = result.trades_df
summary = result.summary_df
```

## 12. Error and determinism rules

- Invalid configuration or market data fails before producing a partial result.
- No NaN, infinity, non-positive price or negative liability is silently
  accepted.
- Input is never implicitly sorted, deduplicated, forward-filled or repaired.
- Event ordering is determined only by input timestamp and documented
  same-timestamp rules.
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
- add pricing unit tests.

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
- implement CSV/Parquet mapping in Python;
- add Python and end-to-end tests.

### Phase 5 — real-data validation

- add the representative OKX file schema as a documented fixture;
- run data-quality checks before a backtest;
- record base IV and margin assumptions with every result;
- run baseline, 2x and 3x scenarios;
- review PnL, drawdown and margin-breach evidence.

## 15. Acceptance criteria

### AC-01: Rust workspace and Python package

- `cargo build --workspace` succeeds.
- `maturin develop` installs an importable Python package in the project
  environment.
- `backtest-core` has no Python or exchange-SDK dependency.

### AC-02: Black–Scholes correctness

- Call and put match at least three independent reference cases within a
  documented tolerance.
- Put-call parity holds within the same tolerance.
- `T = 0` returns exact intrinsic payoff.
- Invalid `S`, `K`, `T` or IV returns a typed error, not NaN.
- A full repricing is used for all IV stresses.

### AC-03: Minute-data validation

- Valid one-minute UTC arrays are accepted.
- Different lengths, empty input, duplicate/out-of-order timestamps, gaps,
  NaN/infinite prices and non-positive prices are rejected.
- Rejected data does not produce a partial result.

### AC-04: 24-hour lifecycle

- Entry uses only the current minute's close.
- Strike equals entry spot and does not change later.
- Time to expiry decreases monotonically to zero.
- Settlement at 24 hours equals call plus put intrinsic payoff.
- Incomplete trailing data is skipped and counted.

### AC-05: IV scenarios and causality

- Baseline keeps IV constant and never triggers the 1.5x reserve rule.
- A 2x or 3x scenario enters at base IV, jumps only at the configured minute
  and keeps the stressed IV through expiry.
- No pre-shock equity point uses post-shock IV.

### AC-06: 70/30 strategy

- Initial and reserve quantities follow the documented budgets and floor to
  `quantity_step`.
- The reserve sale happens at most once per 24-hour trade.
- It uses the original strike and expiry.
- The trigger is inclusive at exactly `1.5 * entry_iv`.
- Reserve quantity is capped by actual available margin at the trigger; a
  reduced or rejected reserve order is explicit in results.
- Each new 24-hour trade sizes its 70/30 budgets from that trade's entry equity.

### AC-07: Accounting and margin

- `equity = cash - liability` holds at every result point within tolerance.
- Opening a fairly marked tranche does not create instant PnL.
- Locked margin increases on sale and is released at expiry.
- `available_margin = equity - locked_margin` is reported every minute.
- A negative available margin sets `margin_breached` without silently
  liquidating the position.

### AC-08: PnL and drawdown

- Expiry PnL equals received premiums minus intrinsic payoff.
- Maximum drawdown matches a hand-calculated equity fixture.
- Summary values reconcile with the first and last equity points and trade log.

### AC-09: Native/Python boundary

- One Python call runs a complete scenario; no per-minute Python callback is
  used.
- Returned arrays have stable documented dtypes and equal column lengths.
- Python exposes equity, trades and summary as pandas objects.
- Native errors surface as actionable Python exceptions.

### AC-10: Determinism and tests

- Two identical runs return equal result arrays.
- A synthetic two-day fixture completes two trades and reconciles all summary
  counts.
- `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace` and Python tests all pass.
- A benchmark records throughput for at least one million minute points; no
  hard machine-specific performance threshold is imposed in the MVP.

### AC-11: Honest reporting

- Every result records `dataset_id`, base IV, stress multiplier, shock time and
  margin per straddle.
- Reports are labeled `synthetic Black–Scholes scenario backtest`.
- No result claims historical option execution or exchange margin fidelity.

## 16. Implementation-agent checklist

### Before coding

- [ ] Read this document completely.
- [ ] Read `docs/research/btc_24h_black_scholes_fit_gap.md`.
- [ ] Inspect the current working tree and preserve unrelated user changes.
- [ ] Record the starting commit, or explicitly record that the repository has
      no initial commit.
- [ ] Confirm the actual OKX input schema from a representative file; do not
      guess column names, units or timezone.
- [ ] Convert each acceptance criterion into one or more named tests.
- [ ] Keep every phase inside the frozen MVP scope.

### While implementing

- [ ] Implement behavior test-first in small vertical slices.
- [ ] Keep `backtest-core` independent from Python and pandas.
- [ ] Validate both sides of the PyO3 boundary.
- [ ] Use typed Rust errors; never panic on user data.
- [ ] Do not sort, repair or forward-fill invalid minute data silently.
- [ ] Derive `T` from timestamps, not row count.
- [ ] Use the current minute only when opening or triggering the reserve.
- [ ] Reprice both legs with full Black–Scholes after an IV shock.
- [ ] Keep the original strike and expiry for the reserve tranche.
- [ ] Check accounting invariants at every state transition in tests.
- [ ] Keep result generation columnar; avoid a Python object per minute.
- [ ] Do not add exchange clients, async services, databases or plugin systems.
- [ ] Do not present synthetic marks as observed market data.

### Verification and handoff

- [ ] Run `cargo fmt --check`.
- [ ] Run `cargo clippy --workspace --all-targets -- -D warnings`.
- [ ] Run `cargo test --workspace`.
- [ ] Build/install with maturin in a clean project environment.
- [ ] Run the complete Python test suite.
- [ ] Run the deterministic repeat test.
- [ ] Run the million-minute benchmark and record environment plus result.
- [ ] Run baseline, 2x and 3x on the approved sample data.
- [ ] Reconcile final equity, trade PnL, locked margin and summary counts.
- [ ] Update this document if any public contract changes.
- [ ] Report files changed, commands and exact results, unsupported cases and
      remaining assumptions.
- [ ] Do not claim live readiness or historical option validation.

## 17. Definition of done

The MVP is done only when all acceptance criteria pass, the OKX input mapping
is documented from a real sample, results are reproducible, and baseline/2x/3x
runs produce auditable PnL, equity, drawdown and margin outputs.

The next decision is then evidence-based:

- reject or revise the hypothesis;
- run additional underlying periods and parameter robustness checks; or
- invest in historical option quotes, IV and exchange-specific margin data.

Live trading remains a separate later milestone.
