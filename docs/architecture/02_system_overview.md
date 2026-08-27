# MVP architecture overview

This compact pack helps readers navigate the system. Normative behavior is
defined by the [canonical architecture](01_btc_24h_rust_python_mvp.md), the
[minute-data contract](../data/okx_btcusdt_swap_1m.md), and the
[epic briefs](../epics/README.md).

## System context

```mermaid
flowchart LR
    Researcher[Researcher] -->|config + approved minute file| Backtester[MVP backtester]
    OKX[Official OKX REST v5 history candles] -->|prepared ascending CSV| Backtester
    Backtester -->|typed tables + metadata| Researcher
    Backtester -->|synthetic scenario results| Reports[Local reports under target/]
```

The only approved market input is the BTC-USDT-SWAP perpetual minute close.
Options, IV shocks, and margin are synthetic model inputs, not replayed exchange
observations.

## Containers and components

```mermaid
flowchart TB
    subgraph Python[Python orchestration]
        Loader[Strict CSV/Parquet loader]
        Config[Typed config and metadata]
        Boundary[One bulk PyO3 call]
        Tables[Typed pandas result tables]
        Loader --> Config --> Boundary --> Tables
    end

    subgraph Rust[Rust workspace]
        Native[back-tester native adapter]
        Core[backtest-core]
        Pricing[Black-Scholes + expiry payoff]
        Engine[Lifecycle + accounting]
        Results[Columnar result buffers]
        Native --> Core
        Core --> Pricing
        Core --> Engine --> Results
    end

    Boundary --> Native
    Results --> Native
```

Rust is the single source of truth for pricing, validation, lifecycle,
accounting, and results. Python validates bulk I/O and renders the returned
buffers; it does not reproduce the financial state machine.

## Data and run flow

```mermaid
flowchart LR
    File[Approved 9-column OKX file] --> Snapshot[Immutable byte snapshot]
    Snapshot --> Hash[SHA-256 / dataset_id]
    Snapshot --> Parse[Strict dtype + schema parse]
    Parse --> Validate[UTC cadence, confirm=1, finite positive close, >=1441]
    Validate --> Arrays[contiguous int64 ns + float64 close]
    Hash --> Metadata[DatasetMetadata]
    Metadata --> Call[Single native run]
    Arrays --> Call
    Call --> Buffers[Typed buffers + validity masks]
    Buffers --> Frames[Canonical pandas tables]
    Frames --> Reconcile[Summary/table reconciliation]
```

Hashing and parsing use the same snapshot. Runtime code never sorts, fills, or
repairs input. Exact fields and rejection rules live in the
[data contract](../data/okx_btcusdt_swap_1m.md).

## Exact 24-hour lifecycle

```mermaid
sequenceDiagram
    participant M as Minute timestamp
    participant T as Active trade
    participant A as Accounting
    participant R as Result buffers

    M->>T: Observe current close and elapsed time
    alt expiry boundary (exactly 24h)
        T->>A: Settle old trade payoff
        A->>A: Realize PnL and release old margin
        A->>T: Attribute post-payoff breach to old trade
        T->>R: Finalize old completed-trade row
        T->>T: Check for a complete next 24h window
        opt complete window exists
            A->>T: Attempt next entry at current close
        end
        T->>R: Emit one post-event boundary equity row
    else active trade before expiry
        T->>A: Reprice liabilities at current scenario IV
        opt inclusive reserve trigger and not attempted before
            A->>T: Attempt reserve within available 30% margin budget
        end
        T->>R: Emit minute equity row
    end
```

## Trade lifecycle state machine

```mermaid
stateDiagram-v2
    [*] --> Ready
    Ready --> Active: complete 24h window / entry funded
    Ready --> Exhausted: complete later window / entry cannot be funded
    Ready --> IncompleteTail: later rows / no complete 24h window
    Ready --> Completed: no later rows
    Active --> Active: minute reprice / optional one-time reserve attempt
    Active --> Settling: timestamp = entry + 24h
    Settling --> Ready: payoff, release, finalize old trade
    Exhausted --> [*]
    IncompleteTail --> [*]
    Completed --> [*]
```

`Ready` means there is no active trade. Before every entry the engine first
checks that the input contains a complete 24-hour window. `Active` covers minute
repricing and the optional one-time reserve attempt. `Settling` is the exact
expiry event: payoff is realized, margin is released, and the completed-trade
row is finalized. The return to `Ready` between settlement and a possible
same-timestamp entry is a transient internal state; the engine emits only one
post-event equity row for that timestamp.

`Exhausted` maps to a valid result with
`terminal_status=capital_exhausted` when a later complete window exists but its
entry cannot be funded. Insufficient capital for the very first entry is instead
a typed `InsufficientInitialCapital` error with no result. `IncompleteTail` is a
terminal reason, not an active-trade state: strictly later rows that cannot form
a complete window are excluded while `Ready`, the result keeps
`terminal_status=completed`, and `skipped_incomplete_window_count=1` records the
reason. With no later rows, `Completed` has the same terminal status and no
skipped tail.

At a shared daily boundary the immutable event order is settlement and margin
release, complete-window check, optional next entry, then exactly one post-event
row. Full details, including reserve sizing, drawdown, and table schemas, remain in the
[canonical architecture](01_btc_24h_rust_python_mvp.md).
