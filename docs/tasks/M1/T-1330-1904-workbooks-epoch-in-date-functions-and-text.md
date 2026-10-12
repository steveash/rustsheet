---
id: T-1330
title: "1904 workbooks: epoch in date functions and TEXT date tokens, negatives as minus (SPEC-0005 step 4)"
milestone: M1
lane: functions-date
status: done
priority: P3
size: M
tier: opus
depends_on: [T-1105]
spec: docs/specs/SPEC-0005-workbook-date-system.md
---

## Goal
In a 1904 workbook, date functions and TEXT date tokens use the 1904 epoch, and negative serials format as a minus plus the absolute value, as probed.

## Context
SPEC-0005 rollout step 4 (docs/specs/SPEC-0005-workbook-date-system.md); evidence probes/date-system-1904-negatives (excel-graph): the failing cells A1, A2, A6, A7, A14 (TEXT negatives), A8, A9 (TEXT epoch), A10-A12 (DATE, YEAR, DATEVALUE); A4, A5, A13 and A15 already pass (the engine prints a minus in either system), and A18 is T-1329's. docs/excel-semantics/dates.md "The 1904 date system". The 1462-day shift applies to untrusted serials: checked arithmetic, an Excel error out of range (SPEC-0005). The formatter part builds on T-1105's `FormatContext`; date functions read `WorkbookSettings::date_system`. Split if it is bigger than M.

## Acceptance criteria
- [x] The listed probes/date-system-1904-negatives cells pass and leave the baseline; probes/date-system-1900-negatives does not regress.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`DateSystem::to_1900_serial`/`serial_from_1900_serial` (formatter/dates.rs) shift a 1904 serial by 1462 days,
checked, with no 1904 serial below 0. The formatter's day, month, year and weekday-name tokens, DATE,
YEAR/MONTH/DAY and `parse_datevalue_text` (DATEVALUE and the date-text readers built on it) use them.
probes/date-system-1904-negatives passes 18 of 18; probes/date-system-1900-negatives stays at 18 of 18.
`WeekendPattern` moved to functions/date_and_time/weekend.rs to keep date_and_time.rs under its size ratchet.
Eval delta: probes 13 known failures fixed (8 in date-system-1904-negatives, 5 in negative-date-sections-1904);
poi 566 -> 538 (28 fixed in DateFormatTests.xlsx and DateFormatNumberTests.xlsx, both 1904 workbooks);
0 regressions. Follow-up T-1416 (probe, then apply the epoch to the remaining date readers).
