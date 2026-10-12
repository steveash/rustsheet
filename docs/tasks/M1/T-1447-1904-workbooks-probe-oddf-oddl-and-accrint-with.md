---
id: T-1447
title: "1904 workbooks: probe ODDF*/ODDL* and ACCRINT with a maturity from 9995-12-31 on (the coupon schedule break)"
milestone: M1
lane: functions-financial
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
In a 1904 workbook, ODDFPRICE, ODDFYIELD, ODDLPRICE, ODDLYIELD and ACCRINT match Excel for a maturity from
9995-12-31 on, where Excel's coupon schedule breaks.

## Context
Found by T-1440 (docs/excel-semantics/dates.md, "the coupon schedule breaks for any maturity serial m with
m + 2·1462 past the last date"). The COUP* functions, PRICE, YIELD and (M)DURATION follow
`DateSystem::coupon_schedule_breaks` (formatter/dates.rs). The odd-period functions and ACCRINT also walk a coupon
schedule, but dates.md lists them as unprobed. Also probe COUPDAYS on basis 1 with a second settlement in the broken
range, to confirm it ignores the settlement (T-1440 review note).

## Acceptance criteria
- [ ] A 1904 probe and its 1900 twin cover ODDFPRICE, ODDFYIELD, ODDLPRICE, ODDLYIELD and ACCRINT with maturities 9995-12-30 and 9995-12-31 and in 9999, plus COUPDAYS basis 1 with two settlements; findings are recorded in dates.md.
- [ ] The new probe cells pass, or each remaining failure is filed as a task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Last-digit precision of the financial functions (T-1374, T-0916).
