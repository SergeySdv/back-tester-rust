# Where to obtain historical BTC option data for Bybit and Deribit

Review date: **2026-08-25**. Purpose: a causal backtest of selling the nearest
approximately 24-hour ATM call and put, with capital reserved for a second sale
after IV rises, followed by realistic modeling of execution, fees, expiry, and
exchange margin risk.

The research is based only on official exchange documentation and APIs and on
the vendors' own pages, catalogs, APIs, and terms. Marketing claims without a
verifiable catalog are marked as unverified. Prices and catalogs may
change after the specified date.

> Repository attribution: mentions below of the existing native replay cache,
> multi-instrument L2 runtime and single-instrument manifest refer to the separate
> `back-tester-2026` repository, which was explored earlier. The current
> `back-tester-rust` repository has no such runtime/manifest; its MVP is a synthetic
> scenario backtest from `docs/architecture/`.

## Executive summary

1. **The best ready-made self-service source for both exchanges is Tardis.dev.** It
   documents Bybit Options from 2023-04-05 and Deribit, including options, from
   2019-03-30; gives trades, BBO, L2, snapshots and option-chain with IV/Greeks.
   However, the first **Bybit BTC-USDT option** found in its public catalog starts
   on 2025-02-19: earlier Bybit coverage cannot automatically be treated as
   USDT-margined. Data for the first day of each month can be downloaded without
   a key and audited before purchase.
   Public Options plan starts from **$350/month**, but Academic/Solo/Professional
   receive a full four-year history only with annual payment; older
   history and one-time sampling vary by order. [Coverage
   Deribit](https://docs.tardis.dev/historical-data-details/deribit), [CSV and
   free days](https://docs.tardis.dev/downloadable-csv-files), [current
   price and conditions](https://tardis.dev/).
2. **Amberdata is a strong institutional alternative**, especially if you need
   ready-made minute option-chain/IV/Greeks together with tick-level order-book
   events. Detailed coverage confirms Bybit options market datasets with
   2024-03-15 and different Deribit dataset starts from 2021-11-16; a separate
   marketing page states Deribit trades start on 2021-05-21. No public fixed price
   is shown for the required package; an online order or quote is required. A
   monthly plan gives one year of history, an annual plan gives full history, and
   S3 is a separate add-on. [Deribit
   coverage](https://www.amberdata.io/deribit-market-data), [Bybit option L2
   example](https://docs.amberdata.io/http/market/options-order-book-events),
   [ordering FAQ](https://www.amberdata.io/online-market-data-ordering-faq).
3. **CoinAPI is a possible flexible pay-as-you-go way**, especially for selective
   trades/quotes/L2 flat files. The official catalog shows a separate
   `BYBITOPT` only from 2025-10-02; Deribit options are declared. But public pages
   do not establish that historical option-chain IV/Greeks exist in flat files,
   and the exact depth of each series must be checked in the symbol catalog after
   obtaining a key. PAYG L2 pricing starts at $8/GiB for the first GiB of each
   SKU per day; new users are promised $25 in credits after payment verification.
   [BYBITOPT catalog](https://www.coinapi.io/products/market-data-api/docs/metadata-tables/supported-exchanges/exchanges_B),
   [Flat Files](https://www.coinapi.io/products/flat-files), [pricing](https://www.coinapi.io/products/flat-files/pricing).
4. **Kaiko and Coin Metrics are worthy of a vendor trial, but not a blind buy.** Kaiko
   publicly displays historical Deribit and Bybit BTC-USDT option instruments
   and a convenient one-minute derivative-price file; however, the exact dates of L2/IV by venue and
   pricing require a quote. Coin Metrics' public catalog confirms both venues,
   instrument lifecycle, snapshots, IV and Greeks, but ranges of different datasets
   are not the same and there is no public commercial price. [Kaiko derivative
   price details](https://docs.kaiko.com/cloud-delivery/data-feeds/reference-data/derivatives-price-details),
   [Coin Metrics market data](https://docs.coinmetrics.io/market-data).
5. **Official free backfill is sufficient for an MVP only on Deribit.**
   Deribit provides trades/expired instruments since 2016 via history host, as well as
   index and settlement/delivery history. This allows you to test signals and
   estimate rough PnL, but not model causal execution from bid/ask/L2 or a continuous
   IV surface. Bybit public download on the verified page lists Spot and
   Contract, but not Options; public REST option trades are limited to recent
   window. [Deribit history host specification](https://statics.deribit.com/files/DeribitInstitutionalSetupGuide.pdf),
   [Bybit historical download inventory](https://www.bybit.com/derivatives/vi-VN/history-data).
6. **A prospective collector must be started immediately in every scenario.**
   No backfill will restore historical account equity,
   locked IM/MM, reserve availability, fee tier and specific portfolio version
   margin. These states must be recorded from your account and versioned
   along with the public market.

For the first realistic result, I would buy the **full venue-wide BTC option
chain**, but only for 1–3 months initially, from Tardis for Bybit Options and
Deribit. Preselecting only “ATM symbols” is impossible without look-ahead or
selection bias because the ATM strike changes with the underlying. I would start
the collector in parallel. Before paying, request the exact price of a one-off
sample from Tardis and confirm that the supplied symbols are USDT-settled Bybit
options and the selected Deribit family. On the review date, the public one-off
form showed $10 per venue/day for all instruments, $20/day for all option venues,
$2 per symbol/day, and a $300 minimum order; checkout or quote terms must be
rechecked before payment. For a high-precision multi-year short-volatility test,
Tardis is shortlist No. 1 and Amberdata No. 2 after a comparative sample audit.

## What exactly does a strategy need?

A one-minute `BTCUSDT` perpetual series supplies only the underlying signal. At
each historical entry timestamp, the backtest needs **a complete set of contracts
that were actually listed** so it can causally select the nearest future expiry
and ATM strike instead of choosing a contract retroactively. For both the call
and put it needs:

- instrument creation/expiry, strike, option type, multiplier, currencies,
  tick/quantity rules and listing status;
- exchange timestamp and local receive timestamp;
- BBO with size or L2 snapshot + deltas with sequence identifiers;
- public trades with trade ID/sequence and aggressor side;
- mark price, bid/ask/mark IV, underlying/index and preferably exchange Greeks;
- authoritative delivery price and settlement record;
- existing maker/taker/delivery/liquidation fees;
- account margin mode, equity, available balance, order/position IM, MM and
  locked/occupied margin before and after each action.

The last item is not ordinary public market data. Bybit explicitly states that
short-option order IM uses the index, order/mark price, OTM amount, IM factors,
MM, fee, and received premium; therefore “70% of capital” cannot be replaced by
70% of premium or 70% of notional. [Bybit option IM/MM
formula](https://www.bybit.com/en/help-center/article/Initial-Maintenance-Margin-Calculations-Options).
Deribit recalculates IM/MM continuously, and at 100% usage a new increasing risk
order is not possible. [Deribit margin behavior](https://support.deribit.com/hc/en-us/articles/25944811089565-What-is-margin).

Model clarification: two ATM legs do not make the loss linear. A short ATM call
and short ATM put both have negative vega, so both sold legs become more expensive
when IV rises; a linear-vega approximation works only for a small local shock.
With `IV × 2`, spot movement and expiry approaching, vega/gamma/delta change, and
the short straddle has a nonlinear payoff. Reserve selling increases short vega,
short gamma, and margin demand precisely in the stressed state. The report must
therefore show full bid/ask-surface repricing, variation PnL, and recalculated
IM/MM separately, not one linear adjustment to a “second leg.”

## A. Free official backfill sources

### Deribit

Deribit is the only one of the two venues where the official free backfill is
already useful for an MVP:

- `history.deribit.com` stores public trades and instruments since launch in 2016,
  updates in about five seconds and allows `count=10000` with
  `include_old=true`; this is documented in section 14 of the official
  [Institutional Setup Guide](https://statics.deribit.com/files/DeribitInstitutionalSetupGuide.pdf);
- historical trade contains execution price, execution IV, contemporaneous
  mark price and index price, trade ID and per-instrument trade sequence;
- expired instruments are available through `get_instruments(... expired=true)` and
  `get_instrument`; delivery prices and settlements have separate pagination
  endpoints. [Trade API](https://docs.deribit.com/api-reference/market-data/public-get_last_trades_by_currency_and_time),
  [instruments](https://docs.deribit.com/api-reference/market-data/public-get_instruments),
  [delivery prices](https://docs.deribit.com/api-reference/market-data/public-get_delivery_prices),
  [settlements](https://docs.deribit.com/api-reference/market-data/public-get_last_settlements_by_instrument).

Reproducible query without key:

```bash
curl --get 'https://history.deribit.com/api/v2/public/get_last_trades_by_currency_and_time' \
  --data-urlencode 'currency=BTC' \
  --data-urlencode 'kind=option' \
  --data-urlencode 'start_timestamp=1704067200000' \
  --data-urlencode 'end_timestamp=1704067260000' \
  --data-urlencode 'count=3' \
  --data-urlencode 'sorting=asc'
```

Pagination should continue along the stable `trade_id`/`trade_seq`, with
dedup on the inclusive boundary and checking `has_more`; timestamp-only cursor
may miss trades if one millisecond contains more than one page.

The limitations are critical: no official historical L2/BBO/ticker/bid-IV/ask-IV/Greeks
archive was found. `get_mark_price_history` returns a 5-minute mark only
for a subset of options participating in volatility-index calculation; an empty
response for the rest means missing data, not zero. [Mark-price history](https://docs.deribit.com/api-reference/market-data/public-get_mark_price_history).

### Bybit

The official download page on the date of verification lists `Public Trading
History`, index/premium klines and order book only for **Spot/Contract**, without
separate Options inventory. Therefore, the presence of futures files cannot be
assumed to imply option files. [Bybit Historical Market Data](https://www.bybit.com/derivatives/vi-VN/history-data).

Public V5 API helps only partially:

- instrument discovery and current ticker/book are suitable for prospective
  collection;
- `recent-trade` returns at most 1,000 recent rows and has no historical
  time pagination;
- `historical-volatility` — underlying volatility hourly unit, maximum
  two years and up to 30 days per request, **not** historical contract-level IV surface;
- option kline endpoint is missing.

See [instrument info](https://bybit-exchange.github.io/docs/v5/market/instrument),
[recent trades](https://bybit-exchange.github.io/docs/v5/market/recent-trade),
[historical volatility](https://bybit-exchange.github.io/docs/v5/market/iv) and
[tickers](https://bybit-exchange.github.io/docs/v5/market/tickers).

Account export is useful only for the account's own executions and history; it is
not public chain/order-book archive. The current help page allows options
trade/delivery history and USDC options orders, but does not reconstruct the
market that the trader saw. [Bybit self-export](https://www.bybit.com/en/help-center/article/How-to-Self-Export-Account-Data).

## B. Own prospective collector

### What to collect

For each exchange, save exchange-native payload before normalization and separately
normalized tables:

| Flow | Bybit | Deribit | Purpose |
|---|---|---|---|
| Instruments | REST full pagination, snapshot every 1–5 min | `get_instruments`, then `get_instrument` | Listing, expiry, multiplier, currencies, tick rules |
| L2 | `orderbook.25/100.{symbol}` snapshot+deltas | `book.{instrument}.raw/100ms` snapshot+deltas | Executable spread/depth and gap detection |
| Trades | `publicTrade.BTC` | `trades.option.BTC.raw/100ms` | Real executions and book validation |
| Option state | `tickers.{symbol}` | `ticker.*` and `markprice.options.*` | bid/ask/mark IV, mark, Greeks, OI |
| Index/underlying | venue index WS/REST | `deribit_price_index.*` | ATM selection, BS inputs, settlement |
| Expiry | delivery-price REST | delivery/settlement REST | Payoff and fee reconciliation |
| Private risk | wallet/position/order/account WS + REST snapshots | account summary/positions/orders | available balance, IM/MM, locked margin |

Bybit option order book is documented as depth 25 every 20 ms or depth 100
every 100 ms, ticker - 100 ms. [Bybit order book](https://bybit-exchange.github.io/docs/v5/websocket/public/orderbook),
[Bybit ticker](https://bybit-exchange.github.io/docs/v5/websocket/public/ticker).
Deribit book stream gives `prev_change_id`/`change_id`, which allows you to detect
a gap; `raw` requires authorization. [Deribit book stream](https://docs.deribit.com/subscriptions/orderbook/bookinstrument_nameinterval).

### Minimal infrastructure

The pilot does not need a distributed system:

```text
2 venue collector processes
  -> append-only compressed raw frames (hourly/day partitions)
  -> sequence/gap validator + REST resnapshot
  -> deterministic normalizer
  -> Parquet + manifest/checksums
  -> native replay cache adapter from `back-tester-2026` or a new adapter
```

The practical minimum is one always-on VM in a nearby region, a separate backup
process/host, local NVMe spool, object storage and heartbeat monitoring,
message rate, sequence gaps, reconnects, clock offset, and delayed uploads. Compute
load is small relative to storage and traffic; no exact storage budget can honestly
be stated before a seven-day pilot of the full relevant expiry band. Measure
raw compressed bytes/day separately by channel/venue, p95 messages/s and worst-hour
during sudden movement, then multiply by retention + at least 30% headroom.
As an initial planning configuration, not a benchmark, use two independent hosts
with 4 vCPU/16 GB RAM and a 500 GB NVMe spool each, plus versioned object storage. After
seven days it must be reduced/increased by actual bytes/day, reconnect burst
and lag; a single host or shared disk does not provide independent proof of gaps.

Operational risks:

- WS reconnect creates an irrecoverable gap until a new snapshot; snapshot
  repairs the future book, but not the events inside the gap;
- the ATM strike and next daily expiry change, so subscribing only to the
  current two legs creates survivorship bias;
- venue changes schema, tick rules, symbols, fee and margin algorithms;
- an exchange timestamp can be identical for many events; raw receive order and
  local timestamp must be preserved as a causal tie-breaker;
- the private account stream may diverge from the public market stream; use
  periodic REST reconciliations without rewriting event time;
- one collector does not provide historical data until the launch date.

## C. Purchasing data

### Comparison matrix

| Supplier | Bybit options | Deribit options | Required datasets | History | Delivery | Price/trial | Verdict |
|---|---|---|---|---|---|---|---|
| **Tardis.dev** | All options from 2023-04-05; BTC-USDT found from 2025-02-19 | Confirmed from 2019-03-30 | trades, quotes, L2, snapshots, options_chain IV/Greeks | Catalog per symbol/day, including expired instruments | gzip CSV, raw replay API, client libraries | Options $350/$700/$1000/$3000 per month; one-off $10/venue/day, min $300; first day of the month free | **Shortlist No. 1** |
| **Amberdata** | Options market datasets from 2024-03-15 | Dataset starts 2021-11-16…12-05; marketing trade date 2021-05-21 | trades, BBO/tickers, order-book events + 1-min snapshots, OHLCV/OI; L1 IV/Greeks at 1 min | REST books 18 months; older through annual S3/Snowflake/bulk | REST, WS, CSV, S3 add-on, Snowflake | No public fixed price for the required bundle; trial 20k calls/day | **Shortlist No. 2** |
| **CoinAPI** | `BYBITOPT` from 2025-10-02 | Options confirmed; exact symbol start through catalog | trades, quotes, L2 flat files; current option chain; historical IV/Greeks flat-file not established | Per symbol; catalog/API key | S3-compatible CSV, REST/WS, Snowflake | PAYG; L2 $8/$4/$2 per GiB daily tiers; $25 credits after payment verification | Only after sample audit |
| **Kaiko** | BTC-USDT traded instrument from 2025-02-19 | Inverse BTC trade history from 2016; BTC-USDC instrument from 2025-08-19 | trades, BBO, reference; minute derivative-price fields; exact option L2 range not published | Reference catalog preserves past instruments; dataset ranges by request | REST/stream, daily CSV, AWS/Azure/GCP, Snowflake/BigQuery | Request quote/trial; no individual self-service found | Enterprise candidate after trial |
| **Coin Metrics** | Catalog confirms USDT/USDC options, snapshots, IV/Greeks | Catalog confirms options, snapshots, IV/Greeks | trades, quote/order-book snapshots, contract prices, IV, Greeks, metadata | Per-market min/max publicly queryable; datasets have different ranges | REST JSON/JSON-stream, WS | Community offers only a recent showcase; Professional requires demo/quote | **Shortlist No. 3** after continuity audit |
| **Crypto Lake** | Limited set of Bybit pairs only, no option chain | Venue absent from coverage | trades/books/candles for spot/futures pairs | Bybit from 2023-10-27, but not options | Python API, Parquet/S3 | Individual subscription/sample, 300 GB soft limit | Not suitable |
| **Databento** | Not included in the official venue offering | Absent | CME crypto options exist, but on a different venue/product | — | API/flat files | $125 credits apply to the available catalog | Not suitable |

Below are the details and verifiable limitations of the main options.

### Tardis.dev

Tardis saves exchange WebSocket payload without modification, sets local
timestamp with 100 ns precision, selects the most frequent available feed, checks
order-book sequences where the exchange provides them, publishes incidents and
resubscribes daily to obtain a new snapshot. Resubscription can create a documented
gap of approximately 300–3000 ms, so even the vendor feed is not
mathematically perfect. [Collection methodology](https://docs.tardis.dev/historical-data-details/overview).

The public metadata API returned as of the check date:

```text
bybit-options.availableSince = 2023-04-05T00:00:00Z
datasets.exportedUntil       = 2026-08-25T00:00:00Z
OPTIONS dataTypes            = trades, quotes, options_chain
individual option dataTypes  = trades, incremental_book_L2, quotes,
                               book_snapshot_5, book_snapshot_25,
                               options_chain
channels                     = publicTrade, orderbook.25, orderbook.100, tickers
```

`availableSince=2023-04-05` refers to the entire Bybit options venue, not to
USDT family. Filtering all BTC symbols of the public catalog by suffix
`-USDT` gave the first `availableSince=2025-02-19`; early Bybit option symbols without
suffixes refer to another contract/settlement line. This needs to be checked by
metadata of each instrument, and not by the general date of the venue. [Bybit Options coverage
and channels](https://docs.tardis.dev/historical-data-details/bybit-options).

Reproducing the catalog without a key:

```bash
curl -fsSL 'https://api.tardis.dev/v1/exchanges/bybit-options' \
  -o bybit-options.metadata.json
curl -fsSL 'https://api.tardis.dev/v1/exchanges/deribit' \
  -o deribit.metadata.json
```

Free sample, tested 2026-08-25:

```bash
curl -fL \
  'https://datasets.tardis.dev/v1/bybit-options/options_chain/2023/05/01/BTC-2MAY23-30000-C.csv.gz' \
  -o bybit-options_chain-2023-05-01.csv.gz
curl -fL \
  'https://datasets.tardis.dev/v1/bybit-options/incremental_book_L2/2023/05/01/BTC-2MAY23-30000-C.csv.gz' \
  -o bybit-L2-2023-05-01.csv.gz
gzip -cd bybit-options_chain-2023-05-01.csv.gz | head
```

In fact, the sample contained exchange/local timestamps, strike/expiry,
bid/ask prices and amounts, bid/ask/mark IV, mark, underlying, delta/gamma/vega/
theta; the L2 sample contained a snapshot flag, side, price, and amount. This is much closer
to the required backtest contract than a trade-only source. Field formats are
officially described in [CSV data types](https://docs.tardis.dev/downloadable-csv-files/data-types).
Verified compressed sizes of a specific active contract
`BTC-2MAY23-30000-C` for sample day: quotes 70,617 B, options_chain 3,737,722 B,
incremental L2 138,655 B and snapshot-25 140,040 B; grouped venue trades for the day
occupied 307,196 B. This checks availability and schema, **not** the full-chain size:
the size depends sharply on the number of contracts and the volatility of the regime.
Normalized L2, however, does not preserve the native sequence ID; for a strict gap audit
raw replay is required. A grouped `OPTIONS` snapshot file also cannot be assumed
available for every datatype: the grouped `book_snapshot_25` check returned
HTTP 400 while individual-symbol files were accessible.

For Deribit Tardis additionally archives raw `instrument.state.any`,
`estimated_expiration_price`, `deribit_price_index`, `markprice.options`, `book`
with `prev_change_id`, while ticker/quote has been collected from 2019-10-01. This provides lifecycle and
settlement linkage that normalized CSV alone does not provide. [Deribit channel
history](https://docs.tardis.dev/historical-data-details/deribit).

API limits on published page: 3,000 requests/min for
Academic/Solo/Professional, 9,000 for Business, up to 50 symbols in filter; A/S/P
limited to one active key and source IP, transfer allowance 20 TB/month vs.
60 TB for Business. [Tardis rate limits](https://docs.tardis.dev/api/rate-limits).

The license allows internal access/storage/manipulation and derived data, but
prohibits resale/redistribution of source data except for specifically permitted
aggregation with a minimum resolution of 10 minutes. The contract must be checked
before transferring a dataset to a contractor or publishing results. [Tardis Terms, clauses
8–10](https://docs.tardis.dev/legal/terms-of-service).

### Amberdata

Deribit marketing page states tick-by-tick trades from 2021-05-21, but more
detailed official coverage dictionary shows different dataset start dates:
Deribit OI 2021-11-16, ticker/trades 2021-11-17, book events 2021-11-23,
snapshots 2021-12-02 and OHLCV 2021-12-05. For Bybit options OI, snapshots,
events, ticker, and trades catalogs indicate 2024-03-15. Therefore, procurement
must be assessed by dataset row, and the earlier marketing date confirmed with a sample file.
[Exchange coverage dictionary](https://docs.amberdata.io/data-dictionary/coverage/exchange-coverage.md),
[Deribit market page](https://www.amberdata.io/deribit-market-data).

Option-book events docs show `exchange=bybit`, exchange timestamp,
sequence and per-price-level replacement; size zero removes the level. One REST
request is limited to a one-hour range. [Bybit event semantics and one-hour
limit](https://docs.amberdata.io/http/market/options-order-book-events).
Tick trades contain side/price/volume, index, mark, underlying, and execution IV;
`sequence` is nullable, the REST window is also at most one hour, and backfill uses
a cursor. [Option trades](https://docs.amberdata.io/http/market/options-trades).
Minute snapshots are taken from venue REST, have venue-dependent depth and include
underlying, statistics, OI and Greeks. [Snapshot
specification](https://docs.amberdata.io/http/market/options-order-book-snapshots).
Information endpoint supports `includeInactive=true` and per-contract
availability, allowing expired series to be discovered. [Order-book event
information](https://docs.amberdata.io/http/market/options-order-book-events-information).

`analytics/volatility/level-1-quotes` accepts `deribit|okex|bybit`, stores the
first observation of every minute/hour/day, although ingest occurs every 100 ms, and
returns bid/ask/mark IV, Greeks, OI, index/underlying and `isCarryForward`.
A minute request is limited to 60 intervals and an hourly request to 24, so bulk backfill
via REST requires full cursor/window automation or S3. [Level-1 quotes
specification](https://docs.amberdata.io/http/analytics/derivatives/level-1-quotes).

Trial: 15 calls/s and 20k calls/day; paid on-demand: 20 calls/s and 250k/day.
Monthly subscription gives one year of lookback, annual gives full history, and S3 requires
yearly + add-on. Standard license prohibits redistribution/resale/sublicensing;
extended rights require a custom sales agreement. [API limits](https://docs.amberdata.io/http/http-api-fundamentals),
[history, S3 and license](https://www.amberdata.io/online-market-data-ordering-faq).
From 2025-05-15 REST order-book history is limited to the last 18 months; older
books are available via S3, Snowflake, or bulk delivery. [REST retention
change](https://docs.amberdata.io/changelog/rest-access-historical-order-book-limited).
A separate decorated-trades catalog starts Deribit 2021-09-01, Bybit
2024-06-01; this is another example of why the earliest date must be recorded separately
for each product. [Decorated trades
coverage](https://docs.amberdata.io/data-dictionary/analytics/derivatives/decorated-trades).

There is no public numeric price for the required bundle: online ordering/startup/enterprise
lead to the form or quote. [Amberdata pricing](https://www.amberdata.io/pricing).
Website terms require you to stop using and destroy retained Data after
the subscription ends unless the order form grants other rights; the right to retain
purchased backfill after cancellation must be agreed upon in writing. [Amberdata
terms](https://www.amberdata.io/terms).

Before paying, request:

1. earliest Bybit **options** date for trades, events, snapshots and tickers;
2. raw tick frequency versus 1-minute snapshots;
3. expired USDT option symbols and instrument reference history;
4. exact S3 schema/sample for both venues;
5. gap/completeness report and correction policy;
6. individual/sole-trader eligibility and final price from S3.

### CoinAPI

CoinAPI Flat Files contains trades, quotes and full limit-order-book CSV via
S3-compatible API; from 2026-06-09 new high-frequency partitions hourly, old
remain daily. `coinapi-daily-tail` stores the previous day for only 24 hours and is
not an archive. [S3 layout/retention](https://www.coinapi.io/products/flat-files/docs/s3-api),
[dataset layout](https://www.coinapi.io/products/flat-files/docs/datasets).

The strong point is two timestamps: exchange and CoinAPI receive; quote schema
clearly describes both. [Quotes schema](https://www.coinapi.io/products/flat-files/docs/datasets/quotes).
If the source quote does not contain an exchange timestamp, the schema permits substituting the
receipt timestamp, so this field cannot definitely be interpreted as a venue event
time.
The weakness for this task is that the options REST endpoint documents the current grouped
chain, while the official flat-file pages list trades/quotes/books but do not
establish historical mark IV/Greeks. [Current options endpoint](https://www.coinapi.io/products/market-data-api/docs/rest-api/options/options/exchange_id/current/get).

Public metadata table shows `BYBITOPT` with trade start 2025-10-02; this is
shorter than Tardis and insufficient for a multi-year Bybit test. Exact Deribit/Bybit
contract coverage and file sizes must be obtained through paid catalog/listing, which
consumes credits itself. [Exchange metadata](https://www.coinapi.io/products/market-data-api/docs/metadata-tables/supported-exchanges/exchanges_B),
[estimation guide](https://www.coinapi.io/products/flat-files/docs/estimation-guide).

PAYG pricing resets data-type tiers every UTC day: L2 - $8/GiB per
first GiB, $4/GiB for next 9 and $2/GiB above 10; committed plans begin
from $64/month. [Flat Files pricing](https://www.coinapi.io/products/flat-files/pricing),
[pricing semantics](https://www.coinapi.io/products/flat-files/docs/use-cases).
New users receive $25 in credits only after payment verification and
key creation. [Flat Files FAQ](https://www.coinapi.io/products/flat-files/faq).
Metadata stores per-symbol `data_trade_start`, `data_quote_start` and
`data_orderbook_start`, and a separate history endpoint stores delisted symbols;
the exact inventory must be exported and verified before purchase rather than inferred from
venue-level marketing. [CoinAPI Market Data
FAQ](https://www.coinapi.io/products/market-data-api/faq).
Standard usage allows internal backtesting/trading, but not redistribution of the
raw or normalized feed. [CoinAPI usage policy](https://www.coinapi.io/usage-policy).

### Kaiko

Kaiko's public reference API, queried without a key, confirmed venue codes `drbt`
(Deribit) and `bbit` (Bybit V2). In the sorted instrument catalog, the earliest
traded Deribit inverse BTC option was `BTC-31MAR17-700-P`, with a trade timestamp
of 2016-11-29; the earliest Bybit BTC-USDT option found was
`BTC-4APR25-84000-P-USDT`, from 2025-02-19. The earliest traded Deribit linear
`BTC_USDC` option found was only from 2025-08-19. These are **the first trades in
the catalog**, not promises of when BBO/L2/IV archives begin.

```bash
curl --compressed \
  'https://reference-data-api.kaiko.io/v1/instruments?exchange_code=drbt&class=option&base_asset=btc&trade_count_min=1&orderBy=trade_start_timestamp&order=1&limit=3'
curl --compressed \
  'https://reference-data-api.kaiko.io/v1/instruments?exchange_code=bbit&class=option&base_asset=btc&quote_asset=usdt&trade_count_min=1&orderBy=trade_start_timestamp&order=1&limit=1'
```

Reference endpoint saves listing timestamp, contract size and expired option
definitions, but time filters for listings/expiries are only documented for
Deribit and page size maximum 1,000. [Kaiko derivatives contract
details](https://docs.kaiko.com/rest-api/data-feeds/reference-data/advanced-tier/derivatives-contract-details).

The closest ready-made file to a minute strategy is cloud **Derivative Price
Details**: bid/ask price and amount, bid/ask IV, mark/index/last, expiry/strike,
Greeks, underlying index, settlement price and timestamp. [Cloud schema](https://docs.kaiko.com/cloud-delivery/data-feeds/reference-data/derivatives-price-details).
Exchange-provided metrics are available at a default 1m resolution and at 1h/4h/1d, with page size up to 1,000,
and include OI/TTE/IV/Greeks/settlement; generic history “since July 2020” does not prove
the same start for each venue. [Derivatives risk indicators](https://docs.kaiko.com/rest-api/analytics/derivatives-risk-indicators/exchange-provided-metrics).
Kaiko-computed IV smile/surface publicly lists Deribit BTC, but not Bybit;
for Bybit, exchange-reported IV or an independent calculation is required. [IV smile](https://docs.kaiko.com/rest-api/analytics-solutions/kaiko-derivatives-risk-indicators/implied-volatility-calculation-smile),
[IV surface](https://docs.kaiko.com/rest-api/analytics-solutions/kaiko-derivatives-risk-indicators/implied-volatility-calculation-surface).

Kaiko offers trades, BBO, 30-second snapshots, and L2 products, but its public
documentation does not establish full historical tick-L2 specifically for Bybit/Deribit
options. A stream starts with a snapshot, then deltas; a consistency problem
resets the book, and updates with the same `tsExchange` must be applied as a group.
[L2 stream semantics](https://docs.kaiko.com/stream/data-feeds/level-1-and-level-2-data/level-2-tick-level/bids-and-asks),
[cloud L2](https://docs.kaiko.com/cloud-delivery/data-feeds/level-2-tick-level/bids-and-asks).

Delivery: REST JSON, stream and daily CSV in AWS/Azure/GCP, BigQuery/Snowflake.
REST limit — 6,000 requests/key/min, stream default — 3,000 subscriptions/key/min;
continuation token is used for pagination. Dataset corrections/versioning
must be recorded in manifest. [Cloud delivery](https://docs.kaiko.com/cloud-delivery),
[REST limits](https://docs.kaiko.com/rest-api/general/getting-started/rate-limiting),
[versioning](https://docs.kaiko.com/rest-api/general/getting-started/data-versioning).

There is no public price; a quote depends on instruments, datasets, granularity,
history, and usage, and a trial is available by request. No separate individual
self-service plan was found. [Kaiko pricing and contracts](https://www.kaiko.com/about-kaiko/pricing-and-contracts).
The license is non-transferable; raw redistribution and substitute products are
prohibited, a commercial derived product requires approval, and retention after
termination is usually limited by contract. [Kaiko terms](https://www.kaiko.com/terms).

### Coin Metrics

Unlike general marketing, the public community catalog allows exact option rows
to be checked without an API key. On 2026-08-25 it contained expired
Bybit BTC-USDT options with listing/expiry/strike/call-put/margin asset/settlement,
as well as Deribit inverse options. [Market metadata
schema](https://gitbook-docs.coinmetrics.io/market-data/market-data-overview/market-metadata).

```bash
curl -sS --compressed \
  'https://community-api.coinmetrics.io/v4/catalog-all/markets?exchange=bybit&type=option'
curl -sS --compressed \
  'https://community-api.coinmetrics.io/v4/catalog-all-v2/market-trades?exchange=deribit&type=option&base=btc&format=json_stream'
```

Verified minima from the full catalog differ significantly:

| Venue/dataset | Earliest public catalog timestamp |
|---|---:|
| Deribit BTC option trades | 2016-11-29 |
| Deribit option quotes/books/IV/Greeks | 2021-09-01 |
| Bybit option contract prices/IV/Greeks | 2024-12-06 |
| Bybit option trades | 2026-03-10 |
| Bybit option quotes/books | 2026-06-26 |

The minimum flat dataset and nested order-book depths are reproduced like this:

```bash
curl -sS --compressed \
  'https://community-api.coinmetrics.io/v4/catalog-all-v2/market-trades?exchange=deribit&type=option&base=btc&format=json_stream' \
  | jq -s '[.[] | select(.min_time != null)] | min_by(.min_time)'
curl -sS --compressed \
  'https://community-api.coinmetrics.io/v4/catalog-all-v2/market-orderbooks?exchange=bybit&type=option&base=btc&format=json_stream' \
  | jq -s '[.[] as $x | $x.depths[]? | {market:$x.market,min_time,max_time} |
            select(.min_time != null)] | min_by(.min_time)'
```

Therefore, Coin Metrics is useful for chain state/IV and long Deribit trading
history, but Bybit execution history is too short for the main test.
Historical order books are snapshots, not event deltas: a full snapshot once every
hour and ±10% mid every 10 seconds; raw historic updates are not stored. [Order-book
methodology](https://gitbook-docs.coinmetrics.io/market-data/market-data-overview/market-order-book).
Quotes/books/trades are point-in-time endpoints; IV/Greeks are available raw, 1m, 1h,
1d in JSON/CSV, with exchange/database timestamps. [Timestamp and endpoint
conventions](https://docs.coinmetrics.io/resources/faqs), [market IV](https://docs.coinmetrics.io/market-data-timeseries/market-implied-volatility),
[market Greeks](https://gitbook-docs.coinmetrics.io/market-data/market-data-overview/market-greeks).

Community tier is intended for non-commercial showcase, limited to 10 requests
per 6 seconds and usually exposes only the latest 24 hours of time series; full history requires
Professional access through sales/demo, with no public numeric price. [Coin Metrics
API](https://docs.coinmetrics.io/api/v4/). Professional access for individuals and
redistribution rights must be confirmed by contract; the Community license must not
be used for a commercial live-trading product.

### Crypto Lake and Databento: tested but excluded

Crypto Lake coverage contains only 14 Bybit spot/perpetual pairs with
2023-10-27, uses `-PERP` convention and does not list Deribit at all.
An option chain has not been confirmed for either required venue. [Crypto Lake
coverage](https://crypto-lake.com/coverage/). The product itself provides Parquet
trades, BBO/20-level books, deltas, candles via Python/S3 and origin/receive
timestamps, but this does not make the unsupported venue available. [Data and
schemas](https://crypto-lake.com/data/). There is a small free sample, no trial,
T+1 delivery, book coverage is stated to be about 98% and past gaps are irreparable.
[Crypto Lake FAQ](https://crypto-lake.com/contact/). Individual plan available,
but soft transfer limit 300 GB; pricing page shows $80/month and promo
$64/month for the first six months, Companies $500/month with 3 TB/quarter. [Crypto
Lake subscription](https://crypto-lake.com/subscribe/). Redistribution of raw or
reproducibly derived data is prohibited. [Terms](https://crypto-lake.com/terms-of-service/).

Databento official venue offering lists traditional equities/futures/
options venues, including CME crypto derivatives, but not Bybit or Deribit.
Therefore, Databento's technically strong MBO/MBP/TBBO schemas are not a source
for the required CEX options. [Databento venue catalog](https://databento.com/docs/venues-and-datasets),
[public product scope](https://databento.com/). Anonymous dataset metadata
requires key and returns 401:

```bash
curl -i -sS 'https://hist.databento.com/v0/metadata.list_datasets'
# With the key: curl -u '<API_KEY>:' URL
```

Even generic pricing here does not change the verdict: $125 signup credits are published,
Standard $199/month, Plus $1,750/month annual and Unlimited $4,500/month annual,
but this money does not open venues missing from the catalog. [Databento
pricing](https://databento.com/pricing/).

## Why trade-only is not enough

Historical trades answer “at what price did someone execute?”, but not
“could our order be executed at this moment and with this size?” At daily ATM
options, trades can be rare, spreads wide, and a reserve order after an IV rise
can exceed the available bid size. A trade-only model inevitably:

- does not know bid/ask at the time of decision and chooses the impossible mid/mark fill;
- does not know the depth and impact of the second sale;
- does not distinguish between lack of liquidity and lack of recording;
- replaces continuous mark IV with rare execution IV;
- does not show how much collateral is already locked by an open order;
- does not know how to prove that the ATM contract was visible and traded at that time;
- cannot reconstruct liquidation between two trades.

For an MVP, only an explicitly labeled `trade/mark proxy` with pessimistic
spread/slippage assumptions and a separate sensitivity range is acceptable. Such
a result must not be used to decide whether to launch on a real account.

## What cannot be restored retroactively

Even after public tick data is purchased, the following will usually remain unknown:

1. your historical latency and limit order position in the queue;
2. private rejects, cancel races, and locked margin until the first proprietary live run;
3. cross-collateral haircuts, borrow state, fee tier and account-specific limits;
4. the exact historical portfolio-margin risk matrix if the vendor did not
   archive every parameter version;
5. events inside the documented vendor/exchange gap;
6. hidden/RPI liquidity, if public feed excluded it;
7. execution of a large order with market impact, which has never happened in history.

Therefore, the future live/paper mode must write private account-risk snapshots.
Bybit portfolio margin itself stress-tests spot and IV, and coefficients may change
in extreme conditions. [Bybit margin modes](https://www.bybit.com/en/help-center/article/Margin-Calculations-Under-Different-Margin-Modes).
Deribit portfolio margin also uses a worst-case stress scenario instead of simple
standard-margin formulas. [Deribit Portfolio Margin](https://support.deribit.com/hc/en-us/articles/25944756247837-Portfolio-Margin).

## Currency and contract fork

These instruments cannot be mixed in one cash-PnL formula:

- Bybit BTC options are USDT- and USDC-settled; PnL remains in the appropriate
  stablecoin. [Bybit options PnL](https://www.bybit.com/en/help-center/article/?id=000001552&language=en_US).
- Deribit inverse BTC options quoted/margined/settled in BTC; contract = 1 BTC,
  minimum 0.1, daily expiry 08:00 UTC. [Inverse specification](https://support.deribit.com/hc/en-us/articles/31424939096093-Inverse-Options).
- Deribit linear options quoted/margined/settled in USDC, not USDT. [Linear USDC
  specification](https://support.deribit.com/hc/en-us/articles/31424932728093-Linear-USDC-Options).

For the first comparable linear test, it is more logical to compare Bybit USDT/USDC
with Deribit linear USDC. Testing Deribit's core inverse liquidity requires a separate
BTC-denominated payoff, margin and FX conversion; regular Black-Scholes premium
in USD cannot be directly written as BTC account PnL.

Current fee pages also cannot be applied to the entire history without effective-date
versioning. As of the review date, Bybit non-VIP option maker/taker fees are listed
as 0.02%/0.03% with a cap, and standard Deribit option fees as 3/3 bps with a cap
of 12.5% of premium, but
both pages can change and account tiers differ. [Bybit fees](https://www.bybit.com/en/help-center/article/?id=000001544&language=en_US),
[Deribit fees](https://support.deribit.com/hc/en-us/articles/25944746248989-Fees).

## Practical procurement and inspection plan

### MVP, 2–4 weeks

1. Free backfill Deribit official trades/instruments/index/delivery for
   3–6 months; build a coarse signal/settlement prototype.
2. Download Tardis free samples on the first of the month for both venues and run
   schema/data-quality audit: chain selection, call/put coverage, timestamps,
   crossed books, gaps, expiry and settlement linkage.
3. Ask Tardis one-off quote for specific channels, BTC options only,
   daily/near-daily expiries and 1–3 months; compare cost with Options
   subscription. Do not buy the entire venue history blindly.
4. In parallel, launch a full-chain collector and private-risk recorder for
   Bybit and Deribit.
5. In MVP, divide the results: `market PnL`, `execution PnL`, `fees`, `margin
   utilization`, `unfilled/rejected`, `liquidation proximity`.

### Highly accurate test

1. Buy Tardis tick L2 + quotes + options_chain + trades for both exchanges;
   or Amberdata, if the sample audit shows better instrument lifecycle and
   chain analytics.
2. Buy BTC perp/index data from **the same clocked source** and do not combine a
   minute OKX underlying with a tick option book without an explicit latency model.
3. Require the vendor manifest to include every incident/gap, and do not trade
   during incomplete intervals.
4. Restore fee/margin parameter versions from exchange changelogs and your own
   account snapshots; for an unknown parameter, run a range rather than one
   optimistic value.
5. Validate at least 20 expiry days manually: ATM selection, BBO before order,
   fill capacity, reserve trigger, settlement, IM/MM path and fee reconciliation.

## Questions to the supplier before payment

Submit the same list to Tardis, Amberdata, CoinAPI, Kaiko and Coin Metrics:

1. Do you offer **Bybit BTC USDT-settled options** and **Deribit BTC inverse +
   linear USDC options**; what is the earliest timestamp for each family?
2. Are expired instruments, creation/expiry, tick-size changes, and
   contract multiplier/currency fields archived?
3. Is L2 raw snapshot+deltas or periodic snapshots; what are its depth,
   frequency and sequence/gap behavior?
4. Does the option chain contain exchange mark, bid/ask/mark IV, Greeks, index,
   underlying, and OI; how often is it sampled, and how is carry-forward handled?
5. Are authoritative delivery/settlement and historical fee/margin/risk
   parameters available, or must they be obtained separately?
6. Can you provide a size estimate and a 24-hour sample for two specific expired
   ATM call/put contracts without an agreement?
7. How are incidents, late corrections, duplicates, and file revisions published;
   are there checksums/version IDs?
8. What are the API/export pagination, concurrency, and daily transfer limits,
   and what does the exact backfill cost?
9. Are use by an individual/sole proprietor, local storage, cloud backup,
   developer access, and publication of aggregated PnL without redistribution allowed?

## Data acceptance criterion

Do not load a dataset into the backtester as `verified_metadata=true` until all
of the following hold:

- full symbol/day pagination without hidden top-N limit;
- all selected call/puts existed before the decision timestamp;
- snapshots+deltas give sequence-continuous non-crossed book or gap explicitly
  marked;
- exchange/local timestamps are monotonically and causally ordered with a deterministic
  tie-breaker;
- trade IDs are unique, duplicates and late records are taken into account;
- mark/index/IV currencies and units are documented;
- expiry and delivery price are related to the instrument version;
- source incident intervals are saved in manifest;
- raw file checksum and normalized partition checksum are unchanged;
- settlement/PnL is reproduced in the correct currency;
- margin state is not inferred from premium, but calculated according to versioned rules or
  taken from account-risk observation.

The historical runtime in `back-tester-2026` supports deterministic
multi-instrument L2 replay, but lacks option expiry, Greeks, multi-leg atomicity,
and margin. That runtime does not exist in `back-tester-rust`. Buying data solves
the source-data problem, but future historical-option replay would still require
a versioned option adapter and account-risk engine; raw vendor rows cannot be
losslessly squeezed into the old project's single-instrument integer-multiplier
manifest.
