---
id: T-1253
title: "WEEKDAY: a serial in [0,1) is a Saturday and return_type 0 is #NUM! (2 gen-m6 cells)"
milestone: M6
lane: functions-date
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
WEEKDAY of a serial in [0, 1) is 7 (a Saturday, like serial 0 in TEXT's `dddd`), and a return_type of -0.0001 (and 0 or -1, as probed) is `#NUM!`, as Excel does.

## Context
T-1239's gen-m6 suite (excel-graph sidecars, 2026-10-09):
- gen-m6/ec7b2cc2fb9f0354 Calc!A33 `=WEEKDAY(SIN(SECH(2.5)))` (the serial is 0.1623): Excel 7, engine `#NUM!`
  (`from_excel_date` rejects serial 0 in `fn_weekday`, crates/rustsheet-engine/src/functions/date_and_time.rs).
- gen-m6/5cc04d6f42b23b51 Calc!A27 `=(VLOOKUP(..,Data!C7:D9,WEEKDAY(31330,(-0.0001)),INDEX(Data!B1:E7,4))&REPLACE(NA(),..))`:
  Excel `#NUM!`, engine `#VALUE!`. The VLOOKUP lookup value (FALSE) is not in C7:C9 and range_lookup is `#REF!`, yet the cell is `#NUM!`: a
  value index is coerced before the search, so its error wins (lookup.md "VLOOKUP / HLOOKUP search before validating
  the index"); nothing else in the formula gives `#NUM!`. -0.0001 is 0 or -1 after conversion, so probe
  `WEEKDAY(31330,0)` and `WEEKDAY(31330,-1)` before relying on either. The engine's
  `WEEKDAY(31330,-0.0001)` (and `WEEKDAY(31330,0)`) is `#VALUE!`, Excel's must be `#NUM!` (like WEEKNUM's invalid types,
  dates.md "WEEKNUM return_type edge cases").
dates.md already documents serial 0 as Saturday 0 January 1900 (`TEXT(0,"dddd")` = "Saturday"). Semantics note added
by T-1239's triage in docs/excel-semantics/dates.md "WEEKDAY of serial 0 and return_type 0".

## Acceptance criteria
- [ ] gen-m6/ec7b2cc2fb9f0354 A33 and gen-m6/5cc04d6f42b23b51 A27 pass in `./target/release/rustsheet-eval show <case>`,
      with no new failing gen-m6 cell and no new known failures in eval/baseline/.
- [ ] Unit tests: `WEEKDAY(0)` = 7, `WEEKDAY(0.5,2)` = 6, `WEEKDAY(0.5,3)` = 5, `WEEKDAY(31330,-0.0001)` is `#NUM!`; `WEEKDAY(31330,0)` and `WEEKDAY(31330,-1)` as an excel-graph probe shows (excel-probe first).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Lifting WEEKDAY over ranges and arrays (T-1235).
