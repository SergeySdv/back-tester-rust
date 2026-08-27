# Bybit and Deribit option data for a 24-hour BTC ATM backtest

Research date: 2026-08-25. Only first-party exchange documentation and first-party production endpoints were used.

> Repository attribution: references below to an existing replay runtime,
> canonical replay cache or one-instrument manifest describe the separate
> `back-tester-2026` repository inspected during the original research. The
> current `back-tester-rust` repository has no such runtime or manifest; its MVP
> contract is the synthetic scenario model under `docs/architecture/`.

## Status vocabulary

- **Verified** means the exchange explicitly documents the fact at a cited official URL.
- **Observed** means a read-only request to the production public API returned the described result on the research date. It demonstrates endpoint behaviour, not a retention or service-level promise.
- **Unknown** means no current first-party statement or endpoint establishing the fact was found. It must not be treated as available data.

## Conclusion

The project can support this strategy only after adding an option-specific data and accounting layer. The existing minute BTC perpetual history is enough to drive the underlying path, but it is not enough to reconstruct historically executable option prices, IV, spread, liquidity, margin, settlement, or Greeks.

| Requirement | Bybit | Deribit | Consequence |
|---|---|---|---|
| Historical public option trades | Recent REST window; official download page exists, but option coverage/retention is **unknown** | Official history API, trades and instruments since launch | Deribit is the stronger first-party trade backfill source |
| Historical L2 book | No first-party history endpoint found | No first-party history endpoint found | Capture prospectively, or license an external dataset |
| Historical ticker / complete IV surface / Greeks | No first-party history endpoint found | No complete historical ticker/IV/Greeks endpoint found | Capture prospectively; trade IV is sparse and not mark IV |
| Historical mark price | No contract-level mark history found | Five-minute history for only a subset of VIX-contributing options | Insufficient for an unbiased full-universe backtest |
| Historical underlying index | No dedicated option-index history found in the reviewed endpoints | Range-based index chart history available | Store the venue index prospectively on both venues |
| Settlement/delivery price | Cursor-paginated delivery-price endpoint | Paginated delivery-price and settlement endpoints | Store it as the authoritative expiry cash flow input |
| Current chain, IV and Greeks | REST/WS | REST/WS | Suitable for forward collection and live trading |

Therefore:

1. A trade-only Deribit backtest can be built from official history, but it must model quotes/spreads conservatively and cannot claim exact executable fills or historical margin without additional data.
2. A high-fidelity Bybit or Deribit backtest needs prospective WebSocket capture of chain metadata, ticker/IV/Greeks, order books, trades, index/underlying and account margin state.
3. A Black-Scholes repricing experiment driven by perpetual candles and an assumed constant/current mean IV is possible now, but it is a scenario model, not a historical option-market backtest. Label its PnL and drawdown accordingly.

## Bybit

### Instrument discovery and 24-hour ATM selection

**Verified:** [`GET /v5/market/instruments-info`](https://bybit-exchange.github.io/docs/v5/market/instrument) accepts `category=option`, `baseCoin`, `symbol`, `status`, `limit` (default 500, maximum 1000) and `cursor`. The response supplies `nextPageCursor` and, for each option, `symbol`, `status`, `baseCoin`, `quoteCoin`, `settleCoin`, `optionsType`, `launchTime`, `deliveryTime`, `deliveryFeeRate`, price/tick limits, quantity limits/step and `displayName`.

**Observed:** On 2026-08-25,

```text
GET https://api.bybit.com/v5/market/instruments-info
    ?category=option&baseCoin=BTC&limit=1000
```

returned 738 instruments and an empty `nextPageCursor`. This confirms that one page covered that particular snapshot, not that one page will always cover the chain. Collection must follow cursors until empty.

A current short-dated linear example was `BTC-27AUG26-79250-C-USDT`: BTC base, USDT quote and settlement, 0.01 quantity step/minimum and USD 5 price tick. This symbol is only an observation; never hard-code it.

For each decision timestamp, select the smallest `deliveryTime` strictly after the desired holding horizon, then minimize `abs(strike - underlying_reference)` within that expiry and retain both call and put. If “24-hour” means the exchange daily contract rather than exactly 24 elapsed hours, record the actual `time_to_expiry_ns`; listing and expiry times determine it. Define a deterministic lower-strike or higher-strike tie-breaker. “ATM” is an algorithmic selection, not a permanent symbol.

### Current tickers, IV and Greeks

**Verified current-only REST:** [`GET /v5/market/tickers`](https://bybit-exchange.github.io/docs/v5/market/tickers) with `category=option` requires `symbol` or `baseCoin` and optionally accepts `expDate`. It returns bid/ask prices and sizes, `bidIv`, `askIv`, `lastPrice`, `markPrice`, `indexPrice`, `markIv`, `underlyingPrice`, open interest, volume/turnover, and `delta`, `gamma`, `vega`, `theta`. The method has no historical timestamp or cursor parameter.

**Verified prospective WS:** [`tickers.{symbol}`](https://bybit-exchange.github.io/docs/v5/websocket/public/ticker) is snapshot-only for options and is documented at 100 ms push frequency. It must be recorded prospectively.

**Observed:** On the research date, querying `category=option&baseCoin=BTC&expDate=27AUG26` returned current USDT-settled contracts. Around a BTC underlying near USD 79,354, the 79,250 put had approximately 935/945 bid/ask and 0.455/0.4596 bid/ask IV; its `markIv` was approximately 0.4573. These are transient sanity checks, not calibration inputs.

**Historical availability:** No first-party Bybit endpoint for historical option tickers, bid/ask IV, mark IV or Greeks was found. These fields must be captured prospectively. Do not reconstruct an IV surface by calling the endpoint later.

### Trades

**Verified recent REST:** [`GET /v5/market/recent-trade`](https://bybit-exchange.github.io/docs/v5/market/recent-trade) accepts `category=option`, optional `symbol`, `baseCoin`, `optionType`, and `limit` (default 500, maximum 1000). It has no start/end-time or cursor parameter. Option records contain execution ID, symbol, price, size, taker side, time, block/RPI flags, sequence, and contemporaneous mark price (`mP`), index price (`iP`), mark IV (`mIv`) and execution IV (`iv`).

**Observed:** `category=option&baseCoin=BTC&limit=1000` returned exactly 1000 rows spanning only about 23 minutes at that moment (14:30:02–14:52:55 UTC on 2026-08-25). This is observed depth, not documented retention. The endpoint cannot paginate earlier.

**Verified prospective WS:** [`publicTrade.{baseCoin}`](https://bybit-exchange.github.io/docs/v5/websocket/public/trade), for example `publicTrade.BTC`, streams option executions and the same option-specific mark/index/IV fields. Persist `execId` and `seq` for deduplication and gap checks.

**Verified archive entry point, coverage unknown:** The recent-trade documentation links to Bybit's official [Historical Market Data](https://www.bybit.com/en/derivative-activity/history-data) download page, which advertises downloadable historical public trade data. The public documentation reviewed does not specify option coverage, earliest date, file naming, completeness, or retention. Inspect and record the page's actual option inventory before designing a Bybit backfill; do not assume that perpetual/futures CSV availability implies option availability.

### Order books

**Verified current REST:** [`GET /v5/market/orderbook`](https://bybit-exchange.github.io/docs/v5/market/orderbook) returns a current snapshot. For options, `limit` is 1–25 and defaults to 1. It supplies system timestamp `ts`, matching-engine timestamp `cts`, update ID `u`, cross-sequence `seq`, bids and asks. RPI orders are excluded. It has no historical timestamp or cursor.

**Verified prospective WS:** [`orderbook.{depth}.{symbol}`](https://bybit-exchange.github.io/docs/v5/websocket/public/orderbook) supports option depth 25 at 20 ms or depth 100 at 100 ms. The stream starts with a snapshot and then deltas; a new snapshot or `u=1` resets local state, and zero quantity deletes a level. Persist `u`, `seq`, `cts`, receipt time and snapshot/delta type so gaps cannot silently create a false book. RPI orders remain excluded.

**Observed:** A current REST query for an ATM-ish call returned 15 bid and 22 ask levels. `u` and `seq` happened to be equal in that response; the documentation defines distinct fields, so code must not assume equality.

**Historical availability:** No first-party historical order-book endpoint or retention promise was found. Historical L2 must be collected prospectively or sourced externally.

### Historical volatility, candles, index and delivery

**Verified:** [`GET /v5/market/historical-volatility`](https://bybit-exchange.github.io/docs/v5/market/iv) returns an hourly volatility series for `category=option`, `baseCoin`, `quoteCoin`, and averaging `period`. If time bounds are supplied, both `startTime` and `endTime` are required; each request may cover at most 30 days, and the endpoint exposes the last two years. Records contain only `period`, `value` and `time`.

This is an aggregate “historical volatility” series, not a contract-level historical IV surface: the rows contain no symbol, expiry or strike. It must not be used as if it were the mark IV of the ATM option sold at that timestamp.

**Observed:** A 24-hour USDT/BTC query with `period=7` returned 25 inclusive hourly points.

**Verified:** Bybit's [`/v5/market/kline`](https://bybit-exchange.github.io/docs/api-explorer/v5/market/kline) supports `linear`, `inverse` and `spot`, not `option`. It therefore does not provide one-minute option candles. Option trades can be aggregated into candles, but inactive intervals and bid/ask state remain unknown.

**Verified:** [`GET /v5/market/delivery-price`](https://bybit-exchange.github.io/docs/v5/market/delivery-price) accepts option `symbol`, `baseCoin`, `settleCoin`, `limit` (1–200, default 50) and `cursor`, returning `deliveryPrice` and `deliveryTime`. When no symbol is supplied, the option query is restricted during the documented `DELIVERING` window. Always query explicit symbols and exhaust cursors.

**Unknown:** No dedicated, documented historical option-underlying index series with caller-selected timestamps was found in the reviewed Bybit endpoints. Store `indexPrice` from trades/tickers plus a dedicated live index feed prospectively; perpetual OHLC is not automatically the option settlement index.

### Settlement and margin denomination

**Verified:** Bybit's [option introduction/specification](https://www.bybit.com/en/help-center/article/?id=000001543&language=en_US) describes European options, automatic exercise and cash settlement. USDT and USDC option contracts exist, and the final settlement price is based on an average index during the 30 minutes before expiry. The exact formula and currency must be taken from the selected instrument/specification version.

**Verified:** The [option margin calculation documentation](https://www.bybit.com/en/help-center/article/Initial-Maintenance-Margin-Calculations-Options) distinguishes long and short positions and account modes. A long's loss is bounded by premium; a short consumes initial/maintenance margin, and portfolio margin behaves differently from standard cross/isolated formula examples. The [liquidation documentation](https://www.bybit.com/en/help-center/article/?id=000001547) describes account-level liquidation behaviour.

Consequently, “70% sold now, 30% reserved” cannot be interpreted as 70%/30% notional alone. The backtest needs premium cash flow, contract multiplier, settlement currency, current account mode, exchange risk parameters, open positions, available margin, locked initial/maintenance margin, fees and liquidation rules at every event. Risk parameters may change; snapshot them prospectively.

### Rate and connection limits

**Verified:** Bybit's [rate-limit documentation](https://bybit-exchange.github.io/docs/v5/rate-limit) publishes a default HTTP IP limit of 600 requests per five seconds; after an IP `403`, it instructs clients to wait at least ten minutes. It also documents no more than 500 WebSocket connections per five minutes and no more than 1,000 market-data connections per IP, counted separately by market. UID endpoint limits are rolling per second and exposed through response headers; no stronger endpoint-specific public-market promise was found for the methods above.

**Verified:** The [WebSocket connection guide](https://bybit-exchange.github.io/docs/v5/ws/connect) gives the production option endpoint `wss://stream.bybit.com/v5/public/option`, documents an option limit of 2,000 subscription arguments per connection and 21,000 total characters in an args array, and recommends a heartbeat every 20 seconds.

## Deribit

### Historical host, instruments and trades

**Verified:** Production API transports are `https://www.deribit.com/api/v2/{method}` and `wss://www.deribit.com/ws/api/v2`, per the [JSON-RPC overview](https://docs.deribit.com/articles/json-rpc-overview). Deribit's official [Institutional Setup Guide, section 14](https://statics.deribit.com/files/DeribitInstitutionalSetupGuide.pdf) documents the historical host `https://history.deribit.com/api/v2/public/...`, says it contains trade and instrument information since the 2016 launch, is updated about five seconds after a trade, instructs clients to set `include_old=true`, and documents `count` up to 10,000 there.

The guide lists historical variants of `get_instrument(s)` and `get_last_trades_by_currency|instrument`, including time-bounded methods. This is the primary first-party backfill route for expired option metadata and executions.

**Observed:** The following official-history request returned BTC option trades from 2024 with `trade_seq`, `trade_id`, timestamp, price, amount/contracts, mark price, execution IV, index price, instrument and taker direction:

```text
GET https://history.deribit.com/api/v2/public/get_last_trades_by_currency_and_time
    ?currency=BTC&kind=option
    &start_timestamp=1704067200000
    &end_timestamp=1704067260000
    &count=3&sorting=asc
```

**Verified normal API:** [`get_last_trades_by_currency_and_time`](https://docs.deribit.com/api-reference/market-data/public-get_last_trades_by_currency_and_time) and the related [currency-ID method](https://docs.deribit.com/api-reference/market-data/public-get_last_trades_by_currency), [instrument sequence method](https://docs.deribit.com/api-reference/market-data/public-get_last_trades_by_instrument), and [instrument/time method](https://docs.deribit.com/api-reference/market-data/public-get_last_trades_by_instrument_and_time) accept at most 1,000 rows on the normal host and return `has_more`. The currency method supports `start_id`/`end_id`; instrument methods support `start_seq`/`end_seq`. The normal currency-and-time documentation explicitly limits results to the last 24 hours. Use the history host for older backfills.

Robust pagination should use stable IDs/sequences rather than time alone:

1. Request ascending rows with a bounded time range and maximum count.
2. Resume from the last `trade_id` (currency stream) or `trade_seq` (one instrument).
3. Treat the boundary as potentially inclusive: deduplicate the repeated last record.
4. Validate monotonic IDs and per-instrument sequences; persist raw IDs as strings if their representation is not guaranteed numeric.
5. Continue until `has_more=false`, then reconcile the next interval. Timestamp-only pagination can skip trades when more than one page shares a millisecond.

Trade `iv` is execution IV. It is sparse, selection-biased toward traded contracts and is not a continuous mark-IV surface.

**Verified:** [`GET public/get_instruments`](https://docs.deribit.com/api-reference/market-data/public-get_instruments) accepts `currency`, `kind` and `expired`, returns active/recently expired instruments, and has no pagination. Archive each discovery response and use the history host for old instruments. [`GET public/get_instrument`](https://docs.deribit.com/api-reference/market-data/public-get_instrument) provides creation/expiration timestamps, strike, option type, settlement and quote currencies, contract size, minimum amount and tick rules.

### Current 24-hour ATM BTC contracts and denomination

**Observed at 2026-08-25 14:51 UTC:** `btc_usd` was about USD 79,291; the nearest daily expiry was 2026-08-26 08:00 UTC (about 17 hours away) and nearest strike was 79,500. The inverse symbols were `BTC-26AUG26-79500-C` and `BTC-26AUG26-79500-P`. A current inverse call described `contract_size=1 BTC`, minimum 0.1 contract, base/quote/settlement currency BTC, USD counter currency, 0.0001 BTC base tick with a higher-price tick rule, and daily settlement. These values are observations; discover the live chain dynamically.

**Verified inverse specification:** [Inverse Options](https://support.deribit.com/hc/en-us/articles/31424939096093-Inverse-Options) are European and automatically exercised, cash-settled in BTC, quoted and margined in BTC, with strike against the USD BTC index. The contract size is 1 BTC, minimum size 0.1 contract and daily expiry is 08:00 UTC. The delivery value is the 07:30–08:00 UTC index TWAP; option cash value is converted to BTC using the delivery value. Therefore the statement “Deribit options are USDT-margined” is false for this instrument family.

**Verified linear alternative:** [Linear USDC Options](https://support.deribit.com/hc/en-us/articles/31424932728093-Linear-USDC-Options) use names such as `BTC_USDC-DDMMMYY-STRIKE-C|P`, are quoted, margined and settled in USDC, use a BTC multiplier of 1 and a 0.01-contract minimum, and expire daily at 08:00 UTC. At expiry Deribit describes physical delivery into a matching future followed immediately by USDC cash settlement, yielding the stated cash result. This is USDC, not USDT.

**Verified schedule:** The [Contract Introduction Policy](https://support.deribit.com/hc/en-us/articles/25944688876957) states that BTC inverse and linear option chains include one through four daily expiries, with daily contracts expiring at 08:00 UTC.

If cross-collateral is enabled, other assets may support the account, but the instrument PnL and risk remain calculated in its settlement currency; see [Cross-collateral specifications](https://support.deribit.com/hc/en-us/articles/25944777203869-Cross-collateral-specifications). Do not collapse inverse BTC and linear USDC into one “USDT-margined” model.

### Order books

**Verified current REST:** [`GET public/get_order_book`](https://docs.deribit.com/api-reference/market-data/public-get_order_book) accepts `instrument_name` and `depth=1..10000`. It returns a current book plus change ID, timestamp, mark/index/underlying values, best quotes, open interest and statistics. It has no historical time or cursor parameter.

**Verified prospective WS:** [`book.{instrument_name}.{interval}`](https://docs.deribit.com/subscriptions/orderbook/bookinstrument_nameinterval) supports `raw`, `100ms` and `agg2`; `raw` requires authorization. The first event is a full snapshot; subsequent actions are `new`, `change` or `delete` and carry `prev_change_id`/`change_id` for gap detection. A [grouped/truncated stream](https://docs.deribit.com/subscriptions/orderbook/bookinstrument_namegroupdepthinterval) offers depths 1, 10 or 20 at `100ms`/`agg2`.

**Historical availability:** No Deribit-hosted historical order-book endpoint or retention promise was found. Deribit's first-party [Tardis partnership announcement](https://insights.deribit.com/exchange-updates/celebrating-our-tardis-dev-partnership-get-free-historical-data/) documents a third-party tick-data route, but its advertised free period concerned Q4 2021/Q1 2022 and expired in 2022. It is not evidence of a current first-party bulk dataset or free entitlement.

### Tickers, IV, Greeks and trade streams

**Verified current REST:** [`GET public/ticker`](https://docs.deribit.com/api-reference/market-data/public-ticker) returns option mark price, mark/bid/ask IV, five Greeks (`delta`, `gamma`, `vega`, `theta`, `rho`), index/underlying values, best quotes, open interest and 24-hour statistics.

**Verified prospective WS:** [`ticker.{instrument_name}.{interval}`](https://docs.deribit.com/subscriptions/market-data/tickerinstrument_nameinterval) supports `raw` (authorized only), `100ms` and `agg2`. [`incremental_ticker.{instrument_name}`](https://docs.deribit.com/subscriptions/market-data/incremental_tickerinstrument_name) is a lower-frequency snapshot-then-changes alternative, at most once per second. [`markprice.options.{index_name}`](https://docs.deribit.com/subscriptions/market-data/markpriceoptionsindex_name), for example `markprice.options.btc_usd`, streams mark price and IV for the option chain.

Public trade streams are available by [`trades.{instrument_name}.{interval}`](https://docs.deribit.com/subscriptions/trades/tradesinstrument_nameinterval) or consolidated [`trades.option.BTC.{interval}`](https://docs.deribit.com/subscriptions/trades/tradeskindcurrencyinterval). Option executions include execution IV and contemporaneous mark/index price. Raw streams require authorization; `100ms` and `agg2` are alternatives.

**Historical availability:** No native historical ticker, bid/ask-IV, complete mark-IV-surface or Greeks endpoint was found. Historical trades do not fill this gap: execution IV is not mark IV, and trades have no historical Greeks. Record these streams prospectively.

### Historical mark, index and settlement

**Verified limited mark history:** [`GET public/get_mark_price_history`](https://docs.deribit.com/api-reference/market-data/public-get_mark_price_history) accepts instrument/start/end timestamps and returns five-minute `[timestamp_ms, mark_price]` points. It works only for the subset of options participating in volatility-index calculations; all other options, futures and perpetuals return an empty result.

**Unknown:** The endpoint documents no retention start, exact qualifying-option rule, pagination mechanism or maximum requested range. Test every selected contract and treat an empty array as missing data, not a zero price.

**Verified index:** Current index is available via [`public/get_index_price`](https://docs.deribit.com/api-reference/market-data/public-get_index_price) and prospectively via [`deribit_price_index.{index_name}`](https://docs.deribit.com/subscriptions/market-data/deribit_price_indexindex_name). Historical [`public/get_index_chart_data`](https://docs.deribit.com/api-reference/market-data/public-get_index_chart_data) accepts `index_name` and `range=1h|1d|2d|1m|1y|all`, returning chronological average-index points. The caller cannot select resolution or paginate.

**Observed:** `index_name=btc_usd&range=all` returned 14,601 points, beginning at timestamp `1472320800000` (2016-08-27) and ending near request time. This demonstrates current reach, not guaranteed retention or fixed sampling cadence.

**Verified delivery/settlement:** [`public/get_delivery_prices`](https://docs.deribit.com/api-reference/market-data/public-get_delivery_prices) accepts `index_name`, offset and count (default 10, maximum 1000), and returns `records_total`. [`public/get_last_settlements_by_instrument`](https://docs.deribit.com/api-reference/market-data/public-get_last_settlements_by_instrument) supports type, count up to 1000, continuation and search timestamp; a currency variant is also available. Persist the authoritative delivery price and settlement record for expiry PnL.

### Account logs are not public market data

**Verified:** [`private/get_transaction_log`](https://docs.deribit.com/api-reference/account-management/private-get_transaction_log) and the UI [Transaction Log](https://support.deribit.com/hc/en-us/articles/25944587269021-Transaction-log) can retrieve/export the user's account history back to account creation with continuation pagination. [Monthly Reports](https://support.deribit.com/hc/en-us/articles/25944616523037-Monthly-reports) are also account-specific.

These sources help reconcile live/paper execution and locked margin. They are not a public historical book, ticker, IV surface or Greeks archive.

### Rate and connection limits

**Verified:** Deribit's [Rate Limits](https://docs.deribit.com/articles/rate-limits) use credits. Documented defaults for non-matching-engine requests (including market data) are 500 credits/request, a 50,000-credit pool and 10,000-credit/second refill: 20 requests/second sustained and burst 100. `public/get_instruments` has a special 10,000 cost, 500,000 pool, one request/second sustained and burst 50. Subscribe methods cost 3,000 with a 30,000 pool, about 3.3 requests/second sustained and burst 10.

Unauthenticated public traffic is additionally limited per IP, but the documentation publishes no fixed numeric allowance. Excess usage can return `too_many_requests` (`10028`) and disconnect a session. The [JSON-RPC overview](https://docs.deribit.com/articles/json-rpc-overview) documents at most 32 WebSocket connections per IP and 16 sessions per API key.

## Data that must be captured prospectively

For both venues, run the collector before claiming a faithful replay:

| Stream/state | Why it is required |
|---|---|
| Complete instrument discovery snapshots | Reconstruct what was listed and selectable without survivorship/look-ahead bias |
| Option ticker / mark-price / mark-IV / bid-IV / ask-IV / Greeks | Price positions, run the 1.5x/2x IV stress, and explain vega/PnL |
| L2 snapshot plus deltas | Model executable spread, depth, reserve-leg fill and market impact |
| Public executions | Validate book evolution and provide fallback trade-driven pricing |
| Venue option index and underlying reference | ATM selection, pricing input and settlement reconciliation |
| Delivery price and settlement events | Final payoff and automatic exercise |
| Account/risk snapshots | Reproduce locked initial/maintenance margin, available collateral and liquidation |
| Fee/risk-parameter version | Avoid applying today's fees and margin formula to old trades |
| Collector health/gaps | Prevent a silent data gap from being interpreted as a quiet market |

The collector should first subscribe/discover the entire relevant expiry band, not only today's chosen strike. Otherwise tomorrow's ATM and reserve-sale strike may be absent. Reconcile WebSocket state periodically against REST snapshots without overwriting event time, and fail a replay interval on an unresolved sequence gap.

## Proposed normalized dataset

Use a venue-neutral envelope on every record:

```text
venue                 enum {BYBIT, DERIBIT}
channel               enum {INSTRUMENT, BOOK, TRADE, TICKER, INDEX, SETTLEMENT, RISK}
instrument_uid        stable internal uint64
venue_symbol          string
exchange_ts_ns        int64 UTC; null only when the venue supplies none
receive_ts_ns         int64 monotonic-wall correlated receive time
sequence              string/uint64 as supplied; nullable
source                enum {REST, WS, BULK_DOWNLOAD, DERIVED}
ingest_run_id          UUID/string
schema_version         uint16
is_gap                 bool
raw_ref                optional content-addressed raw-message reference
```

Keep venue identifiers as lossless strings at ingestion. Convert prices/quantities once into exact fixed-point integers using the instrument-version scales; never infer decimal scales from later instruments.

### `instrument_version`

```text
instrument_uid, venue_symbol, valid_from_ts_ns, valid_to_ts_ns
base_ccy, quote_ccy, settlement_ccy, collateral_ccy, counter_ccy
option_style, option_type, strike_ticks, strike_scale
creation_ts_ns, expiry_ts_ns, settlement_period
contract_model          enum {LINEAR, INVERSE}
contract_multiplier_num, contract_multiplier_den, multiplier_ccy
min_qty_ticks, qty_scale
price_tick_rules_json   # threshold-dependent ticks are versioned, not flattened
underlying_index_name, settlement_index_name
```

Do not force `contract_multiplier` to an integer: Deribit inverse/linear and Bybit contracts need explicit rational multiplier and currency semantics. Do not assume quote currency equals account currency.

### `book_event`

```text
instrument_uid, exchange_ts_ns, receive_ts_ns
event_kind              enum {SNAPSHOT_BEGIN, SNAPSHOT_LEVEL, SNAPSHOT_END, UPSERT, DELETE}
side                    enum {BID, ASK}
price_ticks, qty_ticks  int64
sequence, previous_sequence, cross_sequence
depth, aggregation_ms, is_rpi_excluded, is_gap
```

For replay efficiency this event table could be converted into the canonical
snapshots/trades and native replay cache of `back-tester-2026`, but the raw
normalized delta stream should remain auditable. This is not an available
adapter in `back-tester-rust`. Venue quantity semantics must be converted
through `instrument_version`; reject non-exact conversion.

### `option_trade`

```text
instrument_uid, trade_id, trade_sequence
exchange_ts_ns, receive_ts_ns, taker_side
price_ticks, qty_ticks
mark_price_ticks, index_price_ticks
execution_iv_ppm, mark_iv_ppm
is_block_trade, is_rpi_trade, is_liquidation
```

IV units require an explicit convention. Store normalized annualized decimal IV as parts per million after validating each venue's wire convention, and retain the raw value. Null means unavailable; never substitute the current mean IV.

### `option_ticker`

```text
instrument_uid, exchange_ts_ns, receive_ts_ns
bid_price_ticks, bid_qty_ticks, ask_price_ticks, ask_qty_ticks
last_price_ticks, mark_price_ticks, index_price_ticks, underlying_price_ticks
bid_iv_ppm, ask_iv_ppm, mark_iv_ppm
delta_ppm, gamma_scaled, vega_scaled, theta_scaled, rho_scaled
open_interest_ticks, volume_24h_ticks
```

Store each Greek with a documented scale and the venue's definition/unit. Greeks are optional observations, not authoritative PnL; the pricing engine should recompute them from the same spot/forward, IV, rate, expiry and settlement convention used in the scenario.

### `index_observation` and `settlement_event`

```text
index_observation:
  venue, index_name, exchange_ts_ns, receive_ts_ns, price_ticks, sampling_method

settlement_event:
  instrument_uid, settlement_ts_ns, delivery_price_ticks
  option_payoff_settlement_ticks, fee_settlement_ticks
  settlement_ccy, source_record_id
```

### `account_risk_snapshot`

```text
venue, account_id_hash, exchange_ts_ns, receive_ts_ns
margin_mode, collateral_ccy
equity_ticks, available_balance_ticks
initial_margin_ticks, maintenance_margin_ticks
locked_margin_ticks
position_instrument_uid, signed_qty_ticks, average_entry_price_ticks
risk_parameter_version
```

This is needed for the proposed 70% initial allocation / 30% reserve logic. Strategy capital, order premium/notional and exchange-locked margin are three separate quantities.

### Dataset manifest and quality contract

Each partition should additionally declare:

```text
venue, date, channel, schema_version
first_exchange_ts_ns, last_exchange_ts_ns, row_count
first_sequence, last_sequence, gap_count, duplicate_count
instrument_metadata_hash, price_scale, qty_scale
source_urls, acquisition_time, checksum
completeness          enum {COMPLETE, PARTIAL, UNKNOWN}
missing_intervals[]
```

Historical `back-tester-2026` note: that repository's manifest is
one-instrument and its hot path assumes exact integer quantities and quote
currency equal to account currency. It is not a component of
`back-tester-rust`. A future historical-option replay would need a separate
multi-instrument manifest/adapter or a versioned extension; silently squeezing
inverse BTC-settled options into a linear cash model would produce incorrect
PnL and margin.

## Pricing and backtest constraints

1. Use Black-Scholes only for linear European options under internally consistent inputs. Deribit inverse BTC-settled options have nonlinear currency conversion; implement the exchange payoff/margin semantics or choose its linear USDC family.
2. “Both legs are ATM, therefore loss is linear” is not generally valid. Option price is nonlinear in spot, IV and time; delta, gamma, vega and theta change as the market moves and expiry approaches. A second sale after IV rises has a different premium and risk state.
3. A 2x IV shock must specify whether it multiplies decimal IV, volatility points, or variance; cap/floor rules, skew movement and bid/ask execution must be explicit.
4. Select the option using only chain/index information available at that historical decision time. Persist the rejected candidates or the discovery snapshot to prove absence of look-ahead.
5. Revalue and settle in the instrument's settlement currency, then apply a separately timestamped FX/index conversion for account-level USD/USDT reporting.
6. Backtest the margin state event-by-event. A reserve can become unusable because the short's maintenance/initial margin rises before the 50% IV trigger is reached.

## Remaining unknowns to resolve before implementation

- Does Bybit's current Historical Market Data inventory actually include BTC USDT option executions for the required dates, and what are its earliest date and completeness rules?
- Does Bybit offer any unlinked first-party bulk option ticker/book dataset? None was found in the reviewed official API/docs.
- What are the exact historical margin/risk parameters and fees for each test date and account mode? Current formulas alone are not historical evidence.
- Which Bybit venue index exactly settles each chosen USDT option, and is a historical first-party series downloadable at the required resolution?
- Which Deribit contracts qualify for mark-price history over each desired date range, and what are that endpoint's practical range/retention limits?
- What target execution fidelity is acceptable if only trades exist: mid, pessimistic bid/ask proxy, or “unfillable/unknown”? This is a model decision, not recoverable market data.

Until these are resolved, the defensible first milestone is: ingest Deribit official historical option trades/instruments plus index/delivery history, run a clearly labelled trade/BS scenario backtest, and simultaneously begin prospective full-chain collection on Bybit and Deribit for a later executable-price and margin-aware replay.
