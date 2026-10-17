---
id: T-1237
title: "DATEDIF reads numeric text"
milestone: M6
lane: functions-date
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
DATEDIF reads numeric text dates, as Excel does. (The DATE month/day bounds this task first covered were
probed and implemented by T-1419: probes/date-huge-month-day, dates.md "DATE month and day bounds".)

## Context
probes/date-argument-modes (excel-graph, 2026-10-09, T-1090): `INDEX(DATE(2024,A1:A3,1),2)` (month 45031.5) is
`#NUM!` in Excel, while the engine returns 1415864 (G3, G4). That date would be about year 5776, inside the 9999
limit, so the cause is not the year range: find it with the excel-probe skill before implementing (bisect the month
bound, e.g. `DATE(2024,32767,1)`, `(2024,32768,1)`, `(2024,40000,1)`, negative months, and the day equivalent
`DATE(2024,1,32768)` / `(2024,1,2958466)`). `DATEDIF("45000",46000,"d")` is 1000 in Excel, `#VALUE!` here (F89).
Semantics: docs/excel-semantics/dates.md "Date function argument modes".

More evidence (T-1239's gen-m6 suite, excel-graph, 2026-10-09): gen-m6/5cc04d6f42b23b51 Calc!A4
`=(DATE(2042,8,1E+15)%*(100-CELL("row",Data!B12)))` is 74663.6 in Excel, so `DATE(2042,8,1E+15)` = 84845 =
`DATE(2042,8,32767)`: a huge *day* is not `#NUM!` but acts like 32767 (the engine gives `#NUM!`). Make sure the month/day
bounds found by the probe keep this cell; add `DATE(2042,8,32768)`, `(2042,8,1E+6)`, `(2042,8,-1E+15)` to the probe
(dates.md "Date function argument modes").

## Acceptance criteria
- [ ] probes/date-argument-modes F89 passes; baseline updated. (G3/G4 and gen-m6/5cc04d6f42b23b51 A4 are T-1419's.)
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Already satisfied on `main`: `datedif_date_text` (date_and_time.rs) reads plain number text, and
probes/date-argument-modes F89 (`DATEDIF("45000",46000,"d")` = 1000, excel-graph) passes bit-exact
with no baseline entry (the DATE month/day bounds were T-1419's). No code change; closing the task.
