# Strategy research readiness roadmap

- Status: active planning document
- Date: 2026-08-27
- Starting point: accepted synthetic MVP at commit `093a9db`

## Purpose

The current project correctly answers a narrow question: how a synthetic
24-hour BTC ATM short straddle behaves on a minute underlying path under fixed
Black–Scholes IV scenarios and simplified model-USD margin.

It does not yet answer whether the strategy had a tradable historical edge,
would have survived exchange risk controls, or should be traded with real
capital. This document defines the smallest evidence-driven path from the
accepted MVP to meaningful strategy testing. It is a roadmap, not a change to
the frozen MVP contract.

## Current readiness

| Capability | Current status | Meaning |
|---|---|---|
| Pricing and 24-hour lifecycle | PASS | Deterministic synthetic Black–Scholes implementation |
| Fixed IV scenarios `1x/2x/3x` | PASS | Causal mark-to-model stress testing |
| Strict OKX minute-data boundary | PASS | Approved 4,321-row integration sample |
| Strategy sensitivity research | LIMITED | Useful for model-path and solvency sensitivity only |
| Statistical profitability evidence | NOT READY | Only three non-overlapping trades |
| Exchange-realistic economic backtest | NOT READY | IV, margin, execution and contract rules are synthetic |
| Historical option replay | NOT IMPLEMENTED | Deliberately deferred from the MVP |
| Paper/live trading | OUT OF SCOPE | Requires a separate acceptance phase |

The current technical result must therefore be described as a **synthetic
scenario backtest**, not a historical option backtest.

Local evidence is the
[`final MVP validation report`](../reports/final_mvp_validation_performance.md)
and the approved
[`OKX minute-data contract`](../data/okx_btcusdt_swap_1m.md). They establish
technical readiness and sample provenance, not profitability.

## Evidence from the accepted sample

The three-day OKX run uses `initial_capital_usd=1,000`, `base_iv=0.55`,
`margin_per_straddle_usd=100` and daily compounding of position size.

| Trade | Entry equity | Initial quantity | Realized PnL |
|---:|---:|---:|---:|
| 1 | 1,000.00 | 7.0 BTC straddles | 3,959.73 |
| 2 | 4,959.73 | 34.7 BTC straddles | 46,795.77 |
| 3 | 51,755.50 | 362.2 BTC straddles | 484,665.08 |

This compounding, combined with the illustrative margin assumption and absence
of liquidation, explains the economically implausible final PnL. The maximum
drawdown ratios are approximately 397% for baseline, 445% for `2x`, and 457%
for `3x`; a ratio above 100% is mathematically valid when equity becomes
negative, but represents insolvency for economic interpretation.

All stress reserve attempts were rejected. All scenarios open at the same base
IV, remain open until expiry, and settle to IV-independent intrinsic payoff.
The resulting identical terminal PnL is expected. The stress scenarios measure
intratrade equity and solvency sensitivity here, not an executable IV edge.

## P0 blockers for meaningful strategy research

All P0 items are required before a result may be called evidence for or against
the economic hypothesis.

### 1. Long, representative underlying history

Acquire at least 6–12 months and preferably several years of one-minute data,
covering bull, bear, sideways, high-volatility and low-volatility regimes. The
minimum period is only a screening floor: the acceptance target must also state
the number of complete non-overlapping trades and regime coverage.

Required implementation and evidence:

- immutable dataset manifest with source query, symbol, timezone, coverage,
  files, row counts and SHA-256 hashes;
- schema, cadence, duplicate, ordering, invalid-price and completion checks;
- deterministic partitioning into contiguous segments of at least 1,441
  boundary points when real history contains gaps;
- no forward-fill, interpolation or hidden repair;
- explicit accounting of excluded gaps, incomplete tails and unavailable days;
- tests proving that no trade crosses a missing-data boundary.

The raw Tardis trades sample is not a minute-candle dataset and cannot enter the
current loader directly. Any trade-to-candle aggregation requires a separate,
source-backed contract; empty minutes must not be silently fabricated.

#### Manifest-level segment and accounting contract

The broad-run manifest must declare one `grid_anchor_utc` before any comparison.
Candidate entries are `grid_anchor_utc + n * 24h`; the anchor never restarts at
a segment boundary. A candidate is accepted only when all 1,441 exact minute
boundary points from entry through expiry lie wholly inside one validated
contiguous segment. A gap or incomplete window rejects that candidate without
sorting, filling or repair. Its interval is no-position time.

Adjacent accepted 1,441-point inclusive windows share exactly one boundary
timestamp/source point. That point is processed once with carried portfolio
state in the canonical order: settlement/payoff, realized PnL and margin
release, next entry attempt, then exactly one post-event equity row. The
primary result forbids duplicate boundary/source rows, separate pre-event and
post-event rows for the same timestamp, and independent window resets.

Segments are processed in UTC chronological order as one primary portfolio:

- initial capital is supplied once; cash, equity, running peak and drawdown
  carry across segment boundaries;
- after a completed trade and throughout a gap/no-position interval, option
  liability and locked margin are zero, equity equals carried cash, and no
  synthetic minute rows are emitted for unavailable input;
- the accepted windows are aggregated chronologically and position sizing
  compounds from carried equity;
- every compared scenario or strategy variant receives the identical
  manifest-derived accepted-window set; a failed/insolvent variant remains a
  failed result rather than silently dropping adverse windows;
- primary drawdown duration is elapsed UTC time from the running-peak timestamp
  until recovery, or until the manifest end when unrecovered. It includes
  gaps/no-position periods while equity remains below the peak; exposure-only
  duration, if reported, is a separate metric.

Independent per-segment runs may be reported only as clearly labelled
sensitivity runs. They reset capital and running peak and must not be summed,
compounded or substituted for the primary manifest-level result.

### 2. Target product and pricing convention

Select the venue and exact option family before calibrating IV or economics.
Freeze:

- underlying index or forward used by the venue;
- inverse versus linear payoff and settlement/collateral currency;
- contract multiplier, quantity step, strike grid and listed expiries;
- premium quote convention, interest, carry, funding and basis treatment;
- settlement-price window and expiry event rules.

Venue/product, settlement/collateral, multiplier, expiry and quote conventions
are blocking decisions for IV alignment, sizing, margin, costs and execution.
Dependent economic implementation must not begin until they are dated,
versioned and supported by official venue specifications.

The current BTC-USDT-SWAP close is a perpetual proxy, and the current model uses
linear model-USD accounting. It must not be described as a literal Deribit,
Bybit or OKX contract. Product specifications must be refreshed from official
venue documentation at implementation time.

### 3. Explicit IV methodology

`base_iv` is currently a manual scenario parameter. Before testing claims about
actual IV behavior, choose and document one of two modes:

1. **Scenario mode** — run a pre-registered IV grid and label every result as
   synthetic sensitivity analysis.
2. **Historical-IV mode** — use a time-causal IV index or option-chain-derived
   ATM IV observable at each entry, with exact source, timestamp alignment,
   expiry interpolation and missing-data rules.

Historical-IV mode must prevent look-ahead and record the IV source identity in
result metadata. A single current mean IV applied to old prices is not
historical evidence.

Scenario IV can support only R1 screening/falsification under declared
assumptions; it cannot confirm a historical IV edge. R2 requires a causal
historical IV series from a time-aligned IV index or option-chain-derived
method. The target product decision above governs the source, expiry alignment
and quote convention.

### 4. Margin, solvency and liquidation

Replace the constant `margin_per_straddle_usd` assumption with a selected,
versioned risk model that includes at least:

- initial and maintenance margin;
- mark-dependent short-option requirements;
- collateral and settlement currency;
- defined offsets between call and put legs;
- liquidation or forced-deleveraging event order and fees;
- explicit behavior when available margin or equity crosses its threshold.

Until liquidation exists, any run with `any_margin_breach=true` or negative
equity must be classified as economically invalid, even if expiry PnL is
positive.

### 5. Position sizing and risk limits

The current 70/30 rule is a margin-budget rule and may compound exposure very
aggressively. Add a documented risk policy with deterministic limits on:

- maximum quantity and notional per trade;
- maximum leverage or margin utilization;
- maximum loss or stress loss per trade;
- compounding policy and optional fixed-capital comparison;
- action after a breach, drawdown limit or capital-exhaustion event.

Tests must cover boundary rounding, binding limits and the exact priority when
multiple limits apply.

### 6. Execution costs and fill model

Add, at minimum, configurable fees, bid/ask spread and deterministic slippage.
For stronger evidence, constrain quantity by observable liquidity and define a
partial-fill policy. Premium received must represent the selected executable
price, not the synthetic mid mark.

Synthetic execution must remain visibly distinct from historical option fills.

### 7. Pre-registered success and failure rules

Define thresholds before running the broad dataset. At minimum include:

- zero unresolved margin breaches and zero negative-equity paths;
- maximum acceptable drawdown and stress loss;
- minimum number of completed trades and regime coverage;
- net return after costs, hit rate, profit factor and tail loss;
- Sharpe/Sortino only with an explicit return interval and annualization rule;
- an untouched out-of-sample period;
- failure conditions that stop promotion even when terminal PnL is positive.

Thresholds are a user/research decision and must not be chosen after viewing
results merely to make the strategy pass.

## P1 research-quality improvements

These items are required for repeatable comparison of strategy variants after
the P0 model is credible:

- batch runner for a declared parameter grid without per-minute Python calls;
- immutable run manifest containing dataset hashes, complete config, model ID,
  software commit and platform details;
- daily/trade return series and reconciled metrics including drawdown duration,
  exposure, turnover, expected shortfall and loss distribution;
- train/calibration, validation and untouched out-of-sample partitions;
- walk-forward evaluation where parameters are recalibrated causally;
- robustness checks across IV, costs, margin, sizing and execution assumptions;
- comparison against simple baselines such as no trade, fixed quantity and
  non-compounding sizing;
- machine-readable rejection reasons for invalid economic runs;
- one command that reproduces tables and a decision-oriented report.

If resampling, bootstrap or another randomized method is introduced, its seed
must be part of both config and result metadata.

## Supporting more than one strategy

The current engine intentionally implements one frozen short-straddle state
machine. Before calling the project a general strategy backtester:

- specify the second concrete strategy and its required events and state;
- separate reusable portfolio/risk transitions from strategy decisions only
  where both implementations demonstrate the shared boundary;
- preserve deterministic bulk execution in Rust;
- avoid a generic plugin system or inheritance hierarchy without a concrete
  acceptance need;
- give every strategy a versioned config, result label and dedicated invariant
  tests.

This work is not needed to complete the current short-straddle research path.

## Conditional historical-option phase

Historical option replay is warranted if synthetic tests survive realistic
solvency, sizing and cost constraints, or if the research question explicitly
concerns observed IV, quotes, spreads or execution.

That phase requires timestamped option chains or quotes, instrument metadata,
strike/expiry selection, bid/ask and stale-quote rules, forward/index alignment,
dated/versioned venue specification changes, and a no-look-ahead contract. Only
then may results be called a historical option replay.

### Tardis.dev acquisition decision

Tardis data is **not required** for the current synthetic scenario engine or
for extending its OKX underlying-candle history. It becomes useful when the
project proceeds to historical option-market replay or order-book-aware
execution research.

Do not bulk-purchase or download a multi-year archive before selecting the
target venue and contract family. Use the free first-day-of-month samples to
implement and test the ingestion contract, measure compressed and expanded
volume, and confirm that the fields and timestamps answer the research
question. Tardis documents free samples for Deribit and OKX Options; its stated
historical coverage begins on 2019-03-30 for Deribit and 2020-02-01 for OKX
Options. Exact channel incidents and symbol coverage must still be captured
from exchange metadata for the requested dates.

Minimum normalized datasets by purpose:

| Research purpose | Tardis datasets | Required? |
|---|---|---|
| Select listed ATM strike/expiry and obtain historical IV/Greeks | `options_chain` with grouped symbol `OPTIONS` | Yes for chain-based historical IV |
| Use executable top-of-book entry/mark/exit assumptions | `quotes` for `OPTIONS`; `book_ticker` only after the selected exchange/date is confirmed to support it | Yes for quote replay |
| Validate observed trading and liquidity | `trades` for `OPTIONS` | Recommended; trades alone are insufficient |
| Model depth, market impact or partial fills | `book_snapshot_25` or reconstructed `incremental_book_L2` | Only for depth-aware execution |
| Align forward, perpetual, index and basis | matching futures/perpetual `quotes`, `trades`, and `derivative_ticker` as available | Required if the selected pricing convention uses them |
| Reproduce another account's fills or liquidation | Not available from public market data | Impossible without private account/order state |

`options_chain` is the smallest useful option-research dataset because it
contains active instruments, strikes, expiries, bid/ask, bid/ask IV, mark IV,
Greeks, open interest and underlying information. For tradability claims it
must be paired with quotes; for fill-depth claims it must be paired with order
book data. The existing one-day Tardis BTC-USDT-SWAP raw-trades file is neither
an option dataset nor a complete minute-candle path and must not be used as a
substitute.

The phrase **exact replay** needs a narrow definition:

- Tardis can reproduce the sequence of recorded public market-data messages;
- normalized files may differ structurally from exchange-native messages;
- L2 data reconstructs visible book state but generally does not reveal queue
  priority, hidden liquidity, private orders, account collateral or the exact
  exchange risk-engine state;
- therefore the project may claim historical market replay or a documented
  fill simulation, but not guaranteed reproduction of fills for a hypothetical
  account.

When acquisition is approved, preserve daily gzip files unchanged under the
ignored `data/` tree and commit only a manifest containing URLs/request
parameters, exchange and data-type IDs, date range, symbols, sizes, hashes,
coverage incidents and licensing notes. The parser must stream or partition
daily files rather than loading a multi-year tick archive into memory at once.

Primary references:

- [Tardis downloadable CSV files](https://docs.tardis.dev/downloadable-csv-files)
- [Tardis normalized data types](https://docs.tardis.dev/downloadable-csv-files/data-types)
- [Tardis Deribit coverage](https://docs.tardis.dev/historical-data-details/deribit)
- [Tardis OKX Options coverage](https://docs.tardis.dev/historical-data-details/okex-options)
- [Tardis billing and subscriptions](https://docs.tardis.dev/faq/billing-and-subscriptions)

## Paper and live readiness

Paper/live trading remains a separate phase after an accepted economic
backtest. It requires, at minimum:

- authenticated exchange connector and instrument discovery;
- real-time market-data validation and clock synchronization;
- order state machine, idempotency, retries and reconciliation;
- pre-trade limits, kill switch and independent risk checks;
- persistent positions, cash and audit log;
- monitoring, alerts, secret management and incident procedures;
- paper-trading evidence under realistic latency and failure injection;
- a separate user decision before any real order can be sent.

Backtest acceptance must never authorize live trading automatically.

## Dependency and decision contract

| Decision or artifact | Required before | Promotion effect |
|---|---|---|
| Long-history manifest, hashes, UTC grid and segment rules | Product/IV calibration on the broad sample | Makes the comparison population immutable |
| Dated venue/product, settlement/collateral, multiplier, expiry and quote conventions | IV alignment and all sizing, margin, solvency, cost or fill economics | Blocks economic claims until frozen |
| Scenario or causal historical-IV methodology | Sizing/margin calibration and broad hypothesis run | Scenario IV caps claims at R1; causal historical IV is required for R2 |
| Cross-segment accounting contract above | Any multi-segment aggregation or comparison | Prevents resets, gap crossing and window-selection bias |
| Sizing, margin, solvency and liquidation | Fees/spread/slippage/fill layer | Establishes whether a path remains economically valid |
| Costs and deterministic fill rules | Experiment runner and out-of-sample decision | Enables net-of-cost comparison |

Unresolved items in this table are product/methodology blockers, not defaults
for an implementation agent to invent.

## Proposed implementation sequence

| Order | Proposed epic | Explicit prerequisites | Exit outcome |
|---:|---|---|---|
| 1 | EPIC-004: long-history datasets, manifest and segment/grid contract | Accepted MVP | Broad, auditable comparison population |
| 2 | EPIC-005: target product and IV methodology | EPIC-004 manifest; official dated product evidence | Frozen product conventions plus declared scenario IV or causal historical IV |
| 3 | EPIC-006: sizing, margin, solvency and liquidation | EPIC-005 product, collateral, multiplier, expiry/quote and IV decisions | Economically valid solvency path |
| 4 | EPIC-007: fees, spread, slippage and deterministic fills | EPIC-006 risk/accounting behavior | Net-of-cost executable-price results |
| 5 | EPIC-008: experiment runner, metrics and out-of-sample protocol | EPIC-004–007 plus pre-registered success/failure rules | Reproducible hypothesis decision |
| 6 | Optional historical-option replay epic | R2 evidence plus an explicit acquisition decision | Observed option-market evidence under R3 rules |
| 7 | Separate paper-trading program | Accepted economic/replay evidence and separate authorization | Operational evidence without real capital |

Each epic needs an immutable brief and the existing
`developer -> QA -> reviewer` acceptance workflow. The dependency order above
is normative: changing it requires an explicit contract decision, not an
implementation convenience.

## Readiness gates

| Gate | Required evidence | Promotion meaning |
|---|---|---|
| R0 — technical MVP | Current accepted tests, coverage and sample integration | Synthetic engine is trustworthy |
| R1 — scenario research | Long data, bounded sizing, costs, solvency handling, pre-registered metrics and scenario IV | Strategy can be screened or falsified only under declared assumptions; no historical IV edge can be confirmed |
| R2 — economic backtest | Selected product plus causal historical IV from a time-aligned index or option-chain-derived method, realistic margin/execution and out-of-sample evidence | Historical economic hypothesis can be judged, without claiming option quote replay |
| R3 — historical option replay, if needed | Time-causal option chain and bid/ask data plus explicit quote-selection, staleness and execution rules | Historical option-market behavior under the documented replay rules can be claimed |
| R4 — paper trading | Reliable connector, risk controls, reconciliation and operational tests | Paper orders can be evaluated |
| R5 — live decision | Separate risk review and explicit user authorization | Limited live trial may be considered |

Passing a gate means only that its stated question can be evaluated. It does
not imply that the strategy is profitable or that the next gate must be pursued.

## Definition of ready for the next broad backtest

The next broad strategy run may be treated as decision evidence only when:

- P0 items 1–7 have accepted contracts and tests;
- the dataset manifest and hashes are preserved with the result;
- the run contains the predeclared minimum trades and required regimes;
- costs, sizing, margin and insolvency behavior are active, not report-only;
- scenario assumptions and any historical inputs are clearly distinguished;
- calibration and out-of-sample periods are immutable before evaluation;
- all summaries reconcile to detailed trades and equity arrays;
- the report includes failed scenarios and limitations, not only favorable PnL.

The accepted MVP remains the reference implementation for deterministic
pricing and lifecycle behavior while these realism layers are added.
