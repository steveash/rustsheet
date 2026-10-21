---
id: T-2161
title: "CELL with a multi-cell range as info_type lifts over it"
milestone: M5
lane: functions-logic
status: todo
priority: P3
size: S
tier: sonnet
campaign: visible
depends_on: []
---

## Goal
CELL lifts over a multi-cell range given as `info_type`, as it already does over an array.

## Context
Gap triage of gen-m5 (T-2114, excel-graph oracle, 2026-10-20). `CELL(INDEX(D2:E9,0,1),A9)` is `#SPILL!` in Excel (an array under a filled column) and `#VALUE!` here; `CELL({"row","col"},B2)`
already spills (information.md "Information function argument modes"). Rule: docs/excel-semantics/information.md "TYPE of a
one-cell range, ISEVEN/ISODD past 2^63, CELL over a range info_type" (added by this triage). Code: `fn_cell`
(functions/information/).

Failing cells (`make eval-show CASE=gen-m5/<case>.case.json`), 1 in eval/baseline/gen-m5.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/c0651f0e0fb544d0 Calc!A293 | `=CELL(INDEX(Data!D2:E9,0,1),Data!A9)` | "#SPILL!" | "#VALUE!" |

gen-m6 triage (T-2180, excel-graph oracle, 2026-10-21): One more cell: `EOMONTH(CELL(INDEX(Data!A8:D10,0,2),Data!B$1),-9)` is `#SPILL!` in Excel (CELL lifts over the 3-cell info_type column, EOMONTH lifts over the result), ours `#VALUE!`.

Failing cells (`make eval-show CASE=gen-m6/<case>.case.json`), 1 in eval/baseline/gen-m6.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m6/3df034e67b14fccb Calc!A223 | `=EOMONTH(CELL(INDEX(Data!A8:D10,0,2),Data!B$1),-9)` | "#SPILL!" | "#VALUE!" |

## Acceptance criteria
- [ ] The 1 gen-m6 known-failure line above disappears from eval/baseline/gen-m6.txt (`make eval-baseline`), or each remaining one is named in the Outcome with its cause.
- [ ] These 1 known-failure lines disappear from eval/baseline/gen-m5.txt (`make eval-baseline`), with no new known failure in any suite.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083 (gap triage 2026-10-21): every cell this task owns has the wrong type, error, text or boolean, or is a
number further than half a unit in Excel's 15th significant digit, so the task is in the campaign. Its acceptance
criteria above are already the visible-parity stage.

## Out of scope
- CELL's reference argument as a range (already settled: no intersection).
