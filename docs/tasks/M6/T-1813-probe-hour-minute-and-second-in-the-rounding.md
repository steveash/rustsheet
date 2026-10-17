---
id: T-1813
title: "Probe HOUR, MINUTE and SECOND in the rounding window just below serial 2958466"
milestone: M6
lane: functions-date
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's result for HOUR, MINUTE and SECOND of a serial in [2958465.9999942, 2958466) is evidenced (#NUM! or 0:00:00), and the engine matches it.

## Context
T-1353 made `time_part_fn!` (functions/date_and_time.rs) reject `value >= 2958466` before rounding to the nearest second. Whether Excel rounds first (so 2958465.999999 would be `#NUM!`) is unprobed. Probe `HOUR(2958465.999999)`, `MINUTE(...)`, `SECOND(...)` (excel-probe skill) and record it in docs/excel-semantics/dates.md "HOUR, MINUTE and SECOND past 9999-12-31".

## Acceptance criteria
- [ ] A probe case covers the window with oracle values; dates.md records the rule; the engine matches or a fix task is filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
