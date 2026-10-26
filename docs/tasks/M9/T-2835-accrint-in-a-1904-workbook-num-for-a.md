---
id: T-2835
title: "ACCRINT in a 1904 workbook: #NUM! for a coupon date before 1904-01-01 (43 probe cells)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-2337]
campaign: semantics
---

## Goal
In a 1904 workbook ACCRINT is `#NUM!` where today's Excel is, for a coupon date it accrues over that falls before
1904-01-01.

## Context
- T-2562 probed ACCRINT at the start of a 1904 workbook's calendar (excel-graph, 2026-10-26) but did not commit the case:
  its 43 off cells would exceed the visible-parity lock. Land it here as probes/accrint-calendar-start-1904 (tags M9,
  probe, dates, financial; `date_system: 1904`; sheet `S`): A1:A40 =
  `ACCRINT(issue,fi,st,0.05,1000,4,0,cm)` over issue in [DATE(1904,1,1), DATE(1904,1,2), DATE(1904,4,1), 0, 1], fi in
  [DATE(1904,3,31), DATE(1904,6,30)], st in [DATE(1904,2,15), DATE(1950,6,30)], cm in [TRUE, FALSE] (nested in that
  order); B1:B10 = `ACCRINT(DATE(1904,1,1),DATE(1904,3,31),DATE(1950,6,30),0.05,1000,freq,basis,TRUE)` over basis 0..4,
  freq [2, 4]. On 2026-10-26 43 of its 50 cells were `#NUM!` in Excel and numbers in ours (only A19, A23, A24 = 2312.36…
  and the settlement-before-issue cells A17, A18, A21, A22 matched), for example
  `ACCRINT(DATE(1904,1,2),DATE(1904,3,31),DATE(1950,6,30),0.05,1000,4,0,TRUE)` (A11, ours 2324.722222222222), the
  1904-01-01 issue on every basis (B1:B10), and issue 1904-04-01 with first_interest 1904-03-31 and calc_method FALSE
  (A20, ours 2312.361111111111). docs/excel-semantics/financial.md "ACCRINT at the calendar's start" and "COUPNUM in a
  1904 workbook" (fin-coupnum-1904-maturity-sweep C1-C21 says the same).
- The cut matches the 1900 rule moved to the 1904 calendar's first day (the previous coupon of the settlement or of the
  issue, or the regular one with calc_method FALSE, before 1904-01-01). The financial functions read 1904 dates as
  serial + 2 * 1462 (`DateSystem::financial_1900_serial`), so the cut has to be placed in that frame:
  crates/rustsheet-engine/src/functions/financial/accrint.rs `accrint_coupon_serial`.
- Blocked on T-2337: the 1904 financial results changed in Excel for the web after 2026-10-12 and the committed 1904
  sidecars conflict with today's values; implement once T-2337 settles which values are current, and re-fill probes/accrint-calendar-start-1904
  first (its values are Excel for the web's of 2026-10-26).

## Acceptance criteria
- [ ] probes/accrint-calendar-start-1904 lands, re-filled, and every cell passes, with no new known-failure lines in any
      baseline.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
