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
Early-1900 serials behave as in Excel everywhere: `TEXT` of serial 60 and `DATE(1900,1,0)` are oracle-confirmed, EDATE/EOMONTH and friends agree below serial 61, and `DAY/MONTH/YEAR(0)` return 0/1/1900.

## Context
T-0108 fixed DATE, DAY/MONTH/YEAR and number formatting for serials 1..60 but left `from_excel_date` as a weekday-faithful stand-in calendar (see docs/excel-semantics/dates.md). Needs oracle probes first (excel-probe skill) for each function.

## Acceptance criteria
- [ ] Probe case with oracle values for the functions above; engine matches them with no eval regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
