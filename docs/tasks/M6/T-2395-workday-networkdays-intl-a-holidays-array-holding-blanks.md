---
id: T-2395
title: "WORKDAY/NETWORKDAYS(.INTL): a holidays array holding blanks from HSTACK of empty cells is accepted (ours #VALUE!)"
milestone: M6
lane: functions-date
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: visible
---

## Goal
WORKDAY, WORKDAY.INTL, NETWORKDAYS and NETWORKDAYS.INTL accept a holidays array whose elements came from blank cells (`HSTACK(Data!Z1:Z2,{1;2})`), as Excel does.

## Context
Gap triage of the T-2253 gen-m6 top-up sweep (excel-graph, 2026-10-22; the `array_holes` pool's `HSTACK(Data!Z1:Z2,{1;2})`, where
Data!Z is never filled). Rule: docs/excel-semantics/dates.md "WORKDAY and NETWORKDAYS holidays: blank and number-text cells", the bullet
on blanks inside an array (added by T-2253). Excel computes every cell below; ours is `#VALUE!`. A one-cell blank reference already
passes (T-2270), so the array path of `Model::holiday_serials` (crates/rustsheet-engine/src/functions/date_and_time/holidays.rs) is
the likely place: an empty element of an array value (HSTACK of a blank cell) is rejected instead of skipped or read as 0. Excel's
results here are the same whether the blanks are skipped or read as serial 0 (no holiday at serial 0 changes them), so check which by
a probe if the implementation needs it. HSTACK of a blank reference may well hold 0, not a blank, in Excel
(`HSTACK(Data!Z1:Z2,{1;2})` alone is unprobed): probe that first, since it decides whether the fix is in HSTACK or in the holidays reader.

Failing cells (`make eval-show CASE=gen-m6/<case>.case.json`), 6 in eval/baseline/gen-m6.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m6/3eef60b5dc0145e9 Calc!A73 | `=NETWORKDAYS(8812,58239,HSTACK(Data!Z1:Z2,{1;2}))` | 35306 | `#VALUE!` |
| gen-m6/449aebbde77d8ce1 Calc!A30 | `=WORKDAY(46645,26,HSTACK(Data!Z1:Z2,{1;2}))` | 46681 | `#VALUE!` |
| gen-m6/449aebbde77d8ce1 Calc!A81 | `=NETWORKDAYS(37079,51737,HSTACK(Data!Z1:Z2,{1;2}))` | 10470 | `#VALUE!` |
| gen-m6/449aebbde77d8ce1 Calc!A225 | `=NETWORKDAYS(59088,5322,HSTACK(Data!Z1:Z2,{1;2}))` | -38405 | `#VALUE!` |
| gen-m6/449aebbde77d8ce1 Calc!A306 | `=WORKDAY.INTL(47074,-9,3,HSTACK(Data!Z1:Z2,{1;2}))` | 47061 | `#VALUE!` |
| gen-m6/f93597dff1cfa5b6 Calc!A315 | `=WORKDAY(37230,Data!A7,HSTACK(Data!Z1:Z2,{1;2}))` | 37232 | `#VALUE!` |

## Acceptance criteria
- [ ] The six cells pass; `make eval-baseline` removes them with 0 regressions.
- [ ] dates.md records the rule and how the engine reads a blank array element (skipped or 0), with the evidence used.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Text or boolean elements inside a holidays array (T-2270, T-2272).
