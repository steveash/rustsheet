---
id: T-2982
title: "GAMMA.INV, CHISQ.INV and the other gamma inversions give Excel's values at a huge shape (T-1891 rest: E5, E15, the gen-m9 cells, the underflow rule)"
milestone: M9
lane: functions-stat
status: done
priority: P1
size: M
tier: opus
depends_on: [T-1891]
campaign: accuracy
---

## Goal
GAMMA.INV, GAMMAINV, CHISQ.INV, CHISQ.INV.RT and CHIINV give Excel's value at a huge shape or df, and the rules
T-1891 left open (the underflow rule below shape 1, the density and cdf `#NUM!` rules of T-2435) are matched or
recorded.

## Context
- T-1891 split (2026-10-28, session budget): T-1891 made the incomplete gamma pair Temme's uniform asymptotic expansion
  from a shape of 1E8 (`incomplete_gamma.rs` `temme_pair`), so every huge-shape cdf is a fixed amount of work and
  Excel's value: probes/incomplete-gamma-cap E1-E4, E8-E12, E16, E17, E20 bit-exact, E13 1 ulp. Everything else in
  T-1891's Context moves here unchanged (read docs/tasks/M9/T-1891-gamma-dist-poisson-and-the-gamma-inversions-give.md):
  - E5 `GAMMA.INV(0.5,1E300,1E-300)` = 1.0002658235505755 (ours `#NUM!`: statrs' search from a bracket of [1, 2]
    halves and doubles past its budget); E15 `CHISQ.INV(5.6221E-26,4715100000)` = 4714082845.627977 (ours
    4714346327.59633: above `REFINE_SHAPE_MAX` = 1E9 the quantile is statrs' search alone); E14, E19 now 1 ulp.
    An inversion at a huge shape can start from the normal (Wilson-Hilferty) estimate and refine on the Temme cdf,
    a bounded number of cdfs; re-check the gen-m9 cells that pin `#NUM!` above 1E9 (`GAMMAINV(0.999999,10000000000.5,
    1E+10)`, a CHISQ.INV.RT of 10000000000.5 df) before moving the cut.
  - E18 `GAMMA.DIST(1E300,1,1E-10,FALSE)` is `#NUM!` in Excel (ours 0).
  - The underflow rule (`lower_of_underflow`, backed by Excel only at shape 2, E7) probed at a shape below 1 and near 0
    (`GAMMA.DIST(1E-300,0.5,1E100,TRUE)`, `GAMMA.DIST(1E-300,1E-17,1E100,TRUE)`, `CHISQ.DIST(1E-17,1,TRUE)`).
  - The gen-m9 sweep cells of T-2255 (25 known-failure lines in eval/baseline/gen-m9.txt, listed in T-1891) and the
    T-2435 split cells (gen-m9 2801c1c11c21e134 A241, 40425341c5034eed A265; probes/t2435-dist-extremes A25, A26,
    A101, A102, and their `(...)=0` companions B101 and B102, which are `#NUM!` in Excel and TRUE in ours, so they
    pass once A101 and A102 do (T-3021); A21, A22, A33 and A34 pass since T-1891). Also gen-m9/0a614cfe974b59c3 Calc!A327
    `GAMMAINV(1-1E-10,Data!C7,0.5)` with a shape of 1E+15: Excel 500133012372525.06, ours `#NUM!` (the huge-shape
    search; T-3021). Several of the gen-m9 cells are last digits of a small-shape inversion
    (`GAMMA.INV(1E-16,0.25,...)`): leave those to T-0972 if so.
  - Excel evidence for Temme's expansion is all near x = a. Probe the tails (beyond about 3 sigma) and both sides of
    the 1E8 cut (for example `GAMMA.DIST(99990000,1E8,1,TRUE)` against shape 99999999) into
    probes/incomplete-gamma-cap; a 60-digit reference puts the kernel within 1 ulp near the centre and a few hundred
    ulps at 1E-300, the conditioning of e^(-a phi), as the series below the cut (T-1891 fidelity review).
  - `INCOMPLETE_GAMMA_WEIGHT` (lift/kinds/statistical.rs): a cdf from shape 1E8 is now one term, so its bound only
    over-charges there; re-derive it if an inversion changes.

## Acceptance criteria
Amended at the split (2026-10-28, run 2): the shapes below 2E10, the chi-square df cap and the last digits moved to
T-3070 with their cells.
- [x] probes/incomplete-gamma-cap E5 and E18 pass; E13, E14, E15, E19 named in T-3070 (E15 is the 1E9-5E9 band,
      E13, E14, E19 1 ulp each, unchanged); no loop whose length depends on argument values (the closed form is O(1)).
- [x] The underflow rule is probed below shape 1 and near 0 (A117-A122) and matched (0 at every shape).
- [x] No new known-failure lines on existing cases (gen-m9 and probes shrink by 1 and 11); the new probe's remaining
      known failures (the 63 `F` lines of probes/t2982-gamma-inv-huge-shape, new coverage) are each T-3070's, as are the
      owned gen-m9 and t2435 cells, which sit at shapes 1E9 to 1E10 or are a small-shape inversion's (T-0972).
- [x] `make check` green; reviewers approve.

## Out of scope
- The last digits at ordinary shapes (T-0971, T-0972, T-1634).

## Outcome
- GAMMA.INV and GAMMAINV from shape 2E10 are Excel's Wilson-Hilferty closed form (`incomplete_gamma`
  `wilson_hilferty_quantile`, constants fitted on probes/t2982-gamma-inv-huge-shape): within 7 ulps at shape 1E13 and
  above (A13-A41 and A78-A86 pass or are a few ulps off; ours was `#NUM!` everywhere above about 1E9), 100 to 1,500
  ulps at 1E11. probes/incomplete-gamma-cap E5 (`GAMMA.INV(0.5,1E300,1E-300)`) and E18 now pass.
- The cdf where x / beta underflows is 0 at every shape (A117-A122), the density where it overflows `#NUM!` (E18).
- Split to T-3070 (session budget and the evidence): shapes 1E9 to 2E10 (E15, A2-A4, the `#NUM!` band, A44), CHIINV and
  CHISQ.INV.RT above the 1E10 df cap (A92, A93), the 1E11 z^2 term, Temme's tails at the 1E8 cut (A144-A146), E13,
  E14, E19 and the gen-m9 cells. The underflow rule below shape 1 is probed and matched (criterion 2).
- How the form was found: c^(1/3) - 1 is 0.0000886 to 13 digits, and (x/a)^(1/3) less 1 + 0.0000886 - 1/(9a) is
  linear in z with slope s/(3 sqrt a), s = 1 - 2.2106E-5, at shapes 1E11 and 1E13 alike (A25-A77); the earlier
  reading as c times the true quantile plus a z term was the same cube expanded.
- Eval delta: known failures gen-m9 3319 -> 3318, probes 2607 -> 2596 (fixed 12, regressions 0);
  probes/t2982-gamma-inv-huge-shape 49 -> 60 of 123.

Note (2026-10-30 triage): the GAMMA.DIST/GAMMADIST `#NUM!` cells at a huge shape named above (probes/t2435-dist-extremes S!A25, A26, A101, A102 and their B companions; gen-m9 2801c1c11c21e134 A241, 40425341c5034eed A265) are owned by T-3268.
