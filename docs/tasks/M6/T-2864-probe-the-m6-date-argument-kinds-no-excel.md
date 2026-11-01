---
id: T-2864
title: "Probe the M6 date argument kinds no Excel cell covers yet: empty text in 20 date parameters, an error in WEEKDAY, WEEKNUM and ISOWEEKNUM"
milestone: M6
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Every value parameter of the M6 date functions below has at least one Excel-backed cell for each argument kind the
report's argument coverage expects.

## Context
Gap triage 2026-10-27 (`make eval` on 7bd9090e, report.json `argument_coverage`, decision 0031 evidence gaps;
decision 0094 asks for each argument mode to be covered). The kinds are those of
`crates/rustsheet-eval/src/argument_coverage.rs` (`classify_value`): number, zero, negative, fraction,
numeric_text, text, empty_text, boolean, error, blank, range. The generator's number pool
(tools/src/rustsheet_tools/gen/generator.py `NUMBER_POOL`) has no empty text or error literal (T-2866), so
generated suites never reach most of these. `array`-mode parameters are not listed: their coverage is not
classified by kind.
These M6 date parameters have no Excel cell of the listed kinds in any suite:
- empty text: DATE.year, DATE.month, DATE.day; TIME.hour, TIME.minute, TIME.second; EDATE.start_date, EDATE.months;
  EOMONTH.start_date, EOMONTH.months; DATEDIF.start_date, DATEDIF.end_date; HOUR, MINUTE, SECOND.serial_number;
  NETWORKDAYS.start_date, NETWORKDAYS.end_date; WORKDAY.start_date; DAYS.end_date, DAYS.start_date;
  YEARFRAC.start_date, YEARFRAC.end_date, YEARFRAC.basis; DAYS360.start_date, DAYS360.end_date, DAYS360.method;
  NETWORKDAYS.INTL.start_date, end_date, weekend; WORKDAY.INTL.start_date, days, weekend; WEEKDAY.return_type.
- error: WEEKDAY.serial_number, WEEKNUM.return_type.
- empty text and error: WEEKNUM.serial_number, ISOWEEKNUM.date.
- a multi-cell range: TIME.second.
- zero: DATEDIF.unit (`DATEDIF(a,b,0)`), TIMEVALUE.time_text (`TIMEVALUE(0)`).

Write one probe case, probes/m6-date-argument-kind-gaps (excel-probe skill, excel-graph, tag M6), with each kind in two forms where
they differ: a literal (`""`, `NA()`, `#DIV/0!`) and a cell holding it (a formula cell `=""`, `=NA()`). Give every
other argument an ordinary valid value, so the probed argument decides the result, and wrap each error-valued
cell in `ERROR.TYPE` too.

## Acceptance criteria
- [ ] probes/m6-date-argument-kind-gaps is committed with oracle values (excel-graph) and covers every parameter/kind pair listed
      above; after `make eval`, report.json `argument_coverage` shows `covered == expected` for them.
- [ ] Every cell where ours differs from Excel is either fixed in this PR (a one-line coercion fix in the function's
      own module) or listed in a new functions-* task (campaign: semantics, with a `## Visible-parity stage`
      section) filed with `make new-task`, naming the case and cells; the new known failures are recorded with
      `make eval-baseline` and each has that owner.
- [ ] Any new Excel rule is recorded in docs/excel-semantics/dates.md (or coercion.md) citing the probe cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- DAY and YEAR (M2: T-2861); the holidays arrays of NETWORKDAYS/WORKDAY (`array` mode); generator pools (T-2866).
