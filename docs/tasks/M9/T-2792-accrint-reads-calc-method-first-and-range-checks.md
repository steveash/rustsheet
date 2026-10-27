---
id: T-2792
title: "ACCRINT reads calc_method first and range-checks each date as it reads it (21 gen-m9 cells)"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
ACCRINT reports the same error as Excel when several of its arguments are bad: calc_method's own error or text comes
before every other argument, each of the three dates is `#NUM!` as soon as it is read off the calendar (before a later
argument's error, text or boolean), and a boolean par is `#VALUE!`. The 21 gen-m9 cells below pass.

## Context
Gap triage of the T-2676 sweep (gen-m9 round 2, excel-graph, `make eval` on agent/research/T-2676-gen-m9-sweep-r2,
2026-10-26). Every top-level argument of each cell was evaluated with the engine (each case's Data sheet copied); the
engine's sub-expression values equal what Excel's error implies, so the difference is ACCRINT's own check order.
Rule and evidence: docs/excel-semantics/financial.md "ACCRINT check order: calc_method first, dates as read (T-2676)".

`accrint_of` (crates/rustsheet-engine/src/functions/financial/accrint.rs) reads issue, first_interest, settlement,
rate, par, frequency, basis, calc_method left to right and returns the first error, then checks the dates against the
calendar after every read. Excel instead:
1. reads calc_method (the 8th argument) first: its error is the result, and text (`"1"`, `"TRUE"`, `""`) is `#VALUE!`
   (the existing no-text rule, financial.md "VDB, DDB and ACCRINT argument types"), before any other argument;
2. then reads the other seven in order, and an issue, first_interest or settlement that does not floor to a calendar
   serial (below 0, or past 2958465 for issue and first_interest) is `#NUM!` at once, as the COUP* readers do
   (financial.md "Analysis ToolPak securities functions check frequency and dates first");
3. par refuses a boolean read from a cell (`#VALUE!`), as issue and rate already do.
Over all 240 Excel-backed ACCRINT outer cells of gen-m9 the model has no counterexample: 40 cells where calc_method is
an error or text all give calc_method's result, and 10 cells where a date is off the calendar before a later bad
argument all give `#NUM!` (one, f81bd3b6dc373e5c A115, is `#SPILL!` from an array par lifted first, as today).

Owned cells (Excel vs ours):
- gen-m9/290180acf017709c.case.json (11): Calc!A22 (#NUM! vs #VALUE!: text issue cell, calc #NUM!), A32 (#VALUE! vs
  #DIV/0!: calc "1"), A47 (#NUM! vs #N/IMPL!: first_interest -0.25), A117 (#NUM! vs #VALUE!: first_interest 2958466
  before a text rate cell), A132 (#N/A vs #REF!: calc #N/A), A162 (#NUM! vs #VALUE!: the inner
  `ACCRINT(Data!E5,..,Data!$A$11,..)` has issue -1 before a TRUE rate cell, so it is #NUM! and STEYX passes it on),
  A232 (#NUM! vs #VALUE!: issue -1), A297 (#VALUE! vs #NUM!: calc "TRUE"), A342 (#VALUE! vs #N/A: calc #VALUE!), A372
  (#NUM! vs #VALUE!: calc #NUM!), A387 (#VALUE! vs #NUM!: calc "1")
- gen-m9/932b33b1ef8b7608.case.json (3): Calc!A33 (#NUM! vs #DIV/0!: first_interest -3.05 before par's #DIV/0!), A233
  (#VALUE! vs #NUM!: par FALSE from a cell before frequency 100), A323 (#REF! vs #NUM!: calc #REF! before first_interest
  2958466)
- gen-m9/f81bd3b6dc373e5c.case.json (7): Calc!A55 (#VALUE! vs #N/A: calc #VALUE!), A70 (#VALUE! vs #NUM!: calc ""),
  A105 (#NUM! vs #N/IMPL!: calc #NUM! before the FORECAST.ETS.SEASONALITY rate), A145 (#N/A vs #NUM!: calc #N/A), A225
  (#NUM! vs #VALUE!: calc #NUM!), A285 (#VALUE! vs #N/A: calc #VALUE!), A290 (#NUM! vs #VALUE!: issue 123456789.123
  before a text frequency cell)

Examples (`make eval-show CASE=gen-m9/<id>.case.json`):
- 290180acf017709c A32 `=ACCRINT(33996,35923,39160,((Data!B5+Data!B8)-SKEW(Data!A12,Data!C8:C8)),2872,2,0,"1")`
- 932b33b1ef8b7608 A323 `=ACCRINT(34182,2958466,37446,EFFECT(Data!B12,8),4792,2,4,INDEX(Data!B6:E7,3,(Data!C12-Data!B1)))`
- 290180acf017709c A117 `=ACCRINT(34205,2958466,43831,Data!D11,1719,1,3)` (D11 = "a")

## Acceptance criteria
- [x] These 21 known-failure lines disappear from eval/baseline/gen-m9.txt with no new ones (`make eval-baseline`
      reports 0 regressions).
- [x] Unit tests in the ACCRINT tests pin one cell of each rule: calc_method's error and text first, a date off the
      calendar before a later error, a boolean par cell `#VALUE!`.
- [x] financial.md's entry says "Ours: since T-2792" and names the function that reads the arguments.
- [x] `make check` green; reviewers approve.

## Out of scope
- Typed boolean par, frequency and basis, and booleans in first_interest and settlement (unprobed; leave as they are).
- ACCRINT's values (T-2562, T-2639).

## Visible-parity stage
`semantics`: the 21 cells are visibly off (wrong error); the task is done when each has Excel's error.

## Outcome
`accrint_of` reads calc_method first (its error, or text as `#VALUE!`), then issue and first_interest with a calendar check (serial 0..=2958465) as each is read, then settlement (not range-checked as read: gen-m9/932b33b1ef8b7608 A28), and a boolean par is `#VALUE!`. gen-m9: 23 known failures fixed (the 21 owned cells plus 2), probes: 6 fixed (t2455-transpose-single-value among them stays passing), 0 regressions. A typed boolean par now also reads `#VALUE!` (cannot be told from a cell; unprobed). Rule recorded in financial.md.
