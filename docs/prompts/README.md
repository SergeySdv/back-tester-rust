# Agent prompts

This folder contains reusable instructions for sequential epic/feature
implementation by specialized agents.

## Files

- [`master_agent.md`](master_agent.md) — process manager and owner of the final
  decision;
- [`developer_agent.md`](developer_agent.md) - implementation of production code and
  required developer tests;
- [`qa_agent.md`](qa_agent.md) - independent verification, expansion of tests and
  measurement coverage;
- [`reviewer_agent.md`](reviewer_agent.md) — read-only review of epic compliance,
  architecture and implementation quality.

All roles are required to respect the root [`../../AGENTS.md`](../../AGENTS.md) and
architectural contract
[`../architecture/01_btc_24h_rust_python_mvp.md`](../architecture/01_btc_24h_rust_python_mvp.md).
The order and canonical acceptance criteria are in
[`../epics/README.md`](../epics/README.md); the selected epic's brief must not be
changed during a cycle merely to obtain `PASS`.

## One cycle of work

One iteration is a complete sequential cycle:

```text
master captures brief and iteration number
-> developer implements and returns mini-report
-> QA runs checks, adds tests and returns a mini-report
-> reviewer does an independent review and returns findings
-> master accepts the result or generates the next defect brief
```

Agents must not work concurrently in one working tree. The manager launches
the next role only after the previous role completes.

No more than three full iterations are allowed for one epic. After the third
iteration, the manager must end the process with `ACCEPTED` or
`BLOCKED_AFTER_3_ITERATIONS`; a fourth cycle is prohibited without a new user
decision.

## Conditions for successful acceptance

An epic is accepted only when all of the following conditions hold:

- developer reported `DONE` and provided a verifiable diff;
- QA reported `PASS`;
- reviewer reported `APPROVED`;
- every epic acceptance criterion has evidence;
- required commands completed successfully;
- coverage corresponds to thresholds;
- there are no unresolved `BLOCKER` and `HIGH` findings.

## Coverage policy

For a financial deterministic core, high coverage is achievable and justified:

| Area | Line coverage | Branch coverage |
|---|---:|---:|
| Rust core | at least 90% | at least 85% |
| Python orchestration/reporting | at least 85% | at least 80% |
| New/changed executable code | at least 90% | measure when diff coverage is supported |

In addition to percentages, requirement-based tests are mandatory for
Black–Scholes, expiry, causality, IV shock, 70/30 sizing, margin, PnL, and drawdown. High
coverage without checking these invariants is not considered sufficient.

Tools are fixed by architecture: `cargo-llvm-cov` for Rust line
coverage, pinned nightly `cargo-llvm-cov --branch` for Rust branch coverage,
`pytest-cov --cov-branch` for Python and `scripts/check_coverage.py` for final
threshold gate. Only generated JSON reports are considered actual.

## Mini-report contract

Each role completes its stage with a compact structured report containing the
iteration number, status, base commit, files, completed work, commands and
actual results, coverage, findings, and next handoff.

The manager includes mini-reports or exact summaries in the final report so the
history of each of the at most three iterations remains visible.
