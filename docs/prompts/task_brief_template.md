# Task brief template

The manager completes this compact brief before implementation and keeps it
stable for the full iteration. Link to canonical criteria instead of copying
large contracts when exact identifiers are sufficient.

```text
TASK BRIEF
Task: <id and title>
Work class: <PATCH|FEATURE|EPIC>
Risk: <LOW|STANDARD|HIGH_RISK> — <reason>
Route: <developer -> QA [-> focused/full reviewer]>
Iteration: <1|2|3> of 3 <or SAME-ITERATION RECHECK for finding set ...>
Base commit / branch: <sha> / <branch>
Goal: <concrete result>
User outcome: <observable value>
Non-goals: <explicit exclusions>
Acceptance criteria: <stable IDs and exact source link>
Affected contracts: <paths/sections or none>
Prerequisites/dependency order: <small DAG/table, ordered IDs, or none>
Maturity/promotion gates: <current level; achieved evidence; next gate or none>
Cross-phase/segment state: <carry/reset, equity/peak/drawdown, gaps, anchor, aggregation, comparison segments; or N/A with reason>
Blocking product/methodology decisions: <decision, owner/evidence needed, dependent work; or none>
Required reads: <scope-specific paths>
Required checks: <focused and full commands/evidence>
Evidence-relevant paths: <explicit tracked/untracked allowlist>
Relevant tool versions: <commands/tools whose versions must be recorded>
Dataset evidence: <approved dataset ID/metadata/source checksum or none>
User-owned changes/data: <paths and preservation rules or none>
Commit/push authority: <allowed action or prohibited>
Known assumptions/blockers: <list or none>
```

Risk classification must follow [`README.md`](README.md). If scope or risk
changes materially, stop the recheck and issue a new full-iteration brief.
