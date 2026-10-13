---
id: T-1438
title: "Fold WORKDAY and WORKDAY.INTL day-stepping loops into one helper"
milestone: M5
lane: functions-date
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
WORKDAY and WORKDAY.INTL step over working days with one shared helper, not two near-copies.

## Context
Found by the quality reviewer on T-1416. In `functions/date_and_time/workday.rs`, `fn_workday` and `fn_workday_intl` each have their own forward/backward loop, and the two are near-identical:
- each checks the bounds against MINIMUM/MAXIMUM_DATE_SERIAL_NUMBER;
- each skips weekends with `is_weekend` and holidays with `holiday_set`;
- each converts the end through `DateSystem::serial_from_1900_serial` (#NUM! before the 1904 epoch).

WORKDAY is WORKDAY.INTL with the default Saturday-Sunday mask (`DEFAULT_WEEKEND`). Make one helper that takes the start date, the day count, the mask and the holidays, and returns the result serial or #NUM!. Behavior must not change.

## Acceptance criteria
- [ ] One stepping helper is shared by both functions, and duplicated lines go down (`check_duplication.py --update`, decision 0010).
- [ ] No eval change: `make eval-check` shows 0 fixed and 0 regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any behavior change, including array lifting of the start date (T-1236).
