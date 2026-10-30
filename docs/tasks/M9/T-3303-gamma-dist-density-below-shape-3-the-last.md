---
id: T-3303
title: "GAMMA.DIST density below shape 3: the last bits of the log form (y above 16, shapes below 0.5, below 1E-15)"
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
GAMMA.DIST's (and CHISQ.DIST's) density below shape 3 is Excel's bit for bit, or within 1 ulp (stated per cell), where
T-2852's EXP(a LN(y))/GAMMA(a)/x*EXP(-y) form stops: y = x / beta above 16, shapes below 0.5, y at most 1E-15, and the
1 to 6 ulp residue inside its range.

## Context
T-2852 (docs/excel-semantics/statistics.md "GAMMA.DIST's density below shape 3 is a log-space form") found Excel's error
there is a times LN(y)'s rounding error, and the form above fixes 44 cells, but it is not bit-exact everywhere, so gamma.rs
`log_form_density` bounds it by fitted constants:
- y above `LOG_FORM_Y_MAX` (16): gen-m9/c468f5046aadfbbc A171 `GAMMA.DIST(40,1.5,1.999,FALSE)` is 10 ulp off on the form
  and 5 on the direct one (Excel is within 1 ulp of the true value); e^-y of the extended quotient x / beta gives 3, but
  loses probes/t2753-gamma-density-ratio-sweep B35 (gated) by 1 ulp. Test the extended quotient in LN too, and Excel's
  `/` (`extended_quotient`) for y.
- Shapes below `LOG_FORM_SHAPE_MIN` (0.5): gen-m9/0a614cfe974b59c3 A367 (shape 0.1) is bit-exact on the direct form and 1
  ulp off on the log one; gen-m9/4e55500bb8ad3cd8 A386 (shape 1E-10) is 4 ulp off on the direct form and exact on the log
  one. T-3060's small-shape cdf leading factor EXP(a*LN(x))/EXP(GAMMALN(a+1)) is a candidate.
- At y up to 1E-15 the log form and T-2753's EXP(a LN(y) - y)/GAMMA(a)/x are each exact at different cells (switch A39,
  A40, B39, B41 on the latter, about 20 others on the former).
- Residue within the range: the m9-chisq-dist A36, A38, A40, m9-gamma-dist-a2 A8, A18, t1049-chisqdist A4, t1048-gammadist
  A6, B6 and gen-m9/a54c738c697617c3 A29 are 1 to 6 ulp off (bit-exact or 1 ulp on the direct form).
A probe sweep (GAMMA.DIST densities at shapes 0.05 to 2.9, x / beta 1E-17 to 40, both a literal y with beta = 1 and an
inexact quotient) is likely needed first (`excel-probe` skill). Shape 50 below a/100 is T-3219's.

## Acceptance criteria
- [ ] The cells named above are bit-exact or within 1 ulp, or each form tried is recorded in statistics.md with its counts
      (decision 0056); no gated regression and no cell newly visibly off.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Shape 50 at y below a/100 (T-3219); shapes from 3 up (T-3147, T-3219).
