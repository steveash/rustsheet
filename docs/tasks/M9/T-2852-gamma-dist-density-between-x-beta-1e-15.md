---
id: T-2852
title: "GAMMA.DIST density between x / beta 1E-15 and 1E-10, and at shape 50: find Excel's form (up to 37 ulp off today)"
milestone: M9
lane: functions-stat
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
GAMMA.DIST's density matches Excel bit for bit (or within 1 ulp, stated) where T-2753's log form does not reach: x / beta
from 1E-15 to about 1E-10 at shapes below 20, the 2-3 ulp residues below 1E-15, the normal range at a few ulp, and shapes of 20 and up.

## Context
T-2753 (probes/t2753-gamma-density-switch, -ratio-sweep, excel-graph 2026-10-27) found Excel's density below x / beta =
1E-15 is EXP(a LN(y) - y) / GAMMA(a) / x within 1 ulp (gamma.rs `tiny_ratio_density`). Between 1E-15 and 1E-10 no tried form
fits: the log form misses by 4 to 37 ulp (switch A6 26, A20 -26, B20 -37) and the direct y^a e^-y / Gamma(a) / x by 5 to
20; from 1E-5 up both are 1 to 4 ulp off (sweep A9:A11, A21:A23). At shape 50 (sweep A46:A48, B46:B48, the TOMS 654
`rcomp` branch of `incomplete_gamma::gamma_prefactor`) the engine is 34 to 92 ulp off. The forms tried are listed in the
T-2753 Outcome. Below 1E-15 the log form is still 2 to 3 ulp off at a dozen cells near 2E-16 and at sweep A19
(statistics.md "GAMMA.DIST's density at a tiny x / beta"), and its shape cutoff at 20 is unprobed (probe shapes 5, 10,
19.9). Try Excel's x87 kernels in extended precision (an 80-bit exponent argument; x87_exp.rs, x87_log10.rs),
and a series in y for small y. CHISQ.DIST shares `gamma_density` and must not regress.

## Acceptance criteria
- [ ] The engine matches the cells named above (or the plan for each that cannot is filed); no regressions.
- [ ] statistics.md records the forms; `make check` green; reviewers approve.

## Out of scope
- The #NUM! at a subnormal x / beta (T-2738).
