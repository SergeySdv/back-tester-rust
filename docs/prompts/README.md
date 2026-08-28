# Agent workflow

This folder defines the scalable, sequential workflow used for repository
changes. The root [`../../AGENTS.md`](../../AGENTS.md), the assigned task brief,
and every affected canonical contract remain authoritative.

## Roles and templates

- [`task_brief_template.md`](task_brief_template.md) — compact task contract;
- [`master_agent.md`](master_agent.md) — classification, routing, and final
  decision;
- [`developer_agent.md`](developer_agent.md) — implementation and developer
  evidence;
- [`qa_agent.md`](qa_agent.md) — independent verification and test evidence;
- [`reviewer_agent.md`](reviewer_agent.md) — focused or full read-only review.

## Work classes and risk escalation

The manager records one class before work begins. Class describes breadth;
risk determines the minimum route and checks. Risk escalation always wins.

| Class | Intended scope | Default route |
|---|---|---|
| `PATCH` | Small, low-risk, non-normative maintenance or documentation correction | developer -> QA; reviewer may be omitted only under the exception below |
| `FEATURE` | Bounded user outcome, usually spanning behavior or several files | developer -> QA -> focused reviewer |
| `EPIC` | Broad, cross-layer, architectural, or canonical-contract delivery | developer -> QA -> full reviewer |

A change is `HIGH_RISK` regardless of size if it affects a financial formula,
lifecycle, accounting, margin, IV causality, data validation or repair, public
result schema, native boundary, security, dependency, CI, release, dataset
provenance, or a normative contract. `HIGH_RISK` requires independent QA,
independent reviewer, direct requirement tests where executable, and the full
applicable gates. It follows the EPIC route even when its diff is small.

A PATCH may omit reviewer only when all of these are true:

- it is low-risk and non-normative;
- it changes no executable behavior, public contract, dependency, tooling,
  release/CI path, security behavior, or dataset provenance;
- QA independently verifies the exact current tree;
- the manager records why the exception applies and owns the decision.

Uncertainty about classification escalates to FEATURE or `HIGH_RISK`; it never
justifies a lighter route.

## Task brief and status ownership

Before implementation, the manager fills
[`task_brief_template.md`](task_brief_template.md), records the base commit and
initial user-owned changes, and freezes the goal, non-goals, acceptance
criteria, affected contracts, checks, and commit authority for the iteration.

Only the manager owns durable task status such as `ACCEPTED`, `BLOCKED`, or a
decision to start another full iteration. Developer `DONE`, QA `PASS`, and
reviewer `APPROVED` are stage results, not final acceptance. Durable final
reports contain stable decisions and evidence; transient labels such as
`pending QA` or `review in progress` belong only in iteration handoffs.

## Planning and roadmap contract

A plan or roadmap must be executable, not merely chronological. It records
prerequisites and dependency order as a small DAG/table (or an explicit
`none`), and each implementation handoff confirms that the proposed sequence
respects those dependencies.

Promotion claims are evidence-gated. A general plan names its current maturity
level, achieved evidence, and the gate for the next level. For this project the
levels are distinct: synthetic scenario, economic backtest, historical option
replay, and paper/live operation. Passing a technical integration or synthetic
backtest gate does not promote work to a later level, and claims must stop at
the highest level actually evidenced.

When phases or comparison segments interact, the plan defines applicable
cross-phase state and accounting semantics: capital carry versus reset,
equity/running-peak/drawdown continuity, time gaps and no-position periods,
entry-grid anchor, aggregation/compounding, and whether comparisons use
identical segments. A non-trading plan may mark these items `not applicable`
with a reason rather than invent domain state.

Unresolved product or methodology choices are blockers before dependent work.
For this project they include, when relevant, venue/product,
settlement/collateral, contract multiplier, expiry/quote convention, and IV
source/methodology. The brief records the decision owner and required evidence;
implementation must not silently choose a value merely to advance the plan.

## Full iterations and narrow rechecks

A **full iteration** is the complete route required by class and risk:
developer implementation, independent QA, required reviewer review, and a
manager decision. At most three full iterations are allowed for one task.
After the third, the manager returns `ACCEPTED` when all acceptance gates pass,
or `BLOCKED_AFTER_3_ITERATIONS` when material findings remain unresolved. A
fourth full iteration requires explicit user authorization or a new task.

A **same-iteration recheck** is a narrow confirmation of fixes for one already
reported finding set. It is allowed only when the goal, acceptance criteria,
affected contracts, risk, and production scope are unchanged. The manager may
request at most one remediation pass and one verification recheck by each role
required by the route for that finding set. Rechecks do not reset or conceal
the full-iteration count.

Any new acceptance criterion, materially expanded production scope, new
contract surface, changed risk classification, or unrelated defect starts the
next full iteration. Rechecks must not be chained to avoid that rule.

## Shared-tree coordination

Only one role may write the shared working tree at a time. The manager waits
for the writer to finish before handing the tree to the next writer. Parallel
read-only investigation is optional when agents use the same identified
snapshot and cannot generate artifacts, acquire conflicting locks, or mutate
external state. Its findings are advisory until the active writer/role checks
them against the current tree.

## Proportional reading

Every role reads `AGENTS.md`, the task brief, its role prompt, the initial/current
Git state, and the files and tests in scope. Then:

- docs-only PATCH: read linked active documents and the contracts whose wording
  is touched;
- Python work: read Python boundary/data/reporting sections and the affected
  data contract;
- Rust-core work: read the full affected model/accounting sections and public
  Rust contracts;
- boundary, tooling, dependency, CI, release, or dataset-provenance work: read
  both sides of the boundary and all affected operational/data contracts;
- FEATURE/EPIC: also read the active documentation index, system overview,
  epics index, and canonical brief;
- `HIGH_RISK`: read every applicable financial/data contract in full.

No class may skip an applicable financial or data contract. Historical/archive
material is read only when the task or an active contract points to it.

## Validation matrix

Run focused checks first and the applicable project gates second. The brief may
add checks but may not weaken this minimum.

| Changed area | Minimum evidence |
|---|---|
| Docs-only | Markdown links and fences, trailing whitespace/`git diff --check`, applicable docs/quality guard; coverage `NOT_MEASURED` because no executable code changed |
| Python | Focused tests, full applicable `pytest`, Ruff/quality guard, Python line/branch coverage when executable code changed |
| Rust core | Focused tests, `cargo fmt`, Clippy with warnings-as-errors, workspace tests, Rust line/branch coverage when executable code changed |
| Native boundary, tooling, dependency, CI, release, or packaging | Applicable Rust and Python rows plus build/package/locked-environment and CI-command parity checks |

Direct tests of financial/data invariants are mandatory whenever those
invariants are affected. Repository coverage thresholds remain:

| Area | Line coverage | Branch coverage |
|---|---:|---:|
| Rust core | at least 90% | at least 85% |
| Python orchestration/reporting | at least 85% | at least 80% |
| New/changed executable code | at least 90% | measure when supported |

Coverage is evidence only if the configured tool generated the report against
the relevant tree. Otherwise report `NOT_MEASURED` and why. Do not manufacture
coverage for a non-executable docs-only change.

## Evidence freshness and reuse

Each brief defines an explicit allowlist of evidence-relevant paths. Each
handoff identifies the base commit, current HEAD, working-tree status, changed
files, tool versions, and the following dirty-tree identity:

```bash
git diff --binary --no-ext-diff HEAD -- <relevant-paths...> | shasum -a 256
git ls-files --others --exclude-standard -- <relevant-paths...> \
  | LC_ALL=C sort \
  | while IFS= read -r evidence_file; do
      shasum -a 256 -- "$evidence_file"
    done \
  | shasum -a 256
```

The first digest covers staged and unstaged tracked content relative to HEAD.
The second covers the contents and names of relevant, non-ignored untracked
files and deterministically hashes an empty manifest when none exist. Record
the explicit relevant-path allowlist with both digests; a digest without its
path scope is not an evidence identity.

Do not broadly hash ignored files, secrets, credentials, or unrelated
user-owned content. If such content cannot be safely represented, evidence
reuse is prohibited. Excluded/large datasets are not hashed merely to identify
the tree. When a dataset is an actual test input, record its approved dataset
ID, metadata, and existing source checksum instead.

Evidence is fresh when the command ran after all relevant edits against this
identity and its inputs, configuration, tool versions, and fixtures still
match.

Evidence may be reused only when every file capable of affecting the result is
unchanged, the originating command/result and tree identity are cited, and the
handoff labels it `REUSED`. A change to executable sources, tests, fixtures,
coverage configuration, dependency lock, compiler/interpreter, or command
invalidates the corresponding evidence, as does any change to a relevant tool
version. Coverage JSON follows the same rule; regenerate it or report
`NOT_MEASURED`.

Handoffs are delta-oriented: report changes, new evidence, invalidated/reused
evidence, open findings, and residual risk rather than reproducing an entire
epic history. The manager maintains the authoritative acceptance-criterion
trace and stable final report.

## Operating lessons

- Inspect project CLI help before the first invocation of an unfamiliar or
  version-sensitive command; record the actual supported syntax.
- Broad translation or migration work starts with a file inventory and a
  terminology map, followed by semantic spot checks against the source. A
  clean token scan alone is insufficient.
- Mermaid diagrams require semantic inspection. If no renderer is configured,
  report `NOT_RENDERED` explicitly instead of claiming visual validation.
- A real dataset passing schema, cadence, checksum, and reconciliation checks
  establishes technical integration validity. It is not evidence that a
  strategy is profitable or that synthetic options reproduce exchange fills.
