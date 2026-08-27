# Agent prompts

Эта папка содержит готовые промты для последовательной реализации epic и
feature несколькими специализированными агентами.

## Файлы

- [`master_agent.md`](master_agent.md) — менеджер процесса и владелец итогового
  решения;
- [`developer_agent.md`](developer_agent.md) — реализация production code и
  необходимых developer tests;
- [`qa_agent.md`](qa_agent.md) — независимая проверка, расширение тестов и
  измерение coverage;
- [`reviewer_agent.md`](reviewer_agent.md) — read-only review соответствия epic,
  архитектуре и качеству реализации.

Все роли обязаны соблюдать корневой [`../../AGENTS.md`](../../AGENTS.md) и
архитектурный контракт
[`../architecture/01_btc_24h_rust_python_mvp.md`](../architecture/01_btc_24h_rust_python_mvp.md).
Порядок и канонические acceptance criteria находятся в
[`../epics/README.md`](../epics/README.md); brief выбранного epic нельзя менять
в течение цикла ради получения PASS.

## Один цикл работы

Одна итерация — это полный последовательный цикл:

```text
master фиксирует brief и номер итерации
  -> developer реализует и возвращает mini-report
  -> QA запускает проверки, добавляет тесты и возвращает mini-report
  -> reviewer делает независимый review и возвращает findings
  -> master принимает результат или формирует следующий defect brief
```

Агенты не работают параллельно в одном working tree. Manager запускает
следующую роль только после завершения предыдущей.

Для одного epic разрешено не более трёх полных итераций. После третьей
итерации manager обязан завершить процесс статусом `ACCEPTED` либо
`BLOCKED_AFTER_3_ITERATIONS`; четвёртый цикл без нового решения пользователя
запрещён.

## Условия успешной приёмки

Epic принимается только одновременно при следующих условиях:

- developer сообщил `DONE` и предоставил проверяемый diff;
- QA сообщил `PASS`;
- reviewer сообщил `APPROVED`;
- все acceptance criteria epic имеют evidence;
- обязательные команды завершились успешно;
- coverage соответствует порогам;
- отсутствуют нерешённые `BLOCKER` и `HIGH` findings.

## Coverage policy

Для финансового детерминированного ядра высокое покрытие достижимо и оправдано:

| Область | Line coverage | Branch coverage |
|---|---:|---:|
| Rust core | не ниже 90% | не ниже 85% |
| Python orchestration/reporting | не ниже 85% | не ниже 80% |
| Новый/изменённый executable code | не ниже 90% | измерять при поддержке diff coverage |

Кроме процентов обязательны requirement-based tests для Black–Scholes,
expiry, causality, IV shock, 70/30 sizing, margin, PnL и drawdown. Высокий
coverage без проверки этих инвариантов не считается достаточным.

Инструменты зафиксированы архитектурой: `cargo-llvm-cov` для Rust line
coverage, pinned nightly `cargo-llvm-cov --branch` для Rust branch coverage,
`pytest-cov --cov-branch` для Python и `scripts/check_coverage.py` для итогового
threshold gate. Фактическими считаются только сгенерированные JSON reports.

## Mini-report contract

Каждая роль завершает этап компактным структурированным отчётом. В нём должны
быть номер итерации, status, base commit, файлы, выполненная работа, команды и
их фактические результаты, coverage, найденные проблемы и следующий handoff.

Manager включает mini-reports или их точную сводку в итоговый отчёт, чтобы было
видно, что произошло на каждой из максимум трёх итераций.
