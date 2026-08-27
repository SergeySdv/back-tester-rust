# AGENTS.md

## Назначение проекта

Этот репозиторий предназначен для детерминированного исследования стратегии
продажи синтетического 24-часового BTC ATM straddle на минутной истории
BTCUSDT perpetual с OKX.

Цель первого этапа — проверить гипотезу по PnL, equity, максимальной просадке и
использованию margin. Это сценарный Black–Scholes backtest, а не историческое
воспроизведение рынка опционов и не система live trading.

## Обязательный порядок чтения

Перед изменением кода или документации агент обязан:

1. прочитать этот файл;
2. прочитать `docs/architecture/01_btc_24h_rust_python_mvp.md`;
3. прочитать `docs/research/btc_24h_black_scholes_fit_gap.md`;
4. прочитать [`docs/epics/README.md`](docs/epics/README.md), канонический brief
   текущего epic/feature и применимый prompt из `docs/prompts/`;
5. проверить реальное состояние кода, тестов, конфигурации и Git working tree.

`docs/source/` содержит исторические исходные требования и диаграмму. Это
справочный материал, а не актуальный контракт Rust-реализации.

## Архитектурные границы

### Rust core

Rust отвечает за:

- Black–Scholes и payoff на expiry;
- валидацию числовых входов native boundary;
- причинный minute-by-minute lifecycle;
- IV-сценарии и 70/30 reserve rule;
- позиции, cash, option liability, locked/available margin;
- equity, PnL, drawdown и детерминированный result contract.

`backtest-core` не должен зависеть от Python, pandas или exchange SDK.

### Python orchestration

Python отвечает за:

- загрузку и отображение схемы CSV/Parquet;
- подготовку contiguous arrays;
- пользовательскую конфигурацию и запуск сценария;
- один bulk-вызов Rust на backtest;
- pandas/reporting и будущую интеграцию с универсальным exchange API.

Запрещён Python callback на каждой минуте backtest.

## Неизменяемые правила модели MVP

- Underlying — минутный close BTCUSDT perpetual.
- Опцион синтетический: European call плюс put, обе позиции short.
- Expiry наступает ровно через 24 часа elapsed time.
- `K = S_entry`; ATM гарантирован только при первоначальном входе.
- Strike и expiry reserve-транша совпадают с первоначальными.
- Black–Scholes принимает конечные настраиваемые `r` и `q`; baseline использует
  явные defaults `r=0`, `q=0`, но pricing не имеет права hard-code эти нули.
- IV задаётся как annualized decimal; baseline постоянный, стресс после входа
  полностью переоценивает обе ноги при `2x` или `3x` IV.
- Публичный набор сценариев — непустое уникальное подмножество только
  `baseline`, `stress_2x`, `stress_3x` в каноническом порядке.
- Линейная vega-аппроксимация не заменяет полный Black–Scholes repricing.
- 70% и 30% — бюджеты margin, а не premium и не notional.
- Reserve-trigger срабатывает не более одного раза при
  `current_iv >= 1.5 * entry_iv`.
- Размер reserve ограничивается фактически доступным margin после изменения
  equity; уменьшение или отказ должны быть видны в результате.
- Сделки 24h не перекрываются в MVP.
- На общей суточной границе порядок один: settlement старой сделки, realized
  PnL и release margin, затем entry новой сделки, затем одна post-event row.
- `1.0` quantity — call плюс put на один BTC; внутри quantity хранится целым
  числом шагов, а не накапливаемым `f64`.
- Запрещены look-ahead, скрытое исправление данных и недетерминированные
  решения.
- Accounting ведётся в model USD и не заявляет точного соответствия margin
  конкретной биржи.

Изменение этих правил требует явного решения пользователя и обновления
архитектурного документа до изменения реализации.

## Правила проектирования и кода

### Correctness first

- Финансовая и временная корректность важнее микрооптимизаций.
- Формулы, единицы, timezone, currency и момент события должны быть явными.
- Сначала фиксируй инварианты и тестовые примеры, затем оптимизируй hot path.
- Не выдавай синтетическую option mark за реальную bid/ask, сделку или fill.

### KISS

- Выбирай самое простое решение, которое полностью выполняет acceptance
  criteria.
- Предпочитай явные типы и небольшие функции скрытой магии и сложным framework.
- Не добавляй distributed services, plugin system, database или async runtime,
  пока epic этого явно не требует.

### YAGNI

- Не реализуй будущие exchange connectors, historical option replay, IV
  surface, order book, liquidation engine или live trading заранее.
- Не добавляй configuration knobs без текущего сценария использования.
- Не создавай универсальную абстракцию ради единственной реализации.

### DRY

- Не дублируй формулы, validation rules и accounting transitions.
- Общая логика должна иметь один источник истины и отдельные тесты.
- DRY не оправдывает преждевременную абстракцию: небольшое очевидное повторение
  лучше неверной общей модели.

### Separation of concerns

- Pricing, strategy state, portfolio accounting, margin, data loading и
  reporting должны оставаться отдельными ответственностями.
- Domain core не должен импортировать orchestration или presentation layers.
- Используй dependency inversion только на реальной внешней границе.

## Rust rules

- Используй типизированные ошибки для некорректных данных; не делай `panic!`,
  `unwrap()` или `expect()` на пользовательском пути.
- Проверяй все floating-point inputs на `is_finite()` и допустимый диапазон.
- Не сравнивай вычисленные `f64` на точное равенство без математического
  обоснования; tolerance должен быть именован и протестирован.
- Избегай `unsafe`; если без него нельзя, документируй инварианты и добавляй
  узкие тесты.
- Публичные типы и ошибки должны быть стабильными и осмысленными.
- Избегай аллокаций и динамической диспетчеризации в minute loop без измеренной
  необходимости.
- Код должен проходить formatting, Clippy с warnings-as-errors и все тесты.

## Python rules

- Python-слой остаётся тонким, типизированным и ориентированным на bulk I/O.
- Не выполняй финансовую state machine второй раз в Python.
- Проверяй schema, dtype, timezone, монотонность и contiguous layout до native
  вызова; Rust повторно защищает числовую границу.
- Не подавляй native exceptions и не возвращай частичный результат как успех.
- Не добавляй dependency без доказанной необходимости.
- Публичные функции должны иметь type hints и понятные ошибки.

## Data and determinism

- Источник, symbol, interval, timezone и dataset ID должны сохраняться в
  metadata результата.
- Gaps, duplicates, out-of-order timestamps, NaN, infinity и неположительные
  цены отклоняются явно.
- Не сортируй, forward-fill и не ремонтируй вход молча.
- Время до expiry выводится из timestamps, а не только из номера строки.
- Одинаковый input и config должны давать побитово одинаковые result arrays,
  кроме заранее документированного ограничения платформы.
- Если появляется randomness, seed становится обязательной частью config и
  результата.

## Тестирование и quality gates

Процент покрытия — минимальный индикатор, а не замена проверке требований.

- Rust core: line coverage не ниже 90%, branch coverage не ниже 85%.
- Python orchestration/reporting: line coverage не ниже 85%, branch coverage не
  ниже 80%.
- Изменённый исполняемый код: line coverage не ниже 90%, если diff coverage
  поддерживается инструментами проекта.
- Для pricing, time causality, 70/30 sizing, margin ledger, settlement, PnL и
  drawdown должны существовать тесты каждого acceptance rule и граничного
  состояния независимо от итогового процента.

Минимальные проверки, когда соответствующие части проекта существуют:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
pytest
```

Coverage policy использует `cargo-llvm-cov` для Rust lines, отдельный pinned
nightly `cargo-llvm-cov --branch` job для Rust branches, `pytest-cov` с
`--cov-branch` для Python и `scripts/check_coverage.py` как единый threshold
gate. Канонические команды находятся в разделе 14.2 архитектурного документа;
tool versions и nightly фиксируются scaffold epic в репозитории. Исключения из
coverage допускаются только по reviewed path rule с обоснованием. Агент не
имеет права придумывать процент, если JSON report не был реально сгенерирован.

## Git и рабочее дерево

- До работы зафиксируй branch, base commit и исходный `git status`.
- Сохраняй несвязанные изменения пользователя.
- Не используй destructive Git-команды без прямого разрешения.
- Не коммить `.codex/`, `.idea/`, secrets, credentials, datasets и временные
  build/cache artifacts.
- Commit и push выполняются только по явному запросу пользователя или epic.
- Перед handoff проверь diff, untracked files и реальные результаты команд.

## Агентный workflow

Для epic/feature используй последовательный процесс из
[`docs/prompts/README.md`](docs/prompts/README.md):

1. master-agent выбирает канонический immutable epic brief и фиксирует base
   commit, user-owned changes и iteration context, не меняя его criteria;
2. developer реализует и пишет mini-report;
3. QA проверяет качество, тесты и coverage и пишет mini-report;
4. reviewer независимо проверяет соответствие epic и пишет findings;
5. master принимает результат или начинает следующую итерацию.

Одновременно изменять одно рабочее дерево несколькими агентами запрещено.
Максимум — три полных итерации `developer → QA → reviewer` на один epic.

## Требования к handoff

Каждый исполнитель сообщает:

- задачу и номер итерации;
- base commit и изменённые файлы;
- фактически реализованное поведение;
- связь с acceptance criteria;
- точные команды и фактические результаты проверок;
- coverage или честное `not measured`;
- допущения, риски, blockers и следующий рекомендуемый шаг.

Нельзя писать «готово», «все тесты прошли» или «соответствует epic» без
проверяемых evidence из текущего working tree.
