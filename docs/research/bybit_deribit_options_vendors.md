# Где взять исторические данные BTC-опционов Bybit и Deribit

Дата проверки: **2026-08-25**. Цель: причинный бэктест продажи ближайшего
примерно 24-часового ATM call + put с резервом капитала для второй продажи при
росте IV, а затем реалистичное воспроизведение исполнения, комиссий, экспирации
и биржевого margin risk.

Исследование опирается только на официальную документацию и API бирж, а также
на страницы, каталоги, API и условия самих поставщиков. Маркетинговое заявление
без проверяемого каталога помечено как неподтверждённое. Цены и каталоги могут
измениться после указанной даты.

## Короткий вывод

1. **Лучший готовый self-service источник для обеих бирж — Tardis.dev.** Он
   документирует Bybit Options с 2023-04-05 и Deribit, включая опционы, с
   2019-03-30; даёт trades, BBO, L2, snapshots и option-chain с IV/Greeks.
   Но первая найденная в его публичном каталоге именно **Bybit BTC-USDT option**
   начинается 2025-02-19: раннее Bybit coverage нельзя автоматически считать
   USDT-margined.
   Первый день каждого месяца можно скачать без ключа и проверить до покупки.
   Публичный Options-план стоит от **$350/месяц**, но у Academic/Solo/Professional
   полная четырёхлетняя история доступна только при годовой оплате; более старая
   история и разовая выборка зависят от заказа. [Покрытие
   Deribit](https://docs.tardis.dev/historical-data-details/deribit), [CSV и
   бесплатные дни](https://docs.tardis.dev/downloadable-csv-files), [текущая
   цена и условия](https://tardis.dev/).
2. **Amberdata — сильная институциональная альтернатива**, особенно если нужны
   готовые минутные option-chain/IV/Greeks и одновременно tick-level order-book
   events. Detailed coverage подтверждает Bybit options market datasets с
   2024-03-15 и разные Deribit starts с 2021-11-16; отдельная marketing page
   заявляет Deribit trades с 2021-05-21. Публичная фиксированная цена нужного пакета не показана — надо
   оформить online order или запросить quote. Monthly даёт один год истории,
   yearly — полную историю; S3 — отдельный add-on. [Deribit
   coverage](https://www.amberdata.io/deribit-market-data), [Bybit option L2
   example](https://docs.amberdata.io/http/market/options-order-book-events),
   [ordering FAQ](https://www.amberdata.io/online-market-data-ordering-faq).
3. **CoinAPI — возможный гибкий pay-as-you-go путь**, особенно для выборочных
   trades/quotes/L2 flat files. Официальный каталог показывает отдельный
   `BYBITOPT` только с 2025-10-02; Deribit options заявлены. Но публичные страницы
   не доказывают наличие исторических option-chain IV/Greeks в flat files, а
   точную глубину каждой серии нужно проверять по symbol catalog после получения
   ключа. Стоимость L2 начинается с $8/GiB за первый GiB каждого SKU в день при
   PAYG; новым пользователям обещаны $25 credits после платёжной верификации.
   [BYBITOPT catalog](https://www.coinapi.io/products/market-data-api/docs/metadata-tables/supported-exchanges/exchanges_B),
   [Flat Files](https://www.coinapi.io/products/flat-files), [pricing](https://www.coinapi.io/products/flat-files/pricing).
4. **Kaiko и Coin Metrics достойны vendor trial, но не покупки вслепую.** Kaiko
   публично показывает исторические Deribit и Bybit BTC-USDT option instruments
   и удобный минутный derivative-price file; однако точные даты L2/IV по venue и
   цена требуют quote. Coin Metrics публичный catalog подтверждает обе venue,
   instrument lifecycle, snapshots, IV и Greeks, но диапазоны разных datasets
   неодинаковы и публичная коммерческая цена отсутствует. [Kaiko derivative
   price details](https://docs.kaiko.com/cloud-delivery/data-feeds/reference-data/derivatives-price-details),
   [Coin Metrics market data](https://docs.coinmetrics.io/market-data).
5. **Официальный бесплатный backfill достаточен только для MVP на Deribit.**
   Deribit даёт trades/expired instruments с 2016 через history host, а также
   index и settlement/delivery history. Это позволяет тестировать сигналы и
   грубую оценку PnL, но не причинное исполнение по bid/ask/L2 и не непрерывную
   IV surface. Bybit public download на проверенной странице перечисляет Spot и
   Contract, но не Options; публичные REST option trades ограничены недавним
   окном. [Deribit history host specification](https://statics.deribit.com/files/DeribitInstitutionalSetupGuide.pdf),
   [Bybit historical download inventory](https://www.bybit.com/derivatives/vi-VN/history-data).
6. **Начать собственный prospective collector надо немедленно в любом
   сценарии.** Ни один backfill не восстановит ваши исторические account equity,
   locked IM/MM, доступность резерва, fee tier и конкретную версию portfolio
   margin. Эти состояния надо записывать со своего аккаунта и версионировать
   вместе с публичным рынком.

Для первого реалистичного результата я бы купил **всю BTC options chain venue**,
но только за 1–3 месяца Tardis для Bybit Options и Deribit: заранее выбрать лишь
«ATM symbols» невозможно без look-ahead/selection bias, поскольку ATM strike
меняется вместе с underlying. Одновременно я бы запустил свой collector. До
оплаты надо запросить у Tardis точную
стоимость разовой выборки и подтвердить, что выдаются именно USDT-settled Bybit
symbols и выбранное семейство Deribit. Публичная one-off форма на дату проверки
показывала $10 за venue/day для всех инструментов, $20/day за все options venues,
$2 за symbol/day и minimum order $300; checkout/quote надо перепроверить перед
оплатой. Для высокоточного многолетнего теста short-vol стратегии — Tardis
shortlist №1, Amberdata №2 после сравнительного sample audit.

## Что именно нужно стратегии

Одна минутная серия `BTCUSDT` perpetual — только underlying signal. Для каждой
исторической точки входа нужен **полный набор реально листившихся контрактов**,
чтобы причинно выбрать ближайшую будущую экспирацию и ATM strike, а не выбрать
контракт задним числом. Для call и put необходимы:

- instrument creation/expiry, strike, option type, multiplier, currencies,
  tick/quantity rules и состояние листинга;
- exchange timestamp и local receive timestamp;
- BBO с размером либо L2 snapshot + deltas с идентификаторами последовательности;
- public trades с trade ID/sequence и aggressor side;
- mark price, bid/ask/mark IV, underlying/index и желательно exchange Greeks;
- authoritative delivery price и settlement record;
- действовавшие maker/taker/delivery/liquidation fees;
- account margin mode, equity, available balance, order/position IM, MM и
  locked/occupied margin до и после каждого действия.

Последний пункт не является обычным public market data. Bybit прямо описывает,
что short-option order IM использует index, order/mark price, OTM amount, IM
factors, MM, fee и полученную premium; значит «70% капитала» нельзя заменить
70% premium или 70% notional. [Bybit option IM/MM
formula](https://www.bybit.com/en/help-center/article/Initial-Maintenance-Margin-Calculations-Options).
Deribit пересчитывает IM/MM непрерывно, а при 100% usage новый увеличивающий риск
ордер невозможен. [Deribit margin behavior](https://support.deribit.com/hc/en-us/articles/25944811089565-What-is-margin).

Модельное уточнение: две ATM legs не делают убыток линейным. У short ATM call и
short ATM put отрицательная vega, поэтому при росте IV дорожают обе проданные
ноги; linear-vega approximation работает только для малого локального shock.
При `IV × 2`, движении spot и приближении expiry меняются vega/gamma/delta, а
short straddle имеет нелинейный payoff. Резервная продажа увеличивает short-vega,
short-gamma и margin demand именно в stress state. Поэтому в отчёте должны быть
отдельно full repricing по bid/ask surface, variation PnL и пересчёт IM/MM, а не
одна линейная поправка к «второй ноге».

## A. Бесплатные официальные backfill-источники

### Deribit

Deribit — единственный из двух venue, где официальный бесплатный backfill уже
полезен для MVP:

- `history.deribit.com` хранит public trades и instruments с запуска в 2016,
  обновляется примерно через пять секунд и допускает `count=10000` с
  `include_old=true`; это задокументировано в разделе 14 официального
  [Institutional Setup Guide](https://statics.deribit.com/files/DeribitInstitutionalSetupGuide.pdf);
- historical trade содержит execution price, execution IV, contemporaneous
  mark price и index price, trade ID и per-instrument trade sequence;
- expired instruments доступны через `get_instruments(... expired=true)` и
  `get_instrument`; delivery prices и settlements имеют отдельные пагинируемые
  endpoints. [Trade API](https://docs.deribit.com/api-reference/market-data/public-get_last_trades_by_currency_and_time),
  [instruments](https://docs.deribit.com/api-reference/market-data/public-get_instruments),
  [delivery prices](https://docs.deribit.com/api-reference/market-data/public-get_delivery_prices),
  [settlements](https://docs.deribit.com/api-reference/market-data/public-get_last_settlements_by_instrument).

Воспроизводимый запрос без ключа:

```bash
curl --get 'https://history.deribit.com/api/v2/public/get_last_trades_by_currency_and_time' \
  --data-urlencode 'currency=BTC' \
  --data-urlencode 'kind=option' \
  --data-urlencode 'start_timestamp=1704067200000' \
  --data-urlencode 'end_timestamp=1704067260000' \
  --data-urlencode 'count=3' \
  --data-urlencode 'sorting=asc'
```

Пагинация должна продолжаться по стабильному `trade_id`/`trade_seq`, с
dedup на включительной границе и проверкой `has_more`; timestamp-only cursor
может пропустить сделки, если одна миллисекунда содержит больше одной страницы.

Ограничения критичны: официального historical L2/BBO/ticker/bid-IV/ask-IV/Greeks
archive не найдено. `get_mark_price_history` возвращает 5-минутный mark только
для подмножества опционов, участвующих в volatility-index calculation; пустой
ответ для остальных является missing data, а не нулём. [Mark-price history](https://docs.deribit.com/api-reference/market-data/public-get_mark_price_history).

### Bybit

Официальная download page на дату проверки перечисляет `Public Trading
History`, index/premium klines и order book только для **Spot/Contract**, без
отдельного Options inventory. Поэтому наличие futures файлов нельзя переносить
на опционы. [Bybit Historical Market Data](https://www.bybit.com/derivatives/vi-VN/history-data).

Public V5 API помогает только частично:

- instrument discovery и current ticker/book пригодны для prospective
  collection;
- `recent-trade` возвращает максимум 1000 недавних строк и не имеет исторической
  временной пагинации;
- `historical-volatility` — часовой агрегат underlying volatility, максимум
  два года и до 30 дней за запрос, **не** историческая contract-level IV surface;
- option kline endpoint отсутствует.

См. [instrument info](https://bybit-exchange.github.io/docs/v5/market/instrument),
[recent trades](https://bybit-exchange.github.io/docs/v5/market/recent-trade),
[historical volatility](https://bybit-exchange.github.io/docs/v5/market/iv) и
[tickers](https://bybit-exchange.github.io/docs/v5/market/tickers).

Account export полезен только для собственных исполнений и account history; это
не public chain/order-book archive. Текущая help page разрешает options
trade/delivery history и USDC options orders, но не восстанавливает рынок,
который видел trader. [Bybit self-export](https://www.bybit.com/en/help-center/article/How-to-Self-Export-Account-Data).

## B. Собственный prospective collector

### Что собирать

Для каждой биржи сохранять exchange-native payload до нормализации и отдельно
нормализованные таблицы:

| Поток | Bybit | Deribit | Зачем |
|---|---|---|---|
| Instruments | REST full pagination, snapshot каждые 1–5 мин | `get_instruments`, затем `get_instrument` | Листинг, expiry, multiplier, currencies, tick rules |
| L2 | `orderbook.25/100.{symbol}` snapshot+deltas | `book.{instrument}.raw/100ms` snapshot+deltas | Исполнимый spread/depth и gap detection |
| Trades | `publicTrade.BTC` | `trades.option.BTC.raw/100ms` | Реальные executions и контроль книги |
| Option state | `tickers.{symbol}` | `ticker.*` и `markprice.options.*` | bid/ask/mark IV, mark, Greeks, OI |
| Index/underlying | venue index WS/REST | `deribit_price_index.*` | ATM selection, BS inputs, settlement |
| Expiry | delivery-price REST | delivery/settlement REST | Payoff и fee reconciliation |
| Private risk | wallet/position/order/account WS + REST snapshots | account summary/positions/orders | available balance, IM/MM, locked margin |

Bybit option order book документирован как depth 25 каждые 20 ms или depth 100
каждые 100 ms, ticker — 100 ms. [Bybit order book](https://bybit-exchange.github.io/docs/v5/websocket/public/orderbook),
[Bybit ticker](https://bybit-exchange.github.io/docs/v5/websocket/public/ticker).
Deribit book stream даёт `prev_change_id`/`change_id`, что позволяет обнаружить
gap; `raw` требует авторизации. [Deribit book stream](https://docs.deribit.com/subscriptions/orderbook/bookinstrument_nameinterval).

### Минимальная инфраструктура

Для пилота не нужна распределённая система:

```text
2 venue collector processes
  -> append-only compressed raw frames (hourly/day partitions)
  -> sequence/gap validator + REST resnapshot
  -> deterministic normalizer
  -> Parquet + manifest/checksums
  -> existing native replay cache adapter
```

Практический минимум — одна always-on VM в близком регионе, отдельный резервный
process/host, локальный NVMe spool, объектное хранилище и мониторинг heartbeat,
message rate, sequence gaps, reconnects, clock offset и delayed uploads. Compute
нагрузка небольшая относительно хранения и трафика; точный storage budget нельзя
честно назвать до 7-дневного пилота полной relevant expiry band. Измерять надо
raw compressed bytes/day отдельно по channel/venue, p95 messages/s и worst-hour
во время резкого движения, затем умножать на retention + минимум 30% headroom.
Как стартовая planning-конфигурация, не benchmark: два независимых hosts по
4 vCPU/16 GB RAM с 500 GB NVMe spool каждый и versioned object storage. После
семи дней её надо уменьшить/увеличить по фактическим bytes/day, reconnect burst
и lag; один host или общий диск не дают независимого доказательства gaps.

Операционные риски:

- WS reconnect создаёт невосстановимый промежуток до нового snapshot; snapshot
  чинит будущую книгу, но не события внутри gap;
- ATM strike и следующая daily expiry меняются, поэтому подписка только на
  текущие две ноги создаёт survivorship bias;
- venue меняет schema, tick rules, symbols, fee и margin algorithms;
- exchange timestamp может совпадать у многих событий — raw receive order и
  local timestamp должны сохраняться как causal tie-breaker;
- private account stream может расходиться с public market stream; нужны
  периодические REST reconciliations без перезаписи event time;
- один collector не даёт исторических данных до даты запуска.

## C. Покупка данных

### Сравнительная матрица

| Поставщик | Bybit options | Deribit options | Нужные datasets | История | Доставка | Цена / trial | Вердикт |
|---|---|---|---|---|---|---|---|
| **Tardis.dev** | Все options с 2023-04-05; BTC-USDT найден с 2025-02-19 | Подтверждено с 2019-03-30 | trades, quotes, L2, snapshots, options_chain IV/Greeks | Каталог по каждому symbol/day, включая expired | gzip CSV, raw replay API, client libs | Options $350/$700/$1000/$3000 в мес.; one-off $10/venue/day, min $300; первый день месяца free | **Shortlist №1** |
| **Amberdata** | Options market datasets с 2024-03-15 | Dataset starts 2021-11-16…12-05; marketing trade date 2021-05-21 | trades, BBO/tickers, order-book events + 1-min snapshots, OHLCV/OI; L1 IV/Greeks 1-min | REST books 18 мес.; older via yearly S3/Snowflake/bulk | REST, WS, CSV, S3 add-on, Snowflake | public fixed price нужной пары не показана; trial 20k calls/day | **Shortlist №2** |
| **CoinAPI** | `BYBITOPT` с 2025-10-02 | Options подтверждены; exact symbol start через catalog | trades, quotes, L2 flat files; option chain current; historical IV/Greeks flat-file не доказаны | Per symbol; catalog/API key | S3-compatible CSV, REST/WS, Snowflake | PAYG; L2 $8/$4/$2 per GiB daily tiers; $25 credits after payment verification | Только после sample audit |
| **Kaiko** | BTC-USDT traded instrument с 2025-02-19 | Inverse BTC trade history с 2016; BTC-USDC instrument с 2025-08-19 | trades, BBO, reference; minute derivative-price fields; exact option L2 range не опубликован | Reference catalog сохраняет past instruments; dataset ranges запросить | REST/stream, daily CSV, AWS/Azure/GCP, Snowflake/BigQuery | Request quote/trial; individual self-service не найден | Enterprise candidate после trial |
| **Coin Metrics** | Catalog подтверждает USDT/USDC options, snapshots, IV/Greeks | Catalog подтверждает options, snapshots, IV/Greeks | trades, quote/order-book snapshots, contract prices, IV, Greeks, metadata | Per-market min/max публично queryable; datasets имеют разные ranges | REST JSON/JSON-stream, WS | Community — только недавний showcase; Professional — request demo/quote | **Shortlist №3** после continuity audit |
| **Crypto Lake** | Только ограниченный набор Bybit pairs, не option chain | Venue отсутствует в coverage | trades/books/candles для spot/futures pairs | Bybit с 2023-10-27, но не options | Python API, Parquet/S3 | Individual subscription/sample, 300 GB soft limit | Не подходит |
| **Databento** | Отсутствует в официальном venue offering | Отсутствует | CME crypto options есть, но это другой venue/product | — | API/flat files | $125 credits относятся к доступному каталогу | Не подходит |

Ниже — детали и проверяемые ограничения основных вариантов.

### Tardis.dev

Tardis сохраняет exchange WebSocket payload без модификации, ставит local
timestamp с 100 ns precision, выбирает наиболее частый доступный feed, проверяет
order-book sequences там, где exchange их даёт, публикует incidents и
переподписывается ежедневно для нового snapshot. При переподписке возможен
документированный gap примерно 300–3000 ms, поэтому даже vendor feed не является
математически совершенным. [Collection methodology](https://docs.tardis.dev/historical-data-details/overview).

Публичный metadata API на дату проверки вернул:

```text
bybit-options.availableSince = 2023-04-05T00:00:00Z
datasets.exportedUntil       = 2026-08-25T00:00:00Z
OPTIONS dataTypes            = trades, quotes, options_chain
individual option dataTypes  = trades, incremental_book_L2, quotes,
                               book_snapshot_5, book_snapshot_25,
                               options_chain
channels                     = publicTrade, orderbook.25, orderbook.100, tickers
```

`availableSince=2023-04-05` относится ко всему Bybit options venue, а не к
USDT-семейству. Фильтрация всех BTC symbols публичного каталога по суффиксу
`-USDT` дала первый `availableSince=2025-02-19`; ранние Bybit option symbols без
суффикса относятся к другой контрактной/settlement линии. Это надо проверять по
metadata каждого инструмента, а не по общей дате venue. [Bybit Options coverage
and channels](https://docs.tardis.dev/historical-data-details/bybit-options).

Воспроизведение каталога без ключа:

```bash
curl -fsSL 'https://api.tardis.dev/v1/exchanges/bybit-options' \
  -o bybit-options.metadata.json
curl -fsSL 'https://api.tardis.dev/v1/exchanges/deribit' \
  -o deribit.metadata.json
```

Free sample, проверенный 2026-08-25:

```bash
curl -fL \
  'https://datasets.tardis.dev/v1/bybit-options/options_chain/2023/05/01/BTC-2MAY23-30000-C.csv.gz' \
  -o bybit-options_chain-2023-05-01.csv.gz
curl -fL \
  'https://datasets.tardis.dev/v1/bybit-options/incremental_book_L2/2023/05/01/BTC-2MAY23-30000-C.csv.gz' \
  -o bybit-L2-2023-05-01.csv.gz
gzip -cd bybit-options_chain-2023-05-01.csv.gz | head
```

Фактически sample содержал exchange/local timestamps, strike/expiry,
bid/ask prices and amounts, bid/ask/mark IV, mark, underlying, delta/gamma/vega/
theta; L2 sample содержал snapshot flag, side, price и amount. Это намного ближе
к требуемому backtest contract, чем trade-only source. Форматы полей официально
описаны в [CSV data types](https://docs.tardis.dev/downloadable-csv-files/data-types).
Проверенные compressed sizes конкретного активного контракта
`BTC-2MAY23-30000-C` за sample day: quotes 70,617 B, options_chain 3,737,722 B,
incremental L2 138,655 B и snapshot-25 140,040 B; grouped venue trades за день
заняли 307,196 B. Это проверка доступности/schema, **не** оценка полной chain:
размеры резко зависят от числа контрактов и volatility regime.
Normalized L2, однако, не сохраняет native sequence ID; для строгого gap audit
нужен raw replay. Grouped `OPTIONS` snapshot-файл также нельзя считать
гарантированным для каждого datatype: при проверке grouped `book_snapshot_25`
вернул HTTP 400, тогда как individual-symbol files были доступны.

Для Deribit Tardis дополнительно архивирует raw `instrument.state.any`,
`estimated_expiration_price`, `deribit_price_index`, `markprice.options`, `book`
с `prev_change_id`, а ticker/quote собирает с 2019-10-01. Это даёт lifecycle и
settlement linkage, которых нет в одних normalized CSV. [Deribit channel
history](https://docs.tardis.dev/historical-data-details/deribit).

API limits на опубликованной странице: 3,000 requests/min для
Academic/Solo/Professional, 9,000 для Business, до 50 symbols в filter; A/S/P
ограничены одним active key и source IP, transfer allowance 20 TB/month против
60 TB для Business. [Tardis rate limits](https://docs.tardis.dev/api/rate-limits).

Лицензия разрешает внутренний доступ/хранение/manipulation и derived data, но
запрещает перепродажу/redistribution исходных данных, кроме отдельно оговорённой
агрегации с минимальным resolution 10 минут. Перед передачей dataset подрядчику
или публикацией результатов нужна проверка договора. [Tardis Terms, clauses
8–10](https://docs.tardis.dev/legal/terms-of-service).

### Amberdata

Deribit marketing page заявляет tick-by-tick trades с 2021-05-21, но более
детальный официальный coverage dictionary показывает разные старты datasets:
Deribit OI 2021-11-16, ticker/trades 2021-11-17, book events 2021-11-23,
snapshots 2021-12-02 и OHLCV 2021-12-05. Для Bybit options OI, snapshots,
events, ticker и trades каталог указывает 2024-03-15. Поэтому закупочную оценку
надо строить по dataset rows, а раннюю marketing date подтвердить sample-файлом.
[Exchange coverage dictionary](https://docs.amberdata.io/data-dictionary/coverage/exchange-coverage.md),
[Deribit market page](https://www.amberdata.io/deribit-market-data).

Option-book events docs показывают `exchange=bybit`, exchange timestamp,
sequence и per-price-level replacement; size zero удаляет уровень. Один REST
request ограничен часовым диапазоном. [Bybit event semantics and one-hour
limit](https://docs.amberdata.io/http/market/options-order-book-events).
Tick trades содержат side/price/volume, index, mark, underlying и execution IV;
`sequence` nullable, REST window также максимум один час и backfill идёт по
cursor. [Option trades](https://docs.amberdata.io/http/market/options-trades).
Minute snapshots берутся из venue REST, имеют venue-dependent depth и включают
underlying, statistics, OI и Greeks. [Snapshot
specification](https://docs.amberdata.io/http/market/options-order-book-snapshots).
Information endpoint поддерживает `includeInactive=true` и per-contract
availability, то есть позволяет обнаруживать expired series. [Order-book event
information](https://docs.amberdata.io/http/market/options-order-book-events-information).

`analytics/volatility/level-1-quotes` принимает `deribit|okex|bybit`, хранит
первое наблюдение каждой минуты/часа/дня, хотя ingest идёт каждые 100 ms, и
возвращает bid/ask/mark IV, Greeks, OI, index/underlying и `isCarryForward`.
Минутный запрос ограничен 60 интервалами, часовой — 24; значит массовый backfill
через REST требует полной cursor/window automation или S3. [Level-1 quotes
specification](https://docs.amberdata.io/http/analytics/derivatives/level-1-quotes).

Trial: 15 calls/s и 20k calls/day; paid on-demand: 20 calls/s и 250k/day.
Monthly subscription даёт один год lookback, yearly — full history, S3 требует
yearly + add-on. Standard license запрещает redistribution/resale/sublicensing;
расширенные права — custom sales. [API limits](https://docs.amberdata.io/http/http-api-fundamentals),
[history, S3 and license](https://www.amberdata.io/online-market-data-ordering-faq).
С 2025-05-15 REST order-book history ограничена последними 18 месяцами; более
старые books доступны через S3, Snowflake или bulk delivery. [REST retention
change](https://docs.amberdata.io/changelog/rest-access-historical-order-book-limited).
Отдельный decorated-trades catalog начинает Deribit 2021-09-01, Bybit
2024-06-01; это ещё один пример, почему earliest date надо фиксировать отдельно
для каждого продукта. [Decorated trades
coverage](https://docs.amberdata.io/data-dictionary/analytics/derivatives/decorated-trades).

Публичной numeric price нужного bundle нет: online ordering/startup/enterprise
ведут к форме или quote. [Amberdata pricing](https://www.amberdata.io/pricing).
Website terms требуют прекратить использование и уничтожить retained Data после
окончания subscription, если order form не дал иных прав; возможность держать
купленный backfill после отмены надо согласовать письменно. [Amberdata
terms](https://www.amberdata.io/terms).

Перед оплатой запросить:

1. earliest Bybit **options** date для trades, events, snapshots и tickers;
2. raw tick frequency versus 1-minute snapshots;
3. expired USDT option symbols and instrument reference history;
4. exact S3 schema/sample for both venues;
5. gap/completeness report and correction policy;
6. individual/sole-trader eligibility и итоговую цену с S3.

### CoinAPI

CoinAPI Flat Files содержит trades, quotes и full limit-order-book CSV через
S3-compatible API; с 2026-06-09 новые high-frequency partitions hourly, старые
остаются daily. `coinapi-daily-tail` хранит только предыдущий день 24 часа и не
является archive. [S3 layout/retention](https://www.coinapi.io/products/flat-files/docs/s3-api),
[dataset layout](https://www.coinapi.io/products/flat-files/docs/datasets).

Сильная сторона — две временные метки: exchange и CoinAPI receive; quote schema
явно описывает обе. [Quotes schema](https://www.coinapi.io/products/flat-files/docs/datasets/quotes).
Если source quote не содержит exchange timestamp, схема разрешает подставить
receipt timestamp, поэтому это поле нельзя безусловно трактовать как venue event
time.
Слабость для этой задачи — options REST endpoint документирует current grouped
chain, а официальные flat-file pages перечисляют trades/quotes/books, но не
доказывают исторические mark IV/Greeks. [Current options endpoint](https://www.coinapi.io/products/market-data-api/docs/rest-api/options/options/exchange_id/current/get).

Публичный metadata table показывает `BYBITOPT` с trade start 2025-10-02; это
короче Tardis и недостаточно для многолетнего Bybit test. Exact Deribit/Bybit
contract coverage и file sizes надо получить через paid catalog/listing, что
само расходует credits. [Exchange metadata](https://www.coinapi.io/products/market-data-api/docs/metadata-tables/supported-exchanges/exchanges_B),
[estimation guide](https://www.coinapi.io/products/flat-files/docs/estimation-guide).

Ценообразование PAYG сбрасывает data-type tiers каждый UTC day: L2 — $8/GiB за
первый GiB, $4/GiB за следующие 9 и $2/GiB выше 10; committed plans начинаются
с $64/month. [Flat Files pricing](https://www.coinapi.io/products/flat-files/pricing),
[pricing semantics](https://www.coinapi.io/products/flat-files/docs/use-cases).
Новые пользователи получают $25 credits только после payment verification и
создания key. [Flat Files FAQ](https://www.coinapi.io/products/flat-files/faq).
Metadata хранит per-symbol `data_trade_start`, `data_quote_start` и
`data_orderbook_start`, а отдельный history endpoint сохраняет delisted symbols;
exact inventory до покупки нужно экспортировать и проверить, а не полагаться на
venue-level marketing. [CoinAPI Market Data
FAQ](https://www.coinapi.io/products/market-data-api/faq).
Standard usage разрешает internal backtesting/trading, но не redistribution
raw/normalized feed. [CoinAPI usage policy](https://www.coinapi.io/usage-policy).

### Kaiko

Kaiko public reference API без ключа подтвердил venue codes `drbt` (Deribit) и
`bbit` (Bybit V2). В отсортированном instrument catalog первая торговавшаяся
inverse BTC option Deribit — `BTC-31MAR17-700-P` с trade timestamp
2016-11-29; первая найденная Bybit BTC-USDT option —
`BTC-4APR25-84000-P-USDT` с 2025-02-19. Первая торговавшаяся Deribit linear
`BTC_USDC` option найдена лишь 2025-08-19. Это **первая сделка catalog**, а не
обещание начала BBO/L2/IV archive.

```bash
curl --compressed \
  'https://reference-data-api.kaiko.io/v1/instruments?exchange_code=drbt&class=option&base_asset=btc&trade_count_min=1&orderBy=trade_start_timestamp&order=1&limit=3'
curl --compressed \
  'https://reference-data-api.kaiko.io/v1/instruments?exchange_code=bbit&class=option&base_asset=btc&quote_asset=usdt&trade_count_min=1&orderBy=trade_start_timestamp&order=1&limit=1'
```

Reference endpoint сохраняет listing timestamp, contract size и expired option
definitions, но time filters для listings/expiries документированы только для
Deribit и page size максимум 1,000. [Kaiko derivatives contract
details](https://docs.kaiko.com/rest-api/data-feeds/reference-data/advanced-tier/derivatives-contract-details).

Самый близкий к минутной стратегии готовый файл — cloud **Derivative Price
Details**: bid/ask price и amount, bid/ask IV, mark/index/last, expiry/strike,
Greeks, underlying index, settlement price и timestamp. [Cloud schema](https://docs.kaiko.com/cloud-delivery/data-feeds/reference-data/derivatives-price-details).
Exchange-provided metrics доступны по 1m default, 1h/4h/1d, page size до 1,000
и включают OI/TTE/IV/Greeks/settlement; generic history «с июля 2020» не доказывает
тот же start для каждой venue. [Derivatives risk indicators](https://docs.kaiko.com/rest-api/analytics/derivatives-risk-indicators/exchange-provided-metrics).
Kaiko-computed IV smile/surface публично перечисляет Deribit BTC, но не Bybit;
для Bybit нужен exchange-reported IV или собственный calculation. [IV smile](https://docs.kaiko.com/rest-api/analytics-solutions/kaiko-derivatives-risk-indicators/implied-volatility-calculation-smile),
[IV surface](https://docs.kaiko.com/rest-api/analytics-solutions/kaiko-derivatives-risk-indicators/implied-volatility-calculation-surface).

Kaiko предлагает trades, BBO, 30-second snapshots и L2 products, но публичная
документация не доказывает полный historical tick-L2 именно для Bybit/Deribit
options. Stream начинается со snapshot, затем deltas; consistency problem
сбрасывает book, а updates с одинаковым `tsExchange` должны применяться группой.
[L2 stream semantics](https://docs.kaiko.com/stream/data-feeds/level-1-and-level-2-data/level-2-tick-level/bids-and-asks),
[cloud L2](https://docs.kaiko.com/cloud-delivery/data-feeds/level-2-tick-level/bids-and-asks).

Delivery: REST JSON, stream и daily CSV в AWS/Azure/GCP, BigQuery/Snowflake.
REST limit — 6,000 requests/key/min, stream default — 3,000 subscriptions/key/min;
continuation token используется для pagination. Dataset corrections/versioning
надо фиксировать в manifest. [Cloud delivery](https://docs.kaiko.com/cloud-delivery),
[REST limits](https://docs.kaiko.com/rest-api/general/getting-started/rate-limiting),
[versioning](https://docs.kaiko.com/rest-api/general/getting-started/data-versioning).

Публичной цены нет; quote зависит от instruments, datasets, granularity,
history и usage, trial — по заявке. Отдельный individual self-service тариф не
найден. [Kaiko pricing and contracts](https://www.kaiko.com/about-kaiko/pricing-and-contracts).
Лицензия non-transferable; raw redistribution и substitute product запрещены,
commercial derived product требует согласования, а retention после окончания
обычно ограничен договором. [Kaiko terms](https://www.kaiko.com/terms).

### Coin Metrics

В отличие от общего marketing, публичный community catalog здесь позволяет
проверить точные options rows без API key. На 2026-08-25 он содержал expired
Bybit BTC-USDT options с listing/expiry/strike/call-put/margin asset/settlement,
а также Deribit inverse options. [Market metadata
schema](https://gitbook-docs.coinmetrics.io/market-data/market-data-overview/market-metadata).

```bash
curl -sS --compressed \
  'https://community-api.coinmetrics.io/v4/catalog-all/markets?exchange=bybit&type=option'
curl -sS --compressed \
  'https://community-api.coinmetrics.io/v4/catalog-all-v2/market-trades?exchange=deribit&type=option&base=btc&format=json_stream'
```

Проверенные minima по full catalog существенно различаются:

| Venue/dataset | Earliest public catalog timestamp |
|---|---:|
| Deribit BTC option trades | 2016-11-29 |
| Deribit option quotes/books/IV/Greeks | 2021-09-01 |
| Bybit option contract prices/IV/Greeks | 2024-12-06 |
| Bybit option trades | 2026-03-10 |
| Bybit option quotes/books | 2026-06-26 |

Минимум плоского dataset и nested order-book depths воспроизводятся так:

```bash
curl -sS --compressed \
  'https://community-api.coinmetrics.io/v4/catalog-all-v2/market-trades?exchange=deribit&type=option&base=btc&format=json_stream' \
  | jq -s '[.[] | select(.min_time != null)] | min_by(.min_time)'
curl -sS --compressed \
  'https://community-api.coinmetrics.io/v4/catalog-all-v2/market-orderbooks?exchange=bybit&type=option&base=btc&format=json_stream' \
  | jq -s '[.[] as $x | $x.depths[]? | {market:$x.market,min_time,max_time} |
            select(.min_time != null)] | min_by(.min_time)'
```

Следовательно, Coin Metrics полезен для chain state/IV и длинной Deribit trade
history, но Bybit execution history слишком коротка для основного теста.
Historical order books — snapshots, а не event deltas: полный snapshot раз в
час и ±10% mid каждые 10 секунд; raw historic updates не хранятся. [Order-book
methodology](https://gitbook-docs.coinmetrics.io/market-data/market-data-overview/market-order-book).
Quotes/books/trades — point-in-time endpoints; IV/Greeks доступны raw, 1m, 1h,
1d в JSON/CSV, с exchange/database timestamps. [Timestamp and endpoint
conventions](https://docs.coinmetrics.io/resources/faqs), [market IV](https://docs.coinmetrics.io/market-data-timeseries/market-implied-volatility),
[market Greeks](https://gitbook-docs.coinmetrics.io/market-data/market-data-overview/market-greeks).

Community tier предназначен для non-commercial showcase, ограничен 10 requests
за 6 секунд и для time series обычно последними 24 часами; full history требует
Professional access через sales/demo, публичной numeric price нет. [Coin Metrics
API](https://docs.coinmetrics.io/api/v4/). Доступ Professional для физлица и
права redistribution надо подтверждать договором; Community license не следует
использовать для коммерческого live-trading продукта.

### Crypto Lake и Databento: проверены, но исключены

Crypto Lake coverage содержит только 14 Bybit spot/perpetual pairs с
2023-10-27, использует `-PERP` convention и вообще не перечисляет Deribit.
Options chain ни для одной нужной venue не подтверждена. [Crypto Lake
coverage](https://crypto-lake.com/coverage/). Сам продукт предоставляет Parquet
trades, BBO/20-level books, deltas, candles через Python/S3 и origin/receive
timestamps, но это не делает неподдерживаемый venue доступным. [Data and
schemas](https://crypto-lake.com/data/). Есть small free sample, trial нет,
T+1 delivery, book coverage заявлено около 98% и прошлые gaps неисправимы.
[Crypto Lake FAQ](https://crypto-lake.com/contact/). Individual plan доступен,
но soft transfer limit 300 GB; pricing page показывает $80/month и промо
$64/month первые шесть месяцев, Companies $500/month с 3 TB/quarter. [Crypto
Lake subscription](https://crypto-lake.com/subscribe/). Raw/воспроизводимо-derived redistribution
запрещена. [Terms](https://crypto-lake.com/terms-of-service/).

Databento official venue offering перечисляет традиционные equities/futures/
options venues, включая CME crypto derivatives, но не Bybit или Deribit.
Поэтому технически сильные MBO/MBP/TBBO schemas Databento не являются источником
нужных CEX options. [Databento venue catalog](https://databento.com/docs/venues-and-datasets),
[public product scope](https://databento.com/). Anonymous dataset metadata
требует key и возвращает 401:

```bash
curl -i -sS 'https://hist.databento.com/v0/metadata.list_datasets'
# С ключом: curl -u '<API_KEY>:' URL
```

Даже generic pricing здесь не меняет verdict: опубликованы $125 signup credits,
Standard $199/month, Plus $1,750/month annual и Unlimited $4,500/month annual,
но эти деньги не открывают отсутствующие в catalog venues. [Databento
pricing](https://databento.com/pricing/).

## Почему trade-only недостаточно

Исторические сделки отвечают на вопрос «по какой цене кто-то исполнился», но не
«мог ли наш ордер исполниться в этот момент и этим размером». У daily ATM
опционов сделки могут быть редкими, spread широким, а reserve order после роста
IV — больше доступного bid size. Trade-only модель неизбежно:

- не знает bid/ask в момент решения и выбирает невозможный mid/mark fill;
- не знает depth и impact второй продажи;
- не различает отсутствие ликвидности и отсутствие записи;
- подменяет continuous mark IV редкими execution IV;
- не показывает, сколько collateral уже locked открытым ордером;
- не умеет доказать, что ATM contract был тогда виден и торгуем;
- не восстанавливает ликвидацию между двумя сделками.

Для MVP допустим только явно маркированный `trade/mark proxy` с pessimistic
spread/slippage assumptions и отдельным sensitivity range. Такой результат не
следует использовать для решения о запуске на real account.

## Что невозможно восстановить задним числом

Даже после покупки public tick data обычно останутся неизвестными:

1. ваша историческая latency и положение limit order в очереди;
2. private rejects, cancel race и locked margin до первого собственного live run;
3. cross-collateral haircuts, borrow state, fee tier и account-specific limits;
4. точная историческая portfolio-margin risk matrix, если поставщик не
   архивировал каждую версию параметров;
5. события внутри documented vendor/exchange gap;
6. hidden/RPI liquidity, если public feed её исключал;
7. исполнение большого order при market impact, которого не было в истории.

Поэтому будущий live/paper режим обязан писать private account-risk snapshots.
Bybit portfolio margin сам stress-tests spot и IV, а коэффициенты могут меняться
в extreme conditions. [Bybit margin modes](https://www.bybit.com/en/help-center/article/Margin-Calculations-Under-Different-Margin-Modes).
Deribit portfolio margin также использует worst stress scenario вместо простых
SM formulas. [Deribit Portfolio Margin](https://support.deribit.com/hc/en-us/articles/25944756247837-Portfolio-Margin).

## Валютная и контрактная развилка

Нельзя смешать инструменты в одной формуле cash PnL:

- Bybit BTC options бывают USDT- и USDC-settled; PnL остаётся в соответствующей
  stablecoin. [Bybit options PnL](https://www.bybit.com/en/help-center/article/?id=000001552&language=en_US).
- Deribit inverse BTC options quoted/margined/settled in BTC; contract = 1 BTC,
  minimum 0.1, daily expiry 08:00 UTC. [Inverse specification](https://support.deribit.com/hc/en-us/articles/31424939096093-Inverse-Options).
- Deribit linear options quoted/margined/settled in USDC, не USDT. [Linear USDC
  specification](https://support.deribit.com/hc/en-us/articles/31424932728093-Linear-USDC-Options).

Для первого сопоставимого linear test логичнее Bybit USDT/USDC against Deribit
linear USDC. Для теста основной ликвидности Deribit inverse нужен отдельный
BTC-denominated payoff, margin и FX conversion; обычный Black-Scholes premium
в USD нельзя напрямую записать как BTC account PnL.

Текущие fee pages также нельзя применять ко всей истории без effective-date
versioning. На дату проверки Bybit non-VIP options maker/taker указаны
0.02%/0.03% с cap, Deribit standard options — 3/3 bps с cap 12.5% premium, но
обе страницы изменяемы и account tiers различаются. [Bybit fees](https://www.bybit.com/en/help-center/article/?id=000001544&language=en_US),
[Deribit fees](https://support.deribit.com/hc/en-us/articles/25944746248989-Fees).

## Практический план закупки и проверки

### MVP, 2–4 недели

1. Бесплатно backfill Deribit official trades/instruments/index/delivery за
   3–6 месяцев; построить coarse signal/settlement prototype.
2. Скачать Tardis free samples первого числа месяца для обеих venue и прогнать
   schema/data-quality audit: chain selection, call/put coverage, timestamps,
   crossed books, gaps, expiry and settlement linkage.
3. Попросить Tardis one-off quote на конкретные channels, только BTC options,
   daily/near-daily expiries и 1–3 месяца; сравнить стоимость с Options
   subscription. Не покупать всю venue history вслепую.
4. Параллельно запустить собственный full-chain collector и private-risk
   recorder на Bybit и Deribit.
5. В MVP результаты разделить: `market PnL`, `execution PnL`, `fees`, `margin
   utilization`, `unfilled/rejected`, `liquidation proximity`.

### Высокоточный тест

1. Купить Tardis tick L2 + quotes + options_chain + trades для обеих бирж;
   либо Amberdata, если sample audit покажет лучшее instrument lifecycle и
   chain analytics.
2. Покупать одновременно BTC perp/index data из **того же clocked source**, а
   не соединять минутный OKX underlying с tick option book без явной latency
   модели.
3. Требовать от vendor manifest всех incidents/gaps и не торговать на неполных
   intervals.
4. Восстановить fee/margin parameter versions из биржевых changelogs и своих
   account snapshots; для неизвестного параметра запускать диапазон, а не одно
   оптимистичное значение.
5. Валидировать хотя бы 20 expiry days вручную: ATM selection, BBO before order,
   fill capacity, reserve trigger, settlement, IM/MM path and fee reconciliation.

## Вопросы поставщику до оплаты

Отправить один и тот же список Tardis, Amberdata, CoinAPI, Kaiko и Coin Metrics:

1. Есть ли **Bybit BTC USDT-settled options** и **Deribit BTC inverse + linear
   USDC options**; earliest timestamp каждого семейства?
2. Архивируются ли expired instruments, creation/expiry, tick-size changes и
   contract multiplier/currency fields?
3. L2 — это raw snapshot+deltas или периодические snapshots; какой depth,
   frequency и sequence/gap behavior?
4. Option chain содержит exchange mark, bid/ask/mark IV, Greeks, index,
   underlying, OI; как часто и как обрабатывается carry-forward?
5. Есть ли authoritative delivery/settlement и исторические fee/margin/risk
   parameters, либо это надо получать отдельно?
6. Можно ли получить size estimate и 24-hour sample для двух конкретных expired
   ATM call/put без договора?
7. Как публикуются incidents, late corrections, duplicates и file revisions;
   есть ли checksums/version IDs?
8. Каковы API/export pagination, concurrency, daily transfer limits и цена
   exact backfill?
9. Разрешены ли использование физлицом/ИП, локальное хранение, cloud backup,
   доступ разработчика и публикация агрегированного PnL без redistribution?

## Критерий приёмки данных

Не загружать dataset в backtester как `verified_metadata=true`, пока не выполнены:

- полная symbol/day pagination без скрытого top-N limit;
- все выбранные call/put существовали до decision timestamp;
- snapshots+deltas дают sequence-continuous non-crossed book или gap явно
  помечен;
- exchange/local timestamps монотонно и причинно упорядочены с deterministic
  tie-breaker;
- trade IDs уникальны, duplicates и late records учтены;
- mark/index/IV currencies and units документированы;
- expiry и delivery price связаны с instrument version;
- source incident intervals сохранены в manifest;
- raw file checksum и нормализованный partition checksum неизменны;
- settlement/PnL воспроизводится в правильной валюте;
- margin state не выводится из premium, а рассчитывается по versioned rules или
  берётся из account-risk observation.

Проектный runtime сейчас умеет deterministic multi-instrument L2 replay, но не
умеет option expiry, Greeks, multi-leg atomicity и margin. Поэтому покупка данных
решает source problem, но всё равно требует versioned options adapter и account
risk engine; raw vendor rows нельзя без потерь втиснуть в текущий
one-instrument integer-multiplier manifest.
