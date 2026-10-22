---
id: T-2280
title: "Probe IFS with a one-cell range condition holding text under ROWS (Excel 1, ours #VALUE!)"
milestone: M6
lane: functions-logic
status: done
priority: P3
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
ROWS (and the other shape readers) over IFS with a one-cell range condition give Excel's result (`ROWS(IFS(<text cell>,x)*y)` is 1), with IFS's one-cell condition rule probed.

## Context
Gap triage of gen-m6 (T-2180, excel-graph oracle, 2026-10-21). Observation: docs/excel-semantics/logic.md "ROWS over IFS with a one-cell text
condition" (added by this triage). `fn_ifs` (crates/rustsheet-engine/src/functions/logical/mod.rs) reads a one-cell range condition as a
scalar and returns `#VALUE!` for text; Excel's ROWS is 1. IF reads a one-cell range condition as a scalar (references.md, T-2079);
IFS may lift (logic.md "IFS and SWITCH are element-wise over array/range conditions"). Probe (excel-probe, excel-graph, about 12 cells):
`ROWS(IFS(A1,5))`, `ROWS(IFS(A1:A1,5))`, `ISERROR(IFS(A1,5))`, `TYPE(IFS(A1,5))`, `ROWS(IFS(A1,5)*2)` with A1 text, TRUE and FALSE, and
the IF and SWITCH twins. IFS is an M5 function (catalog); filed at M6 with the gen-m6 triage. Related: T-2189, T-1969.

Failing cells (`make eval-show CASE=gen-m6/<case>.case.json`), 1 in eval/baseline/gen-m6.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m6/870c5e38f51ce5d4 Calc!A360 | `=ROWS((IFS(Data!A$7,Data!D$10)*Data!C10))` | 1.0 | "#VALUE!" |

## Acceptance criteria
- [x] This 1 known-failure line disappears from eval/baseline/gen-m6.txt (`make eval-baseline`), with no new known failure in any suite.
- [x] A new probes case (excel-graph) holds the grid; logic.md states the rule with cells.
- [x] `make check` green; reviewers approve.

## Out of scope
- Legacy IFS arms (T-1969).

## Visible-parity stage
Decision 0083: every cell above is visibly off (a wrong error, a wrong type or a value off in a leading digit). The acceptance criteria above are the stage.

## Outcome
Probed IFS, IF and SWITCH over text, TRUE, FALSE, number, blank and error conditions, given as cells, one-cell ranges,
literals and formula results (probes/t2280-ifs-one-cell-range-condition, 80 cells, excel-graph). Excel's IFS turns a
non-boolean text condition into a 1x1 array of `#VALUE!` and reads a one-cell range condition as a scalar. `ifs_pick`
now does both. logic.md has the rule. The case also probes SWITCH over a range with a blank element, for T-2166's
lifted path: Calc!A61:A67 agree with #1454's rule, and its 4 cells pass once T-2166 is merged.
Eval: gen-m6 known failures 243 -> 242 (the task's cell); gen-m7 1341 -> 1340 (8044dce50616cfe9 A1630, an IFS text
condition inside REDUCE that T-2306 listed; noted there); probes gains 1 case with 82 scored cells, all passing (A61:A67
with T-2166 merged); regressions 0.
