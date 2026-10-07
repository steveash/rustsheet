---
id: T-0997
title: "EDATE and EOMONTH accept serial 0 (day 0 of January 1900)"
milestone: M1
lane: functions-date
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
EDATE and EOMONTH accept serial 0 and treat it as day 0 of January 1900, as Excel does.

## Context
Rule VERIFIED in docs/excel-semantics/dates.md (Early-1900 edges), evidence probes/dates-early-1900-edges (T-0182, excel-graph 2026-10-06). EDATE(0,1) = 31, EOMONTH(0,0) = 31, EOMONTH(0,1) = 59, EDATE(0,12) = 366, EDATE(0,0) = 0, EOMONTH(0.5,1) = 59, EDATE(0.5,1) = 31; EOMONTH(0,-1) stays #NUM!. The engine's `month_arithmetic_date` (formatter/dates.rs) rejects day 0 and returns #NUM!.

## Acceptance criteria
- [x] probes/dates-early-1900-edges A, B and C cells of rows 36, 37, 38, 40, 42, 43 and 44 pass (A39 and A41 keep passing); baseline shrinks.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
EDATE/EOMONTH accept serial 0 (`month_arithmetic_date(0)` is 1900-01-01; EDATE steps back one serial for day 0). Dropped the `excel_date` pre-check, which rejected 0. Eval: probes known failures 211 -> 190 (21 cells fixed, no regressions).
