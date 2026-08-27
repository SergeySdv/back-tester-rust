# EPIC-003: Python boundary, data integration and reporting

- Статус: `PARTIALLY_READY`
- Версия brief: `1.0`
- Зависимость: `EPIC-002` принят
- Внешний blocker: representative OKX CSV/Parquet для `E3-08..E3-10`

## Результат

Python загружает и валидирует minute data, одним bulk-вызовом запускает Rust
scenarios и предоставляет стабильные pandas tables и metadata. На реальном
OKX sample документируется точный mapping и выполняется auditable
baseline/2x/3x scenario run.

## In scope

- typed Python config, dataset metadata и fixed scenario API;
- contiguous NumPy input и один PyO3 call без per-minute callback;
- перевод Rust buffers/validity masks в pandas с контрактными dtypes/order;
- CSV/Parquet loader с явным, source-backed mapping;
- сохранение run config, model ID/version и dataset identity;
- synthetic end-to-end tests и real-data data-quality/run handoff;
- минимальные comparison tables/export, необходимые для PnL/drawdown review.

## Out of scope

- дублирование pricing/state machine в Python;
- угадывание OKX schema по filename;
- download service, exchange connector, live orders и credentials;
- исторические option quotes/IV, dashboard и generic plugin framework.

## Acceptance criteria

- `E3-01`: публичный Python API принимает contiguous `int64 timestamps_ns`,
  `float64 close`, `DatasetMetadata`, config и fixed scenarios одним run call.
- `E3-02`: dtype/length/contiguity errors и native typed errors становятся
  actionable Python exceptions без partial result.
- `E3-03`: equity, trades, tranches, reserve attempts и summary DataFrames имеют
  точные column order/dtypes/nullability; nulls не представлены NaN/negative-ID
  sentinels.
- `E3-04`: scenario/table ordering и результаты повторного вызова
  детерминированы; Python не пересчитывает финансовую state machine.
- `E3-05`: synthetic end-to-end test на 2,881 points завершает две сделки и
  сверяет table counts, final equity, PnL, margin и metadata с native result.
- `E3-06`: loader отклоняет missing columns, invalid dtype/unit/timezone,
  gaps/duplicates/disorder, NaN/infinity и non-positive close; он не сортирует,
  fill и не исправляет данные молча.
- `E3-07`: каждый output маркирован как
  `synthetic Black–Scholes scenario backtest` и не заявляет историческое
  option execution или exchange-margin fidelity.
- `E3-08`: из representative OKX sample документированы exact source, symbol,
  interval, timezone, timestamp unit, price column, coverage и mapping; эти
  значения не выводятся из непроверенного filename.
- `E3-09`: approved sample проходит data-quality checks; его dataset ID и
  checksum/identity сохраняются с результатом.
- `E3-10`: baseline, 2x и 3x run на approved sample создаёт auditable PnL,
  drawdown, margin and reserve outputs, а summary reconciliation проходит.
- `E3-11`: format/lint/tests и Rust/Python coverage gates проходят; packaging
  воспроизводится из clean project environment.

## Правило внешнего blocker

Работу можно принять отдельной feature до `E3-07` на synthetic fixtures. Весь
`EPIC-003` и полный MVP нельзя принять без `E3-08..E3-10`. Отсутствие sample не
разрешает подставить предполагаемые названия колонок, timestamp units, symbol
или timezone; manager возвращает `BLOCKED_EXTERNAL` только для real-data части.

## Required handoff evidence

- exact install/build/test/coverage commands и версии environment;
- schema/dtype assertions и synthetic E2E results;
- после получения sample: mapping document, data-quality summary, dataset
  identity и reconciled scenario summaries;
- явный список ограничений scenario model перед любым решением о live stage.
