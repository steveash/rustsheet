---
id: T-2852
title: "GAMMA.DIST density between x / beta 1E-15 and 1E-10, and at shape 50: find Excel's form (up to 37 ulp off today)"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: M
tier: opus
depends_on: []
campaign: visible
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
- [x] The engine matches the cells named above (or the plan for each that cannot is filed); no regressions.
- [x] statistics.md records the forms; `make check` green; reviewers approve.

## Out of scope
- The #NUM! at a subnormal x / beta (T-2738).

## Visible-parity stage
Decision 0114: Excel is the inaccurate side in this regime: its density carries a times the rounding error of LN(y)
(`GAMMA.DIST(1E-14,2,1,FALSE)` is 11.8 ulp below the true value, the shape-50 cells 50 to 105 ulp above it), so the task
is Excel's own form, stage `visible`. On main the task's cells were up to 26 ulp off in the 1E-15 to 1E-10 band (37 on
T-2753's log form), and visibly off at sweep B21 (5 ulp) and the six shape-50 cells (32 to 114 ulp).

## Outcome
Found Excel's form between x / beta 1E-15 and 1E-10 and beyond (attempt 1 of 2 for the band, decision 0056): from shape
0.5, GAMMA.DIST's density is EXP(a LN(y))/GAMMA(a)/x*EXP(-y) with Excel's LN, EXP and GAMMA. At every switch and sweep
cell from 1E-16 to 1E-10 Excel's relative error less a times LN(y)'s rounding error is within about 1 ulp. gamma.rs
`log_form_density` (was `tiny_ratio_density`) takes it for y above 1E-15 and up to 16, below shape 3 below the mode and
below shape 2.5 above it; each bound is a fit inside its bracket, named with its cells in gamma.rs and statistics.md
("GAMMA.DIST's density below shape 3 is a log-space form"), which also lists every form tried with its counts. From y = 2
up the form is no better than the direct one on balance (UNVERIFIED, counts there). T-2753's form stays at y up to 1E-15,
where it keeps four gated cells bit-exact.
Shape 50 below a/100 (sweep A46:A48, B46:B48) is not found: Excel's error there is 50 times LN(y)'s plus 41 to 52 units,
and no order of the exponent in doubles or an x87 register, nor the ratio form, Loader or `rcomp`, fits (13 to 435 ulp
off); T-3219 owns those cells, with these findings recorded in statistics.md. The rest of this form's bits (y above 16,
shapes below 0.5, y up to 1E-15, the residue) is T-3303 (`defer: phase-1b`); one density dispatch is T-3319.
Review round 1 cut the form at shape 2.5 above the mode, which restores m9-chisq-dist A36, A38, A40, t1049-chisqdist A38
and gated gen-m9/a54c738c697617c3 A29 to their main values.

Main (b2696e397) against HEAD, every cell of every vendored suite (calipers, corpus-bench and poi not fetched; both
binaries rebuilt from a `cargo clean` of the engine and harness):
- Fixed (bit-exact now), 44: 23 in gated suites (probes/t2753-gamma-density-switch and -ratio-sweep 18; gen-m9/
  8dd2312c96effb1b A150, 9cca17438a7e6f8a A133, bc40d37cb4d8301d A165, d8266bc15533702a A230, e6cf3b20a014e6a3 A92), 21
  in probes-numeric (m9-gamma-dist-a0p5, -a1, -a2, m9-chisq-dist, t1049-chisqdist; 1 to 2 ulp before).
- Lost bit-exact, 4 (all ungated probes-numeric): m9-gamma-dist-a1 A28 `GAMMA.DIST(1500000,1,1000000,FALSE)` -1 ulp,
  m9-gamma-dist-a2 A8 `GAMMA.DIST(3,2,1,FALSE)` 1 and A18 `(6,2,2,FALSE)` 1, t1049-chisqdist A4 `CHISQ.DIST(0.1,2,FALSE)` 2.
- Closer, 35 (switch A20 17 -> 1 ulp, B20 23 -> 1, A23 19 -> 1, B23 26 -> 1, A9 14 -> 1, B22 10 -> 1, A8 8 -> 2, A37
  6 -> 1; sweep A20 7 -> 1, A8 4 -> 1, A21 4 -> 1, B21 5 -> 1; m9-chisq-dist A30 -4 -> -1; gen-m9/3e98b26cf6713e77 A220
  3 -> 2; ...). Same distance, 11 (among them ironcalc-calc GAMMA.xlsx E23 and t1049-chisqdist A34, A36, 1 -> -1).
- Farther, 3 (ungated, 1 ulp more): m9-gamma-dist-a0p5 A22 `GAMMA.DIST(50000,0.5,1000000,FALSE)` -1 -> -2,
  t1048-gammadist A6 and B6 `GAMMA.DIST(3,2,1.5,FALSE)` -1 -> -2. No gated cell moves farther; gen-m9/a54c738c697617c3
  A29 is unchanged from main (-2 ulp).
- Visibly fixed, 1: probes/t2753-gamma-density-ratio-sweep B21 `GAMMA.DIST(1,2,1/1E-5,FALSE)` 5 -> 1 ulp. Newly visibly
  off: 0. Type or error changes: 0.
- Gated suites: 0 regressions; `make eval-baseline` drops 5 gen-m9 and 18 probes known failures.
