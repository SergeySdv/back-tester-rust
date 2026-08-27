# EPIC-002: deterministic lifecycle, accounting and result buffers

- Статус: `READY_AFTER_EPIC-001`
- Версия brief: `1.0`
- Зависимость: `EPIC-001` принят
- Внешний blocker: нет; используются синтетические fixtures

## Результат

Rust core выполняет причинный сценарный backtest 24-часового synthetic short
BTC straddle, реализует 70/30 reserve rule, cash/liability/margin accounting и
возвращает все типизированные columnar buffers, определённые архитектурным
контрактом.

## In scope

- валидация minute arrays и обязательного `DatasetMetadata`;
- non-overlapping 24h lifecycle и точный same-timestamp event order;
- baseline/2x/3x IV state, один reserve attempt и полная переоценка;
- integer quantity step counts, 70/30 budgets и free-margin cap;
- cash, liability, locked/available margin, settlement, PnL и drawdown;
- incomplete tail, typed short-history/initial-capital errors и
  `capital_exhausted` terminal result;
- equity, completed-trades, executed-tranches, reserve-attempts, summary и run
  metadata buffers с каноническим порядком и validity masks;
- deterministic fixtures и million-minute benchmark harness.

## Out of scope

- PyO3/pandas conversion сверх scaffold, CSV/Parquet loader и real OKX mapping;
- bid/ask, fees, slippage, liquidation и exchange margin;
- historical option replay, IV surface, overlapping positions и live trading.

## Acceptance criteria

- `E2-01`: данные короче 1,441 точки дают `InsufficientHistory`; gaps,
  duplicates, disorder, length mismatch, cadence не 60 секунд, metadata
  timezone не UTC и invalid price отклоняются без частичного результата.
- `E2-02`: entry использует только текущий close, `K=S_entry`, expiry ровно
  через 24 elapsed hours; reserve сохраняет strike/expiry.
- `E2-03`: на общей границе сначала settlement/realized PnL/release margin,
  затем новый entry, затем одна post-event row; 2,881 точка дают ровно две
  завершённые сделки.
- `E2-04`: ровно 1,441 точка не создаёт skipped tail; неполный последующий хвост
  исключается из equity series и считается один раз.
- `E2-05`: baseline постоянен; 2x/3x shock причинен, не влияет на pre-shock rows
  и полностью переоценивает обе ноги; shock clock начинается заново от entry
  каждой сделки, а новая boundary entry снова использует base IV.
- `E2-06`: quantities хранятся как `u64` step counts; значения непосредственно
  below/on/above границы `quantity_step=0.1` не превышают budget сверх
  документированного tolerance, а overflow/non-finite products дают typed
  errors.
- `E2-07`: первый zero-sized initial entry даёт `InsufficientInitialCapital`;
  после завершённой сделки и при наличии полного следующего окна возвращается
  валидный `capital_exhausted` result.
- `E2-08`: reserve trigger включителен на 1.5x и срабатывает один раз; full,
  reduced и rejected attempts явны, а rejected attempt не создаёт tranche.
- `E2-09`: на каждой строке выполняются `equity=cash-liability` и
  `available_margin=equity-locked_margin`; entry сам по себе не меняет equity,
  margin освобождается на expiry.
- `E2-10`: PnL, return и running-peak drawdown совпадают с ручными fixtures;
  отрицательная equity допускает drawdown выше 100%.
- `E2-11`: все таблицы, dtypes, nullability, enum values, row ordering, counts и
  metadata точно соответствуют разделу 10 архитектурного документа.
- `E2-12`: повторный run побитово воспроизводит integer/result ordering и равные
  floating arrays; все quality/coverage gates проходят, benchmark на 1 млн
  минут записывает environment и результат без hard performance threshold.

## Обязательные fixtures

- 1,440, 1,441, 1,442, 2,880 и 2,881 minute points;
- flat, rising, falling и negative-equity price paths;
- shocks before/on reserve boundary, plus 2x и 3x;
- full/reduced/rejected reserve и later capital exhaustion;
- decimal-looking quantity boundaries и hand-calculated ledger path.

## Handoff в EPIC-003

Передать один Rust run entry point, typed column buffers/validity masks и
проверенные schemas. Python слой не должен повторять эту state machine.
