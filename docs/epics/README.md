# Implementation epic briefs

Эта папка содержит канонические briefs реализации MVP. Общий финансовый и
публичный контракт находится в
[`../architecture/01_btc_24h_rust_python_mvp.md`](../architecture/01_btc_24h_rust_python_mvp.md).

## Порядок выполнения

1. [`EPIC-001_scaffold_pricing.md`](EPIC-001_scaffold_pricing.md) — workspace,
   pricing и coverage tooling;
2. [`EPIC-002_engine_accounting.md`](EPIC-002_engine_accounting.md) — lifecycle,
   accounting и Rust result contract;
3. [`EPIC-003_python_data_integration.md`](EPIC-003_python_data_integration.md) —
   PyO3/Python boundary, loader, reporting и real-data handoff.

Каждый brief immutable в пределах цикла `developer -> QA -> reviewer`. Manager
может уточнить только способ реализации, но не менять цель, in/out scope или
acceptance criteria ради приёмки. Изменение контракта требует отдельного
решения пользователя, правки архитектурного документа и новой версии brief.

`EPIC-001` и `EPIC-002` готовы к реализации на синтетических fixtures.
Синтетическая часть `EPIC-003` также готова, но AC, относящиеся к точному OKX
mapping и реальному прогону, заблокированы до получения репрезентативного
CSV/Parquet-файла.
