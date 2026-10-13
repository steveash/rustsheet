---
id: T-1436
title: "1904 workbooks: financial date arguments (COUP*, bond, ACCRINT, AMORLINC) count from 1904-01-01"
milestone: M1
lane: functions-financial
status: done
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
- [x] A 1904 probe case covers the functions above, and its results are recorded in dates.md "The 1904 date system".
- [x] The engine matches every cell, including probes/date-system-1904-readers A18.
- [x] `make check` green; reviewers approve.

## Out of scope
- Date text coercion and criteria (T-1435).

## Outcome
The probe found a quirk: in a 1904 workbook, YEARFRAC and every financial date reader read each date one day late.
The COUP* functions, PRICE/YIELD/DURATION, ODDF*/ODDL*, ACCRINT(M), AMORLINC/AMORDEGRC, DISC family, PRICEMAT/YIELDMAT
and TBILL* all do this. Serial s is read as 1900 serial s + 1463, and COUPNCD/COUPPCD return a day early. DAYS360 and the
other readers count correctly. Evidence: four probe cases (148 cells, excel-graph) recorded in dates.md, "The 1904 date
system: financial day counts read every date one day late". The probes include a 1900 twin of 40 cells.
- `DateSystem::financial_1900_serial` and `serial_from_financial_1900_serial` (formatter/dates.rs) do the shift.
  YEARFRAC applies it. So do the financial readers, through `Model::financial_date` / `get_financial_date`.
- probes/date-system-1904-readers A18 passes, and every shift-sensitive cell of the new cases passes.
- One exception, split to T-1440: COUPNCD with a 9999-12-31 maturity (A43).
- Eval: probes known failures 2807 -> 2829 (fixed 1, new-coverage 23, regressions 0). The 22 other new-coverage cells
  are last-digit misses (DISC, INTRATE, PRICEDISC, PRICEMAT, YIELDDISC, YIELDMAT, TBILL*; T-1374) or iteration stopping
  points (XIRR, YIELD, ODDFYIELD; T-0916). Most of them fail in the 1900 twin too.
