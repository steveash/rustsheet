---
id: T-1295
title: "WORKDAY.INTL steps whole weeks instead of single days"
milestone: M6
lane: functions-date
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
WORKDAY.INTL costs O(holidays + 7) per weekend mask, not O(days).

## Context
T-1235 lifts the weekend argument; each of up to 127 distinct masks runs the day-stepping loop (up to 2.9M steps with a HashSet lookup). Skip whole weeks (7 minus the weekend count workdays per week), correct for holidays in the skipped span (sorted, binary search). Code: fn_workday_intl in functions/date_and_time.rs.

## Acceptance criteria
- [x] A lifted weekend of 127 distinct masks with days = 2000000 evaluates in well under a second; results unchanged (probes/date-argument-modes, existing WORKDAY.INTL cases).
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`step_workdays` skips whole weeks (workdays per week = 7 minus weekend days), correcting for non-weekend holidays in the skipped span, and stops short of the date-range limits so the day loop still yields the same `#NUM!` edges. Unit test `whole_week_skipping` covers forward/backward, holidays and range overflow; the 2M-day case runs in well under a second. Eval delta: none (no cells changed).
