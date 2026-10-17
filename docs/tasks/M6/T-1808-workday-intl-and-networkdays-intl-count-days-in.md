---
id: T-1808
title: "WORKDAY(.INTL) and NETWORKDAYS(.INTL) count days in closed form"
milestone: M6
lane: functions-date
status: todo
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
One WORKDAY, WORKDAY.INTL, NETWORKDAYS or NETWORKDAYS.INTL call costs O(holidays) (plus O(1) per weekend mask), not one step per calendar day, with every result and error unchanged. Then T-1236 can enter them in the SPEC-0011 table without a weight that caps a lift at a few dozen elements.

## Context
Found while resuming T-1236 (opus, 2026-10-17). SPEC-0011's `KernelWork::Weighted(w)` charges every element the same fixed weight against `MAX_LIFTED_CELL_VISITS` (2^27). Today the kernels loop once per calendar day:
- `step_workdays` (date_and_time/workday.rs) steps one day at a time, up to the whole date range (about 2.96M days).
- `fn_networkdays` scans `from..=to`, and `fn_networkdays_intl` scans once per weekday count.

A weight that covers the worst case (about 2.96M) allows only about 45 elements per lifted call. Main already lifts a weekend range of any size through `Weekend::lift` (one scan, then a memo per mask), so moving `weekend` onto the driver as `Lift` would turn a 1,000-cell weekend range from a value into `#ERROR!`, a regression.

Closed forms:
- **NETWORKDAYS(.INTL):** for each weekday, count `[from, to]` in whole weeks plus the remainder, then subtract the distinct holidays inside `[from, to]` that fall on a working day of the mask.
- **WORKDAY(.INTL):** take whole weeks of `7 - weekend days` working days, then step the rest of the days. Holidays need care: sort and dedup the working-day holidays, then extend the target while holidays fall inside the span. Each pass consumes at least one holiday, so the work is O(H log H).

Keep the existing out-of-range `#NUM!` exactly:
- the step past `MAXIMUM_DATE_SERIAL_NUMBER` / `MINIMUM_DATE_SERIAL_NUMBER` (probes/date-system-1904-readers A43, A44);
- the endpoint `excel_date` checks of NETWORKDAYS;
- the all-weekend `#VALUE!` of WORKDAY.INTL.

Differential-test the closed forms against the current loops (kept as a test oracle) over random starts, day counts, masks and holidays, including the range edges. Semantics: docs/excel-semantics/dates.md "Date function argument modes". The old interim T-1236 branch (commit 55b64dca) has reusable cost estimates (`day_span` / `step_days`) and the tests in `test_fn_date_array_lift.rs`.

## Acceptance criteria
- [ ] The four kernels do no per-calendar-day loop. A unit test gives `WORKDAY(1,2900000)` and `NETWORKDAYS(1,2958465)` in well under a millisecond.
- [ ] A differential test (at least 10,000 random cases, every weekend mask shape, holidays inside and outside the span, start dates next to both range edges) matches the old loops exactly, values and errors.
- [ ] No eval cell changes (`make eval` delta 0); SPEC-0011's kernel-work note says these kernels are O(holidays).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
