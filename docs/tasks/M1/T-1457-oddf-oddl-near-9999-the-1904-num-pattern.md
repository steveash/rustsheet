---
id: T-1457
title: "ODDF*/ODDL* near 9999: the 1904 #NUM! pattern and the 1900 #NUM! at the last dates"
milestone: M1
lane: functions-financial
status: in-progress
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
- [ ] The listed cells leave eval/baseline/probes.txt, or the rule they follow is documented and each cell left is
      explained in dates.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Last-digit precision (T-1374, T-0916).
