---
id: T-1253
title: "WEEKDAY: a serial in [0,1) is a Saturday and return_type 0 is #NUM! (2 gen-m6 cells)"
milestone: M6
lane: functions-date
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
WEEKDAY of a serial in [0, 1) is 7 (a Saturday, like serial 0 in TEXT's `dddd`), and a return_type that truncates to 0 is `#NUM!`, as Excel does.

## Context
T-1239's gen-m6 suite (excel-graph sidecars, 2026-10-09):
- gen-m6/ec7b2cc2fb9f0354 Calc!A33 `=WEEKDAY(SIN(SECH(2.5)))` (the serial is 0.1623): Excel 7, engine `#NUM!`
  (`from_excel_date` rejects serial 0 in `fn_weekday`, crates/rustsheet-engine/src/functions/date_and_time.rs).
- gen-m6/5cc04d6f42b23b51 Calc!A27 `=(VLOOKUP(..,Data!C7:D9,WEEKDAY(31330,(-0.0001)),INDEX(Data!B1:E7,4))&REPLACE(NA(),..))`:
  Excel `#NUM!`, engine `#VALUE!`. The VLOOKUP lookup value is found, so its column index error is the result
  (lookup.md "VLOOKUP / HLOOKUP search before validating the index": arguments fail left to right); the engine's
  `WEEKDAY(31330,-0.0001)` (and `WEEKDAY(31330,0)`) is `#VALUE!`, Excel's must be `#NUM!` (like WEEKNUM's invalid types,
  dates.md "WEEKNUM return_type edge cases").
dates.md already documents serial 0 as Saturday 0 January 1900 (`TEXT(0,"dddd")` = "Saturday"). Semantics note added
by T-1239's triage in docs/excel-semantics/dates.md "WEEKDAY of serial 0 and return_type 0".

## Acceptance criteria
- [ ] gen-m6/ec7b2cc2fb9f0354 A33 and gen-m6/5cc04d6f42b23b51 A27 pass in `./target/release/rustsheet-eval show <case>`,
      with no new failing gen-m6 cell and no new known failures in eval/baseline/.
- [ ] Unit tests: `WEEKDAY(0)` = 7, `WEEKDAY(0.5,2)` = 6, `WEEKDAY(0.5,3)` = 5, `WEEKDAY(31330,0)` and `WEEKDAY(31330,-0.0001)` are `#NUM!`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Lifting WEEKDAY over ranges and arrays (T-1235).
