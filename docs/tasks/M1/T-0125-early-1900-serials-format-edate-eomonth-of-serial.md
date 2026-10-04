---
id: T-0125
title: "Early-1900 serials: format/EDATE/EOMONTH of serial 60 and DAY/MONTH/YEAR(0)"
milestone: M1
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The four known failures in probes/dates-early-1900 (A9, A10, A12, A16) pass: `EOMONTH(31,1)` and `EDATE(31,1)` (59), `TEXT(0,"m/d/yyyy")` ("1/0/1900"), `WEEKNUM(1)` (1); DAY/MONTH/YEAR(0) also match Excel.

## Context
T-0108 fixed DATE, DAY/MONTH/YEAR and number formatting for serials 1..60 but left `from_excel_date` as a weekday-faithful stand-in calendar (see docs/excel-semantics/dates.md). Oracle values are already in probes/dates-early-1900.

## Acceptance criteria
- [ ] The probe cells above leave eval/baseline/probes.txt with no regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
