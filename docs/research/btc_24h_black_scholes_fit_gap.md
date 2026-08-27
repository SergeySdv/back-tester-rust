# BTC 24h Black–Scholes: Rust MVP fit-gap

- Дата проверки: 2026-08-26
- Репозиторий: текущий репозиторий

## Итог

Аудит завершён, архитектурная развилка закрыта: вычислительное ядро строится на
Rust, а загрузка данных, запуск сценариев и отчётность остаются на Python.

На момент аудита в этом репозитории нет `Cargo.toml`, Rust-кода, Python-пакета,
тестов и подтверждённых минутных данных OKX BTCUSDT. Поэтому прежние выводы о
готовом C++ replay/PnL-движке и пройденных CTest/Pytest относятся к отдельному
репозиторию `back-tester-2026` и не являются статусом этого проекта.

Утверждённый implementation brief:
[`../architecture/01_btc_24h_rust_python_mvp.md`](../architecture/01_btc_24h_rust_python_mvp.md).
Канонические этапы реализации:
[`../epics/README.md`](../epics/README.md).

## Что можно переиспользовать

Из `back-tester-2026` переиспользуются проверенные архитектурные принципы, но не
runtime-зависимость и не прямое копирование C++:

- причинная обработка событий в строгом временном порядке;
- числовые типы на горячем пути;
- fail-fast валидация входных данных;
- разделение native core и Python orchestration;
- явные позиции, PnL и неизменяемые результаты;
- тестируемая воспроизводимость и отсутствие Python-вызова на каждой минуте.

Файлы `docs/source` сохраняются как референс исходного Homework 4, а не как
контракт реализации Rust MVP.

## Матрица готовности

| Область | Статус | Следующий шаг |
|---|---|---|
| Rust workspace | Нет | Создать workspace и `backtest-core` |
| Immutable epic briefs | Готово | Реализовывать последовательно EPIC-001..003 |
| Coverage tooling contract | Готово в docs | Настроить и зафиксировать версии в EPIC-001 |
| PyO3/maturin boundary | Нет | Создать отдельный binding crate |
| OKX minute loader | Нет | Зафиксировать реальную CSV/Parquet-схему |
| Black–Scholes | Нет | Реализовать call/put, expiry и тесты |
| 24h ATM lifecycle | Нет | Реализовать фиксированный strike и expiry |
| IV baseline/2x/3x | Нет | Реализовать причинный jump после входа |
| 70/30 reserve rule | Нет | Реализовать один trigger на trade |
| Simplified margin | Нет | Добавить constant margin-per-straddle ledger |
| Equity/PnL/drawdown | Нет | Добавить минутные ряды и summary |
| Python reporting | Нет | Вернуть pandas equity/trades/summary |
| Historical option data | Отложено | Рассматривать только после результата MVP |
| Live integration | Вне MVP | Отдельный этап после валидации |

## Граница модели

- Минутный perpetual задаёт путь underlying, но не исторические option bid/ask,
  ликвидность, IV surface или исполнение.
- Текущая средняя IV на старой истории создаёт сценарный расчёт, а не
  историческую оценку наблюдаемой IV.
- Обе опционные ноги ATM только при входе; strike затем фиксирован.
- Стресс `2x/3x` считается полной переоценкой Black–Scholes, а не линейной
  vega-поправкой.
- 70% и 30% — бюджеты залоченного margin, не премия и не notional.
- Первая margin-модель линейная в model USD и не заявляет соответствие формулам
  Deribit, Bybit, Binance или OKX.

## Текущий блокер данных

Для реального прогона нужен репрезентативный файл минутной истории OKX с
подтверждёнными:

- именами и типами колонок;
- единицами timestamp и timezone;
- символом и рынком;
- периодом покрытия;
- политикой пропусков и дублей.

Отсутствие файла блокирует только точный OKX loader mapping, data-quality audit
реального набора и итоговый scenario run. Оно не блокирует scaffold, pricing,
lifecycle, accounting и generic Python boundary на синтетических fixtures.
Такие fixtures валидируют реализацию модели, но сами по себе не подтверждают
торговую гипотезу.

## Что исследовать перед выводом о гипотезе

Это не блокирует `EPIC-001` и `EPIC-002`, но обязательно до содержательного
решения по результатам real-data backtest:

- получить representative OKX minute file и проверить фактическую схему,
  timestamp units, timezone, gaps и symbol;
- зафиксировать источник текущей IV, момент снимка, expiry/moneyness universe и
  правило вычисления среднего `base_iv`;
- обосновать диапазон `margin_per_straddle_usd` и прогнать sensitivity, не
  называя упрощённую величину ГО Deribit/Bybit/Binance/OKX;
- отдельно оценить влияние proxy `perpetual close` вместо spot/index/forward,
  если synthetic MVP даст привлекательный результат;
- собирать/покупать историю опционов только если этот сценарный этап оправдает
  дополнительную точность и стоимость.

## Проверка этого изменения

Изменение только документационное. Rust workspace и исполняемые тесты ещё не
существуют, поэтому build/test к этому аудиту неприменимы. Критерии будущей
сборки, тестов и handoff зафиксированы в implementation brief.
