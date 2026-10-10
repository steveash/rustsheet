---
id: T-1236
title: "EDATE, EOMONTH, WORKDAY, NETWORKDAYS, WEEKNUM and YEARFRAC lift over arrays but are #VALUE! for a range"
milestone: M6
lane: functions-date
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
EDATE, EOMONTH, WEEKNUM, WORKDAY, WORKDAY.INTL, NETWORKDAYS, NETWORKDAYS.INTL and YEARFRAC lift over an array argument and are `#VALUE!` for a multi-cell range argument, as Excel does.

## Context
probes/date-argument-modes (excel-graph, 2026-10-09, T-1090): these Analysis ToolPak-era functions return `#VALUE!`
when a date, month count, day count, return type or basis argument is a multi-cell range (the engine matches that),
but lift over an array literal in the same place (`INDEX(EDATE({45000.25;45031.5;45100.75},1),2)` = 45061), where the
engine gives `#N/IMPL!` (F20, F22, F24, F26, F36, F38, F40, F54, F56, F58, F60, F62, F66, F68 and their G cells).
Holidays stay read whole (dates.md "WORKDAY holidays take the whole range"); the weekend argument is T-1235's.
Semantics: docs/excel-semantics/dates.md "Date function argument modes".

## Acceptance criteria
- [ ] The listed probes/date-argument-modes cells pass, and the range forms stay `#VALUE!`; baseline updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
