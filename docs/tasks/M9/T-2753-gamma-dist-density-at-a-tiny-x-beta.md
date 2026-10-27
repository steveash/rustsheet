---
id: T-2753
title: "GAMMA.DIST density at a tiny x / beta: Excel's EXP(a LN(y) - y - LNGAMMA(a)) / x, 1E-14 off the exact value"
milestone: M9
lane: functions-stat
status: in-progress
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
- [ ] A probe brackets the regime; the engine matches its cells and rows 2, 4, 15, 19 of t2738-gamma-dist-tiny-ratio.
- [ ] statistics.md records the form; no regressions (CHISQ.DIST and GAMMA.DIST evidence).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The #NUM! at a subnormal x / beta (T-2738, done).
