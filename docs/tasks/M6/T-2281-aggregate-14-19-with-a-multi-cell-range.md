---
id: T-2281
title: "AGGREGATE 14-19 with a multi-cell range k: probe and match (EDATE over it is #VALUE!, ours #NUM!)"
milestone: M6
lane: functions-stat
status: done
priority: P3
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
AGGREGATE functions 14-19 with a multi-cell range k give Excel's result (`EDATE(AGGREGATE(18,3,Data!C3:D3,Data!C3:D10),-21)` is `#VALUE!`), with the k handling probed.

## Context
Gap triage of gen-m6 (T-2180, excel-graph oracle, 2026-10-21). Observation: docs/excel-semantics/statistics.md "AGGREGATE 18 with a multi-cell k
under EDATE" (added by this triage). Ours is `#NUM!`. Probe (excel-probe, excel-graph, about 12 cells): `ROWS(AGGREGATE(18,3,A1:A5,B1:B3))`,
`INDEX(AGGREGATE(14,6,A1:A5,{1;2}),2)`, k ranges whose first cell is text, blank or 0.5, and under EDATE. Code:
crates/rustsheet-engine/src/functions/aggregate.rs. AGGREGATE is an M5 function (catalog); filed at M6 with the gen-m6 triage.

Failing cells (`make eval-show CASE=gen-m6/<case>.case.json`), 1 in eval/baseline/gen-m6.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m6/03333cc38ec06a14 Calc!A120 | `=EDATE(AGGREGATE(18,3,Data!C3:D3,Data!C3:D10),-21)` | "#VALUE!" | "#NUM!" |

## Acceptance criteria
- [x] This 1 known-failure line disappears from eval/baseline/gen-m6.txt (`make eval-baseline`), with no new known failure in any suite.
- [x] A new probes case (excel-graph) holds the grid; statistics.md states the rule with cells.
- [x] `make check` green; reviewers approve.

## Out of scope
- AGGREGATE's other function numbers.

## Visible-parity stage
Decision 0083: every cell above is visibly off (a wrong error, a wrong type or a value off in a leading digit). The acceptance criteria above are the stage.

## Outcome
Probed (probes/aggregate-range-k-first-cell, excel-graph, 32 cells): AGGREGATE 14-19 coerce k
before counting the array's numbers (a text k over no numbers is `#VALUE!`, an error k its error),
a reference k reads its top-left cell, and a referenced error cell as k is `#VALUE!`. New
`Model::aggregate_k` replaces `top_left_k`. Eval: gen-m6 known failures 172 -> 171
(03333cc38ec06a14 Calc!A120); the new probe case passes 32/32; no regressions.
