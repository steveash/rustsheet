---
id: T-3044
title: "CHISQ.TEST and CHITEST last digits: fit Excel's statistic sum and tail (t1049-chisqtest, m9-legacy-chitest, 12 cells)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
defer: phase-1b
---

## Goal
CHISQ.TEST and CHITEST match Excel bit for bit on the probe cells where the engine is 1 to a few ulp off today. If
no formulation of the statistic and its tail does, the task records the candidates and their counts (decision 0056).

## Context
Gap triage 2026-10-28 (`make eval` on origin/main 4b20263be). No open task names these cells. T-3021 routes the one
visibly-off CHITEST cell, m9-legacy-chitest Calc!A23, and keeps it.
- probes-numeric/t1049-chisqtest Calc!A11, A25, A26, A29, A30, A34, A35, A38 (8; A38 is CHITEST). Five of them are
  the same 1x3 test in different shapes and with ignored text or booleans, `CHISQ.TEST({10,20,30},{15,15,30})`
  (A29). There the statistic is 10/3 and df = 2, so p is exp(-5/3). Excel 0.18887560283756183, ours
  0.18887560283756177 (3 ulp). A11 `CHISQ.TEST(C1:C4,B1:B4)`: Excel 0.7710426682428904, ours ...905.
- probes-numeric/m9-legacy-chitest Calc!A3, A4, A9, A10 (4; A9 and A10 are NumericClose, within 15 digits). A3
  `CHITEST(B1:D1,B4:D4)`: Excel 0.7052475879741441, ours 0.705247587974144. A10 `CHISQ.TEST(G1:G3,H1:H3)`: Excel
  0.06234947668967343, ours ...339.
- Engine: `fn_chisq_test` in `crates/rustsheet-engine/src/functions/statistical/chisq.rs` sums
  `(o - e)^2 / e` left to right. At df > 1 it then returns `1.0 - gamma_cdf`, a complement. CHISQ.DIST.RT goes
  through `chi_squared_sf` (the upper tail directly, and the ERFC kernel at df 1). Hypotheses to test, cheapest
  first:
  1. The tail is CHISQ.DIST.RT's (`chi_squared_sf`), not 1 - cdf.
  2. At df 2 the tail is Excel's EXP(-x/2) (x87 EXP, numbers.md).
  3. The statistic's sum order or form differs, for example (o - e)^2 / e against o^2 / e - 2o + e, or a sum
     accumulated in extended precision.
  For each candidate, a worksheet CHISQ.DIST.RT at the engine's statistic says whether the tail or the sum is off.
  If the cached cells cannot separate (2) from (3), the task says "Needs an oracle sweep" and files a research
  probe: CHISQ.DIST.RT(x,2) next to CHISQ.TEST at the same statistic, over a grid of small tables.
- Semantics: statistics.md, the incomplete-gamma and CHITEST entries ("Excel's regularized incomplete gamma ... is
  close to correctly rounded", the df-1 ERFC identity).

## Acceptance criteria
- [ ] Either the 12 cells pass bit-exact in `make eval` (probes-numeric is ungated, so the report's case counts are
      the check), with no new F line in any eval/baseline/*.txt and the CHISQ.TEST lines of
      ironcalc-calc STATISTICAL/CHISQ_TEST.xlsx not grown; or the task is blocked time-boxed (decision 0056), with
      the candidates and their counts recorded in statistics.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- m9-legacy-chitest Calc!A23 (visibly off, T-3021). The two-shape df rule (T-2650). The chi-squared quantiles
  (T-2561, T-2638).

## Deferred (decision 0114)
`defer: phase-1b`: every cell this task owns is already within 15 significant digits of Excel, so it waits until
Phase 1 exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells
visibly off (then give it a `campaign:` stage instead).
