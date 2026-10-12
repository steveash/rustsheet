---
id: T-1390
title: "Database functions: header matching, criteria ranges and error choice differ from Excel on generated data (gen-m9 database sample)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
The database functions match Excel on the shapes that generated workbooks produce: header matching
with non-text or blank headers, criteria ranges that overlap the database or have one row, and the
error each one returns.

## Context
A seeded sample (`rustsheet-gen --milestone M9 --count 20 --seed 17`, using a scratch catalog that
held only the database signatures from T-1086) was not committed. excel-graph accepted all 20
workbooks, and the engine passed only 673 of 800 cells. Examples (Excel, then ours):
- `DSUM(Data!D1:E8,2,Data!C3:E8)` is 1000000000000003.6, where we give 0.
- `DVARP(Data!A1:D11,4,Data!B3:C11)` is 11.9025, where we give `#DIV/0!`.
- `DAVERAGE(Data!A7:A9,1,Data!B9:B9)` is `#VALUE!` (one-row criteria), where we give 3.14159.
- `DGET(Data!A1:D10,1,Data!C1:E4)` is "?b", where we give `#NUM!`.
- `DMAX(Data!A1:A10,1,Data!C3:D8)` is 123456789.123, where we give 0.
- `DAVERAGE(Data!A10:D12,4,Data!B3:E7)` is `#DIV/0!`, where we give -1.25005.
- `DCOUNT(Data!B4:D6,0,Data!D9:D10)` is 0, where we give `#VALUE!`.
The likely root causes are how a criteria header that is a number, a boolean or blank matches a
database column, criteria rows that are entirely blank, and when a one-row criteria range is
`#VALUE!`. Write a probe with one question per cell (excel-probe), record the rules in
docs/excel-semantics/statistics.md, then fix. Split the task if it is bigger than M. T-1054 covers
DVAR/DSTDEV numerics.

## Acceptance criteria
- [ ] A probe answers each root cause, and the rules are in docs/excel-semantics/.
- [ ] The probed cells pass, and a regenerated seed-17 database sample passes at least 95% of cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Lifting (T-1389).
