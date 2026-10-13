---
id: T-1457
title: "ODDF*/ODDL* near 9999: the 1904 #NUM! pattern and the 1900 #NUM! at the last dates"
milestone: M1
lane: functions-financial
status: done
priority: P3
size: M
tier: opus
depends_on: [T-1447]
---

## Goal
ODDFPRICE, ODDFYIELD, ODDLPRICE and ODDLYIELD give Excel's #NUM! (or its value) for dates near the end of the
calendar, in a 1904 workbook and in 1900.

## Context
T-1447 probed these functions (docs/excel-semantics/dates.md, under "the coupon schedule breaks"). Known failures
(excel-graph):
- probes/date-system-1904-odd-accrint-bound A1:C10: #NUM! in 1904 for maturities 9995-12-30, 9995-12-31 and
  9999-06-30, where we give the 1900 values. Column D (9999-12-31) already matches.
- probes/date-system-1904-odd-threshold A1, B1, A5, B5 (#NUM! for 30 June 9000 and 9900, with all dates in the year
  before) and A3, B3, A6, B6 (ODDL* in 9000 and 9900 differ from 1900: 99.6718 vs 99.6715). 9980 through 9995 match.
- probes/date-system-1900-odd-accrint-bound C5, C6 (ODDF* with first coupon 9998-12-31, maturity 9999-06-30) and
  D7:D10 (ODDL* with maturity 9999-12-31): #NUM! in 1900 too.
Working guess (UNVERIFIED): in 1904 these functions move dates by four calendar years, not by 1462 days. That would
explain the non-leap centuries 9000 and 9900, and a quasi-coupon past 9999-12-31 would explain the #NUM! near the
end. Probe more points before implementing (excel-probe): maturities 30 June 9100/9200/9600, 8996 and 9004, and ODDF
first-coupon/maturity combinations around 9998-12-31. Code: functions/financial (odd-period functions) and
`DateSystem::financial_1900_serial` (formatter/dates.rs).
Last-digit misses of ODDFYIELD in these cases are T-0916, not this task.

## Acceptance criteria
- [x] The listed cells leave eval/baseline/probes.txt, or the rule they follow is documented and each cell left is
      explained in dates.md.
- [x] `make check` green; reviewers approve.

## Out of scope
- Last-digit precision (T-1374, T-0916).

## Progress (session 2026-10-13, opus; first session)
- Probed the years: probes/date-system-1904-odd-years and its control probes/date-system-1900-odd-years (114 cells
  each, excel-graph). Findings are recorded in docs/excel-semantics/dates.md (after "The odd-period functions break
  differently"). In short, a 1904 date within about four years before 1 March of a non-leap century year (2100, 2200,
  ..., 9900) breaks the odd-period functions, from 2096 on. So it is not an end-of-calendar effect. This fits moving
  1904 dates by four calendar years instead of 1462 days.
- Baseline: the new cases add 98 known failures. Rows 1, 2, 4, 5 and 6 of the 1904 case fail only in the 12 broken
  years. Row 3 (ODDFYIELD) misses in every column of both cases; that looks like T-0916 last-digit precision, so check
  it before treating it as this task's.
- Next: find where the odd-period code converts 1904 serials (`DateSystem::financial_1900_serial`, formatter/dates.rs).
  Try "add 4 calendar years" (EDATE +48 months) in place of +1462, and see whether that reproduces the #NUM!, the
  99.67182 vs 99.67128 split (2100 vs 2096) and the row 5/6 values (98.86311 / 99.19898 for 2100). Then recheck the
  original T-1447 cells listed in Context.

## Outcome
- In a 1904 workbook ODDFPRICE, ODDFYIELD, ODDLPRICE and ODDLYIELD read a serial s as the 1900 serial s + 2*1462
  (`DateSystem::odd_period_1900_serial`), the date 1462 days later. It fits every probed odd-period cell: the
  non-leap-century breaks, the 99.67182 / 99.67128 split, and the 29 February cells of the new
  probes/date-system-1904-odd-feb29. Dates read past 9999-12-31 are #NUM!.
- New 1900 rules from new probes (date-system-1900-oddf-schedule, date-system-1900-oddf-issue-boundary): ODDF* are
  #NUM! unless the first coupon is on the maturity's schedule and before it, and the issue is not the quasi-coupon
  date one period before the first coupon (`check_odd_first_coupon`). This is what makes the 1904 ODDF* cells #NUM!.
  ODDL* are #NUM! when a quasi-coupon date is past 9999-12-31 (1900 D7-D10 and 1904 A7-A10 of the accrint-bound
  cases).
- Recorded in dates.md (odd-period paragraph) and financial.md (two new sections).
- Eval: probes known failures 2986 -> 2977 against main, while adding five cases (263 cells). Of the T-1447 cells in
  Context, only 1900 C5/C6 remain (T-1490). The cells still failing in the new cases are ODDFYIELD last-digit misses
  (row 3, T-0916). No regressions.
- Follow-ups: T-1489 (do the other financial readers use s + 2*1462?), T-1490 (1900 C5/C6).
