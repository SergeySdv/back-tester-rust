# Implementation epic briefs

This folder contains the canonical MVP implementation briefs. The general
financial and public contract is in
[`../architecture/01_btc_24h_rust_python_mvp.md`](../architecture/01_btc_24h_rust_python_mvp.md).

## Execution order

1. [`EPIC-001_scaffold_pricing.md`](EPIC-001_scaffold_pricing.md) - workspace,
   pricing and coverage tooling;
2. [`EPIC-002_engine_accounting.md`](EPIC-002_engine_accounting.md) - lifecycle,
   accounting and Rust result contract;
3. [`EPIC-003_python_data_integration.md`](EPIC-003_python_data_integration.md) —
   PyO3/Python boundary, loader, reporting and real-data handoff.

Each brief is immutable within a `developer -> QA -> reviewer` cycle. The manager
may clarify the implementation method, but must not change the goal, scope, or
acceptance criteria to obtain acceptance. A contract change requires a separate
user decision, an architecture update, and a new brief version.

`EPIC-001` and `EPIC-002` were implementable on synthetic fixtures. The former
external blocker for `EPIC-003` is resolved: an approved representative OKX
history-candles CSV now provides the exact mapping, data-quality identity, and
reconciled real-run evidence required by `E3-08..E3-10`. Current workflow status
is recorded in the epic brief and validation report; evidence availability does
not replace QA/reviewer acceptance.
