---
id: T-2614
title: "TRIMMEAN: an error percent beats data with no number (gen-m9 a5342a7e24fd803e A288)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
TRIMMEAN returns an error-valued percent before it reports data with no number, as Excel does.

## Context
gen-m9 a5342a7e24fd803e Calc!A288 `=TRIMMEAN(Data!E11:E11,(COVARIANCE.P(Data!B3:E4,Data!B9:B11)-LOGEST(Data!A$6,1E+15,"")))`
is `#N/A` in Excel (excel-graph, 2026-10-22) and `#NUM!` in ours. Data!E11 is TRUE, so the data has no number; the percent
is `#N/A` (COVARIANCE.P of 8 against 3 cells; ours computes that `#N/A` too, and `#N/A - #VALUE!` is `#N/A`). Ours reports
the empty data's `#NUM!` first; with a number as the data (`TRIMMEAN(1,NA())`) ours already gives `#N/A`.
T-1742 found this (the cell was listed under LOGEST); statistics.md "TRIMMEAN: an error percent before data with no number
(T-1742)". Code: functions/statistical (TRIMMEAN); see also statistics.md "A lone text, boolean or empty cell as the data
of QUARTILE, PERCENTILE, PERCENTRANK and TRIMMEAN (T-2256)" for the data rule.

## Acceptance criteria
- [ ] gen-m9 a5342a7e24fd803e Calc!A288 passes; its line leaves eval/baseline/gen-m9.txt (`make eval-baseline`), with no
      new known-failure lines in any baseline.
- [ ] A unit test covers `TRIMMEAN(<TRUE cell>,NA())` = `#N/A` and `TRIMMEAN(<TRUE cell>,0.1)` = `#NUM!`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Whether QUARTILE, PERCENTILE and PERCENTRANK order the same way (unprobed; file a probe task if needed).
