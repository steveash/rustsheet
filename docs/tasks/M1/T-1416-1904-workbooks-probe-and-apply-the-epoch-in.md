---
id: T-1416
title: "1904 workbooks: probe and apply the epoch in the remaining date readers (WEEKDAY, EDATE/EOMONTH, DATEDIF, date-text coercion, criteria, TODAY)"
milestone: M1
lane: functions-date
status: in-progress
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
Every date reader in a 1904 workbook counts serials from 1904-01-01 as Excel does, with probe evidence for each.

## Context
Found while porting T-1235 (#520): typing "1904-01-02" into a cell of a 1904 workbook stores 1463, a serial
counted from 1900, instead of 1. Probe cell input along with the date-text coercion.
Follow-up of T-1330 (SPEC-0005 step 4). T-1330 applies the 1904 epoch to TEXT/cell-format date tokens,
DATE, DATEVALUE (and every reader of `Model::parse_datevalue_text`) and YEAR/MONTH/DAY, through
`DateSystem::to_1900_serial`/`serial_from_1900_serial` (formatter/dates.rs), on the evidence of
probes/date-system-1904-negatives A8-A12. Still 1900-only, and unprobed in 1904: WEEKDAY/WEEKNUM/ISOWEEKNUM
(`from_excel_date`), EDATE/EOMONTH (`month_arithmetic_date`), DATEDIF/DAYS360/YEARFRAC/NETWORKDAYS/WORKDAY,
the financial date arguments (`financial_bonds.rs`), date text coerced to a number (`"1904-01-02"+0`,
`parse_formatted_number` in formatter/format.rs), criteria date text (functions/criteria.rs), TODAY/NOW
(utils.rs, tz/), and DATE with a year in 0..=1903 (1900 is added today) or a negative 1904 serial in YEAR.
DATEDIF's text arguments already read 1904 serials through `parse_datevalue_text` (T-1330) while its
calendar is still 1900. Also confirm with the oracle the hand-derived 1904 bounds in
formatter/test/test_dates.rs (serial 2957003 is 9999-12-31, 2957004 is no date) and DATE/YEAR before the
epoch (#NUM! today). Probe first in a `date_system: 1904` case (excel-probe skill), then implement on the
evidence.

## Acceptance criteria
- [ ] A 1904 probe case covers each reader above; dates.md "The 1904 date system" records the results.
- [ ] The engine follows the evidence, reusing `DateSystem::to_1900_serial`/`serial_from_1900_serial`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Cell display of negatives (not observable through excel-graph; M14).
