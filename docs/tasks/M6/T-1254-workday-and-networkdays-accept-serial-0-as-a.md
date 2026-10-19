---
id: T-1254
title: "WORKDAY and NETWORKDAYS accept serial 0 as a start date and as a holiday (2 gen-m6 cells)"
milestone: M6
lane: functions-date
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
WORKDAY and NETWORKDAYS (and their .INTL forms, if the probe agrees) accept serial 0 as a start/end date and as a holiday, returning Excel's values instead of `#NUM!`.

## Context
T-1239's gen-m6 suite (excel-graph sidecars, 2026-10-09):
- gen-m6/63012866a9d8b7db Calc!A39 `=IFS(TRUE,XLOOKUP(NETWORKDAYS(Data!E$9,7872),Data!E8:E11,Data!A1:B5,..),..)` with E9 blank:
  Excel `#VALUE!` (XLOOKUP's lookup and return arrays differ in size), engine `#NUM!` from `NETWORKDAYS(0,7872)`. So in
  Excel `NETWORKDAYS(0,7872)` is a number.
- gen-m6/6b2aa08e3b916da5 Calc!A35 `=VLOOKUP(WORKDAY(28131,-16,(Data!B7*1)),Data!C2:D5,1,XLOOKUP(..))` with B7 blank (holiday 0):
  Excel `#VALUE!` (from the XLOOKUP range_lookup argument), engine `#NUM!` from `WORKDAY(28131,-16,0)`. So a holiday of 0 is
  accepted.
The exact values are not in eval/: probe them first (excel-probe skill): `NETWORKDAYS(0,10)`, `NETWORKDAYS(0,7872)`,
`NETWORKDAYS(10,0)`, `WORKDAY(0,5)`, `WORKDAY(10,-5)` (reaching 0 and below), `WORKDAY(28131,-16,0)`, `NETWORKDAYS(1,10,0)`
and the .INTL forms. Serial 0 is Saturday 0 January 1900 (dates.md). Code: crates/rustsheet-engine/src/functions/date_and_time.rs.
Record the result in docs/excel-semantics/dates.md (a note was added by T-1239's triage under "WEEKDAY of serial 0 and return_type 0").

More cells (T-1289's gen-m9 suite, excel-graph, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
- gen-m9/50f10c4062fb2e98 Calc!A12 `=AVERAGE((NETWORKDAYS.INTL(44900,0.2)/(Data!$B3-Data!B8)),Data!E10:E12)` (the divisor is 42-"abc", `#VALUE!`): Excel `#VALUE!`, engine `#NUM!` from `NETWORKDAYS.INTL(44900,0.2)`; so an end date of 0.2 is accepted.
- gen-m9/288b99797f58f149 Calc!A10 `=(COLUMNS(Data!B8:E10)%+WORKDAY(33393,(Data!$A10/1.005),Data!A3:D3))` (0 days, holidays "a", 0.3, 0.3, 0.5): Excel 33393.04, engine `#VALUE!` (the text holiday; `#NUM!` without it). Add `WORKDAY(33393,0,{"a",0.3})`, `WORKDAY(33393,1,A3:D3)` and `NETWORKDAYS(33393,33400,A3:D3)` to the probe: a text cell in holidays may only be skipped when days is 0.

## Acceptance criteria
- [ ] An excel-graph probe case with the cells above, plus `XLOOKUP(NA(),A1:A4,B1:B5)` (does an error lookup_value beat the size mismatch?), passes; dates.md records the rule (VERIFIED).
- [ ] gen-m6/63012866a9d8b7db A39 and gen-m6/6b2aa08e3b916da5 A35 pass in `./target/release/rustsheet-eval show <case>`,
      with no new failing gen-m6 cell and no new known failures in eval/baseline/.
- [ ] gen-m9/50f10c4062fb2e98 A12 and gen-m9/288b99797f58f149 A10 pass, with no new failing gen-m9 cell and no new known failures in eval/baseline/.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Array lifting of these functions (T-1236).
