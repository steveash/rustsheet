---
id: T-1052
title: "Probe EDATE of serial 0 with months 2 and negative (day 0 of March 1900)"
milestone: M1
lane: functions-date
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's EDATE results for serial 0 shifted by 2, -1 and -12 months are recorded by a probe, and the engine matches.

## Context
T-0997 made EDATE(0,m) return one serial before the shifted month's first day, verified only for m = 0, 1, 12. EDATE(0,2) gives 60 (the fictitious 1900-02-29) in the engine; Excel may give 59. Negative months give #NUM! in the engine, unverified. Code: `fn_edate` in `functions/date_and_time.rs`; rule in docs/excel-semantics/dates.md.

## Acceptance criteria
- [ ] A probe case records Excel's value for EDATE(0,2), EDATE(0,-1) and EDATE(0,-12); the engine and dates.md agree with it.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
