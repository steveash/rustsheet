---
id: T-2268
title: "DATE: a year, month or day just below an integer reads as that integer (DATE(1983,12,-1E-15) is DATE(1983,12,0))"
milestone: M6
lane: functions-date
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
DATE reads a year, month or day a hair below an integer as Excel does (`DATE(1983,12,-1E-15)` = `DATE(1983,12,0)`, a day of 0.999999995 is 1, a year of -4.2E-16 is 1900), with the window probed.

## Context
Gap triage of gen-m6 (T-2180, excel-graph oracle, 2026-10-21). Cells: docs/excel-semantics/dates.md "DATE: a year, month or day just below
an integer" (added by this triage). `fn_date` (crates/rustsheet-engine/src/functions/date_and_time.rs) floors each argument, so each cell
is a day, a month or a year early (`#NUM!` for the year). The window is not settled by the cells: a day of 0.999999995 (5E-9 below 1) is
1, which fits rounding the serial to Excel's time resolution, but a month of -1E-21 is 0, which needs a rule for the month too, while
`DATE(9999,-32767.5,1)` is `#NUM!` (month floored to -32768, dates.md "DATE floors its month and day").
Probe (excel-probe skill, excel-graph, about 40 cells): day in {1-1E-15, 1-1E-12, 1-1E-9, 1-5E-9, 1-1E-8, 1-1E-6, -1E-15, -1E-9, -0.5},
month and year in {-1E-21, -1E-15, -1E-9, 1-1E-15, 2-1E-9}, and DAY/MONTH of the results. Record the rule, then implement.
Why opus: a numeric window to fit from a probe.

Failing cells (`make eval-show CASE=gen-m6/<case>.case.json`), 5 in eval/baseline/gen-m6.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m6/03333cc38ec06a14 Calc!A267 | `=DATE(2021,6,(-Data!D1+(2*Data!A5)))` | 44348.0 | 44347.0 |
| gen-m6/07171fa3f7493f51 Calc!A311 | `=DATE(((Data!A8/Data!B11)/(1000000000000000.0^1.005)),TAN(SUM(Data!$E1)),23)` | 23.0 | "#NUM!" |
| gen-m6/216e3c05f407ca79 Calc!A173 | `=DATE(1983,12,-1e-15)` | 30650.0 | 30649.0 |
| gen-m6/9843ec1f29523dd2 Calc!A265 | `=DATE(2052,1,COS(IFS(FALSE,Data!D3,Data!E4,(-0.0001),Data!$D3,(-2.5))))` | 55519.0 | 55518.0 |
| gen-m6/b54448af3fde07ca Calc!A379 | `=DATE(2015,(1e-15*Data!E12)%,3)` | 41976.0 | 41946.0 |

## Acceptance criteria
- [ ] These 5 known-failure lines disappear from eval/baseline/gen-m6.txt (`make eval-baseline`), with no new known failure in any suite.
- [ ] A new probes case (excel-graph) holds the grid; every cell passes or is a filed task named in the Outcome.
- [ ] dates.md states the window with cells; the DATE month/day range cells (probes/date-*) still pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
- TIME (T-2267).

## Visible-parity stage
Decision 0083: every cell above is visibly off (a wrong error, a wrong type or a value off in a leading digit). The acceptance criteria above are the stage.
