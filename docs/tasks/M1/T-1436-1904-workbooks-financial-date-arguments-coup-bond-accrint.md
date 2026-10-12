---
id: T-1436
title: "1904 workbooks: financial date arguments (COUP*, bond, ACCRINT, AMORLINC) count from 1904-01-01"
milestone: M1
lane: functions-financial
status: in-progress
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
In a 1904 workbook, the financial functions read their date arguments from 1904-01-01, and the coupon-date functions return 1904 serials.

## Context
Split from T-1416. probes/date-system-1904-readers A18 has `COUPDAYS(DATE(2024,1,15),DATE(2030,6,30),2,1)` = 182 in Excel and 183 in ours. We read the 1904 serials as 1900 days, four years and a day earlier, so maturity 2030-06-30 becomes 2026-06-29. That date is not a month end, so the end-of-month coupon rule is lost.
- COUPNCD passes today only because the error cancels out.
- The readers are the `from_excel_date` calls in functions/financial/ (`financial_bonds.rs`, `accrint.rs`, `financial_depreciation.rs`, `mod.rs`).
- Reuse `DateSystem::date_of_serial` and `signed_serial_from_1900_serial` (formatter/dates.rs, T-1416).

Probe first, in a `date_system: 1904` case: COUPDAYBS, COUPDAYS, COUPDAYSNC, COUPNCD, COUPNUM, COUPPCD, PRICE, YIELD, ACCRINT, ACCRINTM, AMORLINC, DISC, ODDFPRICE. Use dates that straddle a month end or 29 February.

## Acceptance criteria
- [ ] A 1904 probe case covers the functions above, and its results are recorded in dates.md "The 1904 date system".
- [ ] The engine matches every cell, including probes/date-system-1904-readers A18.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Date text coercion and criteria (T-1435).
