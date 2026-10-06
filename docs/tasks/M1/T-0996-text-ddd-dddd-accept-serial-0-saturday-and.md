---
id: T-0996
title: "TEXT ddd/dddd accept serial 0 (Saturday) and read the Sunday-based early-1900 weekday"
milestone: M1
lane: functions-date
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
TEXT with ddd and dddd works for serial 0 and uses the Sunday-based early-1900 weekday, as Excel does.

## Context
Rule VERIFIED in docs/excel-semantics/dates.md (Early-1900 edges), evidence probes/dates-early-1900-edges (T-0182, excel-graph 2026-10-06). TEXT(0,"dddd") is Saturday (A26), TEXT(0.5,"ddd h:mm") is "Sat 12:00" (A27), TEXT(0,"ddd") is "Sat" (A30); the engine gives #VALUE!. Serial 1 is Sunday and serial 60 is Wednesday (A28, A31), so serial 0 is Saturday. TEXT(0,"d") is "0" already. See formatter/format.rs, which reads the stand-in date.

## Acceptance criteria
- [ ] probes/dates-early-1900-edges A, B and C cells of rows 26, 27, 30 and 34 pass (row 34 is TEXT(0,"dddd d mmmm yyyy") = "Saturday 0 January 1900"); baseline shrinks.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
TEXT ddd/dddd use `excel_weekday_index` (Sunday-based, serial 0 = Saturday) instead of the stand-in calendar date. probes/dates-early-1900-edges rows 26, 27, 30, 34 pass (probes known failures 226 -> 214).
