---
id: T-2381
title: "AGGREGATE(15,-0.0001,ref,range k) is #VALUE!: probe the option sign and a multi-cell k"
milestone: M5
lane: functions-math
status: done
priority: P2
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
AGGREGATE matches Excel for a negative fractional `options` and for a multi-cell range `k` in functions 14-19 (`AGGREGATE(15,-0.0001,ref,B2:E12)`).

## Context
Gap triage of gen-m5 round 2 (T-2252, excel-graph oracle, 2026-10-22). `SERIESSUM(MINVERSE(0.2),-AGGREGATE(15,Data!$C12,Data!A6:C12,Data!B2:E12),2,1E+15)` with C12 = -0.0001 is `#VALUE!` in Excel;
ours is `#NUM!`, from the AGGREGATE (MINVERSE(0.2) = 5 and `SERIESSUM(5,-1,2,1E+15)` are fine). Ours truncates the option -0.0001 to
0 (valid) and takes k from B2 (blank, so SMALL's k = 0 is `#NUM!`). Two readings fit: Excel refuses an option below 0 before
truncating it (numbers.md "AGGREGATE" says both numbers truncate and 0-7 is valid; only positive fractions were probed), or a
multi-cell range k for functions 14-19 is `#VALUE!` (a range fourth argument was probed only for functions 1-13). Observation:
numbers.md "AGGREGATE", paragraph "Observation (T-2381)" (added by this triage).
Needs a probe first (`excel-probe` skill, excel-graph): `AGGREGATE(15,-0.0001,E1:E6,1)`, `(15,-0.5,E1:E6,1)`, `(15,-1,E1:E6,1)`,
`(9,-0.0001,E1:E6)`, `(-0.5+15,0,E1:E6,1)`, `(15,0,E1:E6,G1:G2)` (G1:G2 = 1, 2) in a row inside and outside G1:G2, with ROWS of it,
`(14,6,E1:E6,G1:G2)` and `(15,0,E1:E6,{1;2})` as a control. Code: functions/subtotal.rs (`fn_aggregate`).

Failing cells (`make eval-show CASE=gen-m5/<case>.case.json`), 1 in eval/baseline/gen-m5.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/c634f6ec7b14fe08 Calc!A322 | `=SERIESSUM(MINVERSE(0.2),-AGGREGATE(15,Data!$C12,Data!A6:C12,Data!B2:E12),2,1000000000000000.0)` | #VALUE! | #NUM! |

## Acceptance criteria
- [x] A probes case with the cells above, oracle-filled (excel-graph), passes in full; numbers.md "AGGREGATE" states the rule.
- [x] This 1 known-failure line disappears from eval/baseline/gen-m5.txt (`make eval-baseline`), with no new known failure in any suite.
- [x] `make check` green; reviewers approve.

## Out of scope
- AGGREGATE error-cell k under options 1-3 (T-2336).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-22, T-2252): every cell this task owns has the wrong type, error, text or boolean, or is a
number further than half a unit in Excel's 15th significant digit, so the task is in the campaign. Its acceptance
criteria above are already the visible-parity stage.

## Outcome
Probe probes/aggregate-negative-option-range-k (excel-graph, 25 cells) separates the two readings. An `options` below 0 is
`#VALUE!` before truncation: -0.0001, -0.5 and -1, with functions 9 and 15. A multi-cell range k is fine: it reads its first
cell in any row, which we already did. `aggregate_scalar` now refuses a negative option before truncating it, and
numbers.md "AGGREGATE" states the rule (replacing the observation).
Eval delta: gen-m5 17 -> 16 (c634f6ec7b14fe08 A322 fixed); the new probe passes 25/25; no regressions.
