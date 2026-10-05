---
id: T-0020
title: "Report Excel-evidence coverage per function"
milestone: M0
lane: harness
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
We can see which functions are proven by enough Excel evidence, not only which ones pass.

## Context
2026-10-04 eval: 435 functions have checked cells, 112 of them fewer than 10, and about 60
implemented functions have none. Pass rates on 3 cells say little.

## Acceptance criteria
- [x] `make eval` report adds a section: per function, oracle/cached cells checked, source suites, and a flag below a threshold (e.g. 25 cells); plus implemented functions with zero evidence (from the `Function` enum).
- [x] status.md snapshot shows the counts; the gap-triager uses the list to file probe/generator tasks for in-scope functions under the threshold.
- [x] `make check` green; reviewers approve.

## Outcome
`make eval` report.md ends with "Excel evidence per function": per function, the cells whose
expectations come from Excel (oracle sidecar or Excel-cached; hand cases do not count), the source
suites, and a flag below `EVIDENCE_THRESHOLD` (25). It also lists implemented functions with no
Excel evidence; the list comes from the new `rustsheet::implemented_functions()` (engine
`implemented_function_names`, over the `Function` enum), reached through the harness `Engine`
trait. report.json carries `evidence` and `unevidenced`. First numbers (2026-10-05, all suites
including poi): 258 functions at ≥25 cells, 184 below, 69 implemented with none. They are in the
status.md snapshot. The gap-triager now files probe/generator tasks from this list (decision 0031).
Eval delta: none (informational section; the gate is unchanged).
