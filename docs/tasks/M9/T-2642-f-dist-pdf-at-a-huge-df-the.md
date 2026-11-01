---
id: T-2642
title: "F.DIST pdf at a huge df: the last ulp of gen-m9 285d87cea1aaa73c A64 (exponent summed in double-double)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-1864]
---

## Goal
gen-m9/285d87cea1aaa73c Calc!A64 (an F.DIST pdf at df1 about 1.2E8, df2 1) is bit-exact with Excel, so its
known-failure line leaves eval/baseline/gen-m9.txt.

## Context
Split from T-1864 (its bit-level acceptance criterion, moved here by that task's Visible-parity stage rule). T-1864
made `f_pdf` use Loader's saddle-point binomial form where statrs' direct density overflows (`fisher.rs`
`f_pdf_loader`, `functions/statistical/loader.rs` `stirlerr`, `bd0_dev`, `dbinom_raw_dev`). On
`=F.DIST(WRAPROWS(Data!A1,LET(acc,Data!$E$12,w,0.2,100),(2.5*0.2)),(123456789.123+ABS(Data!E6)),1,FALSE)` (x 0.1,
df 1.2E8 and 1) ours is 0.08500367962366501 against Excel's 0.085003679623665, which is also the exact density's
double (mpmath): 1 ulp, no longer visibly off, but still a known failure. T-1864's analysis: the remaining error is
the rounding of the exponent `lc - lf / 2` that `dbinom_raw_dev` passes to `exp` (about 0.67 ulp of the result
there), so summing the exponent's terms (the stirlerr terms, the two bd0 terms, half of lf) in double-double, or an
equivalent compensated sum, should bring the result to the correctly rounded density. Check the change against every
F.DIST pdf cell of probes-numeric (t2220-f-dist-sweep-*, t1530-f-df-cut, t1864-f-pdf-moderate-x), m9-f-dist and
gen-m9, and the unit tests in test_fn_f_pdf_large_df.rs; reuse `crate::arithmetic`'s extended helpers if they fit.

## Acceptance criteria
- [ ] gen-m9/285d87cea1aaa73c Calc!A64's known-failure line disappears from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] No passing F.DIST/F.DIST.RT/FDIST cell changes (compare every suite cell by cell with main), and the
      test_fn_f_pdf_large_df.rs tolerances do not grow.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Excel's own large-shape noise on other F pdf cells (T-2586).
- The beta and Student-t densities (T-2507, T-2585).

## Note (T-3034)
`f_pdf_loader` and `loader.rs` are gone: T-3034 made every x > 0 density Excel's own chain M-final
(`functions/statistical/f_density.rs`). Re-triage this task's cells against it before working it.
