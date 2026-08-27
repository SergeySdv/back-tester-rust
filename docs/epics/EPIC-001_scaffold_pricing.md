# EPIC-001: workspace, pricing and coverage tooling

- Статус: `READY`
- Версия brief: `1.0`
- Зависимости: нет
- Внешний blocker: нет; файл OKX не требуется

## Результат

Создан минимальный Rust workspace с независимым `backtest-core`, PyO3/maturin
package scaffold и проверенной общей реализацией Black–Scholes для call/put и
expiry payoff. Репозиторий генерирует машинно-читаемые line/branch coverage
reports и автоматически проверяет пороги.

## In scope

- Cargo workspace, `backtest-core`, binding crate и тонкий Python package;
- typed domain errors и минимальные config/model types, нужные pricing API;
- `DatasetMetadata` и закрытый набор `baseline/stress_2x/stress_3x` с
  валидацией, без backtest loop;
- normal CDF, Black–Scholes с конечными настраиваемыми `r` и `q`, `T=0`
  payoff и единым `SECONDS_PER_YEAR`;
- Rust/Python packaging smoke test;
- `cargo-llvm-cov`, pinned nightly branch job, `pytest-cov` и единый threshold
  checker;
- unit, boundary и property-style tests, необходимые этому scope.

## Out of scope

- minute lifecycle, позиции, reserve, margin, PnL и drawdown;
- финальные result tables;
- OKX loader и предположения о его колонках;
- exchange API, historical options и live trading.

## Acceptance criteria

- `E1-01`: `cargo build --workspace` проходит; `backtest-core` не зависит от
  Python, pandas или exchange SDK.
- `E1-02`: `maturin develop` в зафиксированном project environment создаёт
  импортируемый Python package; smoke test вызывает native pricing bulk/API
  без Python callback.
- `E1-03`: call/put совпадают минимум с тремя независимыми reference cases в
  документированном tolerance; минимум один case имеет ненулевые `r` и `q`.
- `E1-04`: put-call parity, exact intrinsic payoff при `T=0`, ATM и
  near-expiry boundaries покрыты прямыми тестами.
- `E1-05`: невалидные/неfinite `S`, `K`, `T`, `sigma`, `r` и `q` возвращают
  typed error, а не panic/NaN; отрицательный `T` запрещён.
- `E1-06`: `DatasetMetadata` отклоняет пустые identity fields, interval не 60
  секунд и timezone не UTC; scenario collection отклоняет empty, duplicate и
  custom variants.
- `E1-07`: одинаковые inputs дают одинаковые outputs; pricing formula имеет
  один источник истины в Rust.
- `E1-08`: репозиторий фиксирует совместимые версии `cargo-llvm-cov`, nightly
  toolchain и `pytest-cov`; команды из архитектуры создают JSON reports.
- `E1-09`: `scripts/check_coverage.py` отклоняет отсутствующий/невалидный report
  и enforce пороги Rust lines 90%, Rust branches 85%, Python lines 85% и Python
  branches 80%; фактические reports проходят эти пороги.
- `E1-10`: format, Clippy warnings-as-errors, Rust/Python tests и documented
  coverage commands завершаются с exit code 0.

## Обязательные тесты и evidence

- named Rust tests для каждого pricing/error criterion;
- Python import/native exception smoke tests;
- тест threshold checker на pass, below-threshold и malformed/missing input;
- сохранённые в mini-report команды, test counts, проценты и версии tools;
- review отсутствия state-machine/exchange scope creep.

## Handoff в EPIC-002

Передать стабильные pricing/error/config types, `quantity_step`-совместимую
числовую политику и зелёные quality gates. Изменять pricing semantics в
`EPIC-002` без отдельного contract change нельзя.
