---
id: T-2264
title: "DATEDIF MD and YD: fit Excel's day counts across month ends and leap years (ours one day off)"
milestone: M6
lane: functions-date
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: visible
---

## Goal
DATEDIF "MD" and "YD" give Excel's day counts for ordinary dates: MD is the end's day minus the start's day, plus the length of the month before the end's month when negative; YD moves the end into the start's year (the next year when its month and day come before the start's) and counts from the start.

## Context
Gap triage of gen-m6 (T-2180, excel-graph oracle, 2026-10-21). Rule fitted to all 8 cells: docs/excel-semantics/dates.md "DATEDIF "MD" and "YD"
day counts" (added by this triage). Today `datedif_value` (crates/rustsheet-engine/src/functions/date_and_time.rs) computes MD by
shifting the start by whole months (`shift_months`, clamping to the month end) and YD by moving the start into the end's year, which is
one day off on each cell below. The rule is checked against every cell (2024-03-31 to 2025-10-14: 14-31+30 = 13; 1981-10-13 to
1996-07-25: 1981-10-13 to 1982-07-25 = 285). For an end of 29 February moved into a non-leap year use 1 March and say so as UNVERIFIED
in dates.md (no cell decides it). The early-1900 cells of T-2263 fit the same rule on the displayed calendar.
Sonnet: the Excel evidence is in eval and the rule fits all of it; no numeric fitting.

Failing cells (`make eval-show CASE=gen-m6/<case>.case.json`), 8 in eval/baseline/gen-m6.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m6/144bceacd894868a Calc!A356 | `=DATEDIF(29872,35271,"YD")` | 285.0 | 286.0 |
| gen-m6/22ce5678fa646215 Calc!A294 | `=DATEDIF(19510,55815,"MD")` | 22.0 | 23.0 |
| gen-m6/3516b9fbbd60269b Calc!A87 | `=DATEDIF(28064,42206,"MD")` | 20.0 | 21.0 |
| gen-m6/3516b9fbbd60269b Calc!A353 | `=DATEDIF(51118,54889,"YD")` | 119.0 | 118.0 |
| gen-m6/9c615d338d990644 Calc!A231 | `=DATEDIF(41671,51446,"YD")` | 278.0 | 279.0 |
| gen-m6/c6621490a4d67f4e Calc!A60 | `=DATEDIF(15974,43001,"YD")` | 364.0 | 363.0 |
| gen-m6/c96ac1ada46179c4 Calc!A274 | `=DATEDIF(45382,45944,"MD")` | 13.0 | 14.0 |
| gen-m6/de083028d8b676d2 Calc!A38 | `=DATEDIF(6946,30954,"YD")` | 266.0 | 267.0 |

## Acceptance criteria
- [x] These 8 known-failure lines disappear from eval/baseline/gen-m6.txt (`make eval-baseline`), with no new known failure in any suite.
- [x] Unit tests cover MD across a 31-to-30 month end, YD across a leap day and YD with the end's month-day before the start's.
- [x] dates.md's entry says Ours: implemented (T-2264).
- [x] `make check` green; reviewers approve.

## Out of scope
- Serials below 61 and serial 0 (T-2263).

## Visible-parity stage
Decision 0083: every cell above is visibly off (a wrong error, a wrong type or a value off in a leading digit). The acceptance criteria above are the stage.

## Outcome
DATEDIF "MD" and "YD" now follow the fitted rule (dates.md). All 8 listed cells pass, plus gen-m6/b54448af3fde07ca Calc!A55 (9 baseline lines removed, no new failures). Three IronCalc-origin unit tests had no oracle evidence and were updated to the rule.
