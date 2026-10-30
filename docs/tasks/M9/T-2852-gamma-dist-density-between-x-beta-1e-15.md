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
is Excel's own form, stage `visible`.

## Outcome
Found Excel's form between x / beta 1E-15 and 1E-10 and beyond (attempt 1 of 2 for the band, decision 0056): from shape
0.5 to 3, GAMMA.DIST's density is EXP(a LN(y))/GAMMA(a)/x*EXP(-y) with Excel's LN, EXP and GAMMA. At every switch and
sweep cell from 1E-16 to 1E-10 Excel's relative error less a times LN(y)'s rounding error is within about 1 ulp. gamma.rs
`log_form_density` (was `tiny_ratio_density`) takes it for y above 1E-15 and up to 16 (fits in [5, 20), and the shape
bounds in (0.1, 0.5] and (2.5, 5]); T-2753's form stays at y up to 1E-15, where it keeps four gated cells bit-exact.
Every form tried, with its counts, is in statistics.md ("GAMMA.DIST's density below shape 3 is a log-space form").
Shape 50 below a/100 (sweep A46:A48, B46:B48) is not found: Excel's error there is 50 times LN(y)'s plus 41 to 52 units,
and no order of the exponent in doubles or an x87 register, nor the ratio form, Loader or `rcomp`, fits (13 to 435 ulp
off); T-3219 owns those cells, with these findings recorded in statistics.md. The rest of this form's bits (y above 16,
shapes below 0.5, y up to 1E-15, a 1 to 6 ulp residue) is T-3303.

Main (9efff2847) against HEAD (the same tree before merging T-3070's quantile change), every cell of every vendored suite
(calipers, corpus-bench and poi not fetched; both binaries rebuilt from a `cargo clean` of the engine and harness):
- Fixed (bit-exact now), 44: probes/t2753-gamma-density-switch A6, A7, A22, A35, A36, B6, B7, B8, B9, B34, B37 (1 to 9
  ulp before), -ratio-sweep A11, A23, B8, B10, B11, B20, B23 (1 to 6); probes-numeric m9-gamma-dist-a0p5 A4, A10, A14,
  A20, A24, A26, A30, -a1 A2, A12, A24, A26, -a2 A2, A12, A28, m9-chisq-dist A4, A8, A12, t1049-chisqdist A6, A10, A26,
  A62 (1 to 2); gen-m9/8dd2312c96effb1b A150, 9cca17438a7e6f8a A133 (4 before), bc40d37cb4d8301d A165,
  d8266bc15533702a A230, e6cf3b20a014e6a3 A92.
- Lost bit-exact, 5 (all ungated probes-numeric): m9-chisq-dist A36 `CHISQ.DIST(5,5,FALSE)` 1 ulp, m9-gamma-dist-a1 A28
  `GAMMA.DIST(1500000,1,1000000,FALSE)` 1, m9-gamma-dist-a2 A8 `GAMMA.DIST(3,2,1,FALSE)` 1 and A18 `(6,2,2,FALSE)` 1,
  t1049-chisqdist A4 `CHISQ.DIST(0.1,2,FALSE)` 2.
- Changed, still off, 53: 35 closer (switch A20 17 -> 1 ulp, B20 23 -> 1, A23 19 -> 1, B23 26 -> 1, A9 14 -> 1, B22
  10 -> 1, A8 8 -> 2, A37 6 -> 1; sweep A20 7 -> 1, A8 4 -> 1, A21 4 -> 1, B21 5 -> 1, ...), 11 at the same distance, 7
  further: gen-m9/a54c738c697617c3 A29 `GAMMADIST(HOUR(...),2.5,2.5,FALSE)` 2 -> 6 ulp (within the 15th digit),
  m9-chisq-dist A38, A40 and t1049-chisqdist A38 1 -> 3, m9-gamma-dist-a0p5 A22 1 -> 2, t1048-gammadist A6, B6 1 -> 2.
- Visibly fixed, 1: probes/t2753-gamma-density-ratio-sweep B21 `GAMMA.DIST(1,2,1/1E-5,FALSE)` 5 -> 1 ulp. Newly visibly
  off: 0. Type or error changes: 0.
- Gated suites: 0 regressions (`make eval-baseline`).

