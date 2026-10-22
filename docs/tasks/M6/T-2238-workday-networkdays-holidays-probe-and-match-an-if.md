---
id: T-2238
title: "WORKDAY/NETWORKDAYS holidays: probe and match an IF whose 3-D branch is not taken and a 3-D holidays with 0 days (ours #VALUE! and the start date, unprobed)"
milestone: M6
lane: functions-date
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
WORKDAY, WORKDAY.INTL, NETWORKDAYS and NETWORKDAYS.INTL give Excel's result, backed by oracle
evidence, in the two holidays cases that dates.md leaves unprobed. In the first, the holidays
argument is an IF whose taken branch is an ordinary reference but whose other branch is a 3-D span.
In the second, the holidays argument is a 3-D span and the day count is 0.

## Context
Gap triage 2026-10-21. These are M6 functions. dates.md "An error-valued or 3-D weekend, 3-D date text and holidays"
(probes/date-text-weekend-3d Sheet1!M9-N13, excel-graph, T-1428) shows that a 3-D `holidays` is
`#VALUE!`, also as the taken branch of IF:
`WORKDAY.INTL(45000,5,1,Sheet1:Sheet2!A1)` and `NETWORKDAYS(45000,45030,IF(TRUE,Sheet1:Sheet2!A1))`.
It then says: "Unprobed: an IF whose 3-D branch is not taken (`IF(FALSE,Sheet1:Sheet2!A1,A1)`, ours
`#VALUE!`) and a 3-D holidays with a day count of 0 (ours the start date)."

The engine decides this statically. `functions/date_and_time/holidays.rs` refuses any argument for
which `functions::util::is_span_or_if_of_span` holds, and that helper looks at every IF branch
whatever the condition. So `IF(FALSE,Sheet1:Sheet2!A1,A1)` is refused before the condition is read.
COUNT uses the same helper (`statistical/count.rs`), so change the holidays caller, not the shared
helper, unless COUNT is probed to behave the same way.

Probe (excel-probe skill, excel-graph oracle; about 30 cells), in the two-sheet layout of
probes/date-text-weekend-3d:
- each of the four functions with holidays `IF(FALSE,Sheet1:Sheet2!A1,A1)`,
  `IF(TRUE,A1,Sheet1:Sheet2!A1)`, `IF(A9,Sheet1:Sheet2!A1,A1)` with A9 FALSE, and
  `CHOOSE(2,Sheet1:Sheet2!A1,A1)`;
- WORKDAY and WORKDAY.INTL with days 0 and a 3-D holidays, from a workday and from a weekend start;
- NETWORKDAYS with equal start and end and a 3-D holidays;
- for each holidays form, a lifted one, `ROWS(WORKDAY({45000;45001},5,IF(FALSE,Sheet1:Sheet2!A1,A1)))`.

Record the rule in dates.md under "An error-valued or 3-D weekend, 3-D date text and holidays",
replacing the "Unprobed" sentence. Implement it in `holiday_serials` / `workday_call`.

## Acceptance criteria
- [ ] A new probes case, oracle-filled (excel-graph), covers the forms above. Every cell passes, or
      its known failure is recorded with `make eval-baseline` and has a filed task named in the Outcome.
- [ ] dates.md has no "Unprobed" sentence left for these two cases; the rule cites its cells.
- [ ] Unit tests cover the new holidays rule. COUNT's 3-D handling is unchanged (its probes and unit
      tests still pass), and no gated baseline gains a line.
- [ ] `make check` green; reviewers approve.

## Out of scope
- 3-D `weekend` arguments (already `#VALUE!`, probed).
- COUNT/COUNTA over IF of a 3-D span (statistical lane; probe separately if this task's rule
  suggests COUNT differs).
