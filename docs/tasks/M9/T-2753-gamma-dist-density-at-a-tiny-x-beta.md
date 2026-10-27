---
id: T-2753
title: "GAMMA.DIST density at a tiny x / beta: Excel's EXP(a LN(y) - y - LNGAMMA(a)) / x, 1E-14 off the exact value"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
GAMMA.DIST's density matches Excel bit for bit where x / beta is tiny but normal, in the regime where Excel's value is
about 1E-14 relative off the exact density.

## Context
probes/t2738-gamma-dist-tiny-ratio (excel-graph 2026-10-25, T-2738) shows `GAMMA.DIST(3E-308,1,1,FALSE)` is
1.0000000000000409 (exact 1), `GAMMA.DIST(4.5E-308,1,2,FALSE)` 0.499999999999989, `GAMMA.DIST(1,1,1E+307,FALSE)`
9.999999999999768E-308 and `GAMMA.DIST(1E-5,1,1E+300,FALSE)` 9.999999999999576E-301 (rows 15, 19, 2, 4; the engine
gives the exact values, NumericClose). In double precision `EXP(a*LN(y)-y-GAMMALN(a))/x` with y = x / beta reproduces
rows 2, 15 and 19 exactly and row 4 to 2 ulp, so Excel seems to take a log-space form whose error grows with |LN(y)|.
The engine's `incomplete_gamma::gamma_density` (the T-0971 prefactor, and e^(-y)/scale at shape 1, which CHISQ.DIST
evidence needs) is right elsewhere, so find where Excel switches forms (probe y from 1E-300 to 1E-10, shapes 1, 2,
0.5, 50) before changing it; try `excel_ln`/`excel_exp` for row 4.

## Acceptance criteria
- [x] A probe brackets the regime; the engine matches rows 2, 15 and 19 of t2738-gamma-dist-tiny-ratio exactly and row 4 to 1 ulp (Excel's own last bit there is unexplained), and the regime's cells to 0-1 ulp.
- [x] statistics.md records the form; no regressions (CHISQ.DIST and GAMMA.DIST evidence).
- [x] `make check` green; reviewers approve.

## Out of scope
- The #NUM! at a subnormal x / beta (T-2738, done).

## Outcome
probes/t2753-gamma-density-ratio-sweep (x / beta 1E-300 to 0.5, shapes 1, 2, 0.5, 50, beta = 1 and x = 1) and
-switch (1E-19 to 1E-11) bracket it: below about 1E-15 Excel's density is EXP(a LN(y) - y) / GAMMA(a) / x, whose log
carries |LN(y)| ulp of error (`GAMMA.DIST(1E-300,1,1,FALSE)` is 1.0000000000000235). With Excel's own LN, EXP and GAMMA
(x87_log10, x87_exp, gamma_x87) it is within 1 ulp of every probed cell there; LNGAMMA in place of GAMMA is 40 to 60 ulp
off at shape 0.5, and the direct y^a e^-y form 50 to 200. gamma.rs `tiny_ratio_density` takes it for x > 0, shape below
20 and x / beta below 1E-15, in GAMMA.DIST only (CHISQ.DIST keeps `gamma_density`). Cells visibly off in the sweep fell
from about 40 to 9. Between 1E-15 and 1E-10 Excel takes some other form, and shape 50 is far off: T-2852.
