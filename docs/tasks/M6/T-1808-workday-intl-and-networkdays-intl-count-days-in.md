---
id: T-1808
title: "NETWORKDAYS and NETWORKDAYS.INTL count days in closed form"
milestone: M6
lane: functions-date
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
One NETWORKDAYS or NETWORKDAYS.INTL call costs O(holidays) (plus O(1) per weekend mask), not one step per calendar day, with every result and error unchanged. With T-1295, which does the same for WORKDAY(.INTL)'s `step_workdays`, T-1236 can enter all four in the SPEC-0011 table without a weight that caps a lift at a few dozen elements.

## Context
Found while resuming T-1236 (opus, 2026-10-17). SPEC-0011's `KernelWork::Weighted(w)` charges every element the same fixed weight against `MAX_LIFTED_CELL_VISITS` (2^27). Today the kernels loop once per calendar day:
- `step_workdays` (date_and_time/workday.rs) steps one day at a time, up to the whole date range (about 2.96M days). T-1295 (its own branch) fixes that for WORKDAY and WORKDAY.INTL; this task does not touch it.
- `fn_networkdays` scans `from..=to`, and `fn_networkdays_intl` scans once per weekday count.

A weight that covers NETWORKDAYS' worst case (about 2.96M) allows only about 45 elements per lifted call. Main already lifts a weekend range of any size through `Weekend::lift` (one scan, then a memo per mask), so moving `weekend` onto the driver as `Lift` would turn a 1,000-cell weekend range from a value into `#ERROR!`, a regression.

Closed forms:
- **NETWORKDAYS(.INTL):** for each weekday, count `[from, to]` in whole weeks plus the remainder, then subtract the distinct holidays inside `[from, to]` that fall on a working day of the mask.

Keep the existing errors exactly: the endpoint `excel_date` checks of NETWORKDAYS(.INTL), the serials with no date (in the 1900 system, the loop's handling of serial 60), and the weekend-argument errors.

Differential-test the closed forms against the current loops (kept as a test oracle) over random starts, day counts, masks and holidays, including the range edges. Semantics: docs/excel-semantics/dates.md "Date function argument modes". The old interim T-1236 branch (commit 55b64dca) has tests to reuse in `test_fn_date_array_lift.rs`.

## Acceptance criteria
- [x] NETWORKDAYS and NETWORKDAYS.INTL do no per-calendar-day loop. A unit test gives `NETWORKDAYS(1,2958465)` and `NETWORKDAYS.INTL(1,2958465,"0000011")` in well under a millisecond.
- [x] A differential test (at least 10,000 random cases, every weekend mask shape, holidays inside and outside the span and repeated, both 1900 and 1904 systems, ranges across serial 60 and next to both range edges) matches the old loops exactly, values and errors.
- [x] No eval cell changes (`make eval` delta 0); SPEC-0011's kernel-work note and its T-1236 rollout line say T-1236 follows T-1295 and T-1808.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Done (2026-10-19, opus). `networkdays_per_weekday` (functions/date_and_time/networkdays.rs, where both functions moved so the oversized date_and_time.rs drops under 1,000 lines) counts the span per
weekday in closed form, whole weeks plus the remainder minus the distinct holidays inside it, and
both NETWORKDAYS (Saturday–Sunday mask) and NETWORKDAYS.INTL (each lifted mask) sum it through
`working_days`. Only the endpoints are checked: a date system's dates are one run of serials, so
the old loops' per-day `#NUM!` and skips could only fire at an endpoint. `test_networkdays_closed_form.rs`
keeps both old loops as the oracle over 12,000 random cases (6,000 per date system; codes and all
128 string masks; holidays inside, next to and repeated; starts next to both range edges and
around serial 60), and evaluates 2,000 whole-range calls in about 8 ms (debug; asserted under 5 s). SPEC-0011 notes
that T-1236 follows T-1295 and T-1808. Eval delta 0.
