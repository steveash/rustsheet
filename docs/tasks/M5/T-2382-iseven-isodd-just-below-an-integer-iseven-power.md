---
id: T-2382
title: "ISEVEN/ISODD just below an integer: ISEVEN(POWER(49,0.5-1E-15)) is FALSE"
milestone: M5
lane: functions-logic
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
ISEVEN and ISODD of a value just below an integer match Excel (`ISEVEN(POWER(49,0.5-1E-15))` is FALSE).

## Context
Gap triage of gen-m5 round 2 (T-2252, excel-graph oracle, 2026-10-22). Excel gives FALSE; ours TRUE. Ours computes `POWER(49,0.499999999999999)` = 6.999999999999972 and `Model::parity_of`
(functions/information/mod.rs) truncates it to 6. The 15-digit cut of INT and TRUNC gives 6.99999999999997, still 6, so either
Excel's POWER there is 7 or above (about 17 ulps from ours, unlikely: numbers.md "`^` / POWER are not correctly rounded" has
1-2 ulp misses) or ISEVEN/ISODD round their argument more coarsely before truncating. Observation: docs/excel-semantics/information.md
"ISEVEN of a value just below an integer" (added by this triage).
Needs a probe first (`excel-probe` skill, excel-graph): `POWER(49,0.5-1E-15)` and `=POWER(49,0.5-1E-15)-7` (to see Excel's
value), `ISEVEN(6.999999999999972)`, `ISEVEN(7-2^-50)`, `ISEVEN(7-1E-14)`, `ISEVEN(7-1E-13)`, `ISEVEN(7-1E-10)`, `ISEVEN(6.9999999)`,
`ISODD(7-1E-14)`, `ISEVEN(-(7-1E-14))`, `ISEVEN(1E+15-0.25)`, `ISEVEN(2.9999999999999996)`; the same through ISODD.

Failing cells (`make eval-show CASE=gen-m5/<case>.case.json`), 1 in eval/baseline/gen-m5.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/bf8903e05ec369ca Calc!A392 | `=ISEVEN(POWER((7*Data!$A$9),(0.5-Data!C$5)))` | FALSE | TRUE |

## Acceptance criteria
- [ ] A probes case with the cells above, oracle-filled (excel-graph), passes in full; information.md states the rule (or numbers.md the POWER value, if POWER is the cause, in which case move the fix to functions-math).
- [ ] This 1 known-failure line disappears from eval/baseline/gen-m5.txt (`make eval-baseline`), with no new known failure in any suite.
- [ ] `make check` green; reviewers approve.

## Out of scope
- ISEVEN/ISODD past 2^63 (T-2165, done).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-22, T-2252): every cell this task owns has the wrong type, error, text or boolean, or is a
number further than half a unit in Excel's 15th significant digit, so the task is in the campaign. Its acceptance
criteria above are already the visible-parity stage.
