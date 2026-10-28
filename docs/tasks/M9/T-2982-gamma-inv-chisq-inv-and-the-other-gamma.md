---
id: T-2982
title: "GAMMA.INV, CHISQ.INV and the other gamma inversions give Excel's values at a huge shape (T-1891 rest: E5, E15, the gen-m9 cells, the underflow rule)"
milestone: M9
lane: functions-stat
status: in-progress
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
    A101, A102; A21, A22, A33 and A34 pass since T-1891). Several of the gen-m9 cells are last digits of a small-shape inversion
    (`GAMMA.INV(1E-16,0.25,...)`): leave those to T-0972 if so.
  - Excel evidence for Temme's expansion is all near x = a. Probe the tails (beyond about 3 sigma) and both sides of
    the 1E8 cut (for example `GAMMA.DIST(99990000,1E8,1,TRUE)` against shape 99999999) into
    probes/incomplete-gamma-cap; a 60-digit reference puts the kernel within 1 ulp near the centre and a few hundred
    ulps at 1E-300, the conditioning of e^(-a phi), as the series below the cut (T-1891 fidelity review).
  - `INCOMPLETE_GAMMA_WEIGHT` (lift/kinds/statistical.rs): a cdf from shape 1E8 is now one term, so its bound only
    over-charges there; re-derive it if an inversion changes.

## Progress (run 1, 2026-10-28; session budget, branch pushed in flight)
- Probe probes/t2982-gamma-inv-huge-shape (123 cells, excel-graph) and docs/excel-semantics/statistics.md "GAMMA.INV and
  CHISQ.INV at a huge shape": the quantile is close to true up to shape 5E9, mostly `#NUM!` at 7E9-1E10 (17 of 18 p at
  1E10), and from 2E10 to 1E300 the true quantile times c = 1.0002658235505755 at p = 0.5, any scale. In the tails it
  is not exactly c·q: at shape 1E13, p = 1E-300 (A24) Excel is 12,972 below c·Q(p, a) and 2,600 above Q(p, a·c); every
  gen-m9 huge-shape cell (A327, A341, A239, A338, A296) and E5 is within about 1E-11 of c·q, so c·q alone is not
  visibly right in the tails. The residual is linear in z: at shape 1E13, Excel - c·Q(p, a) = -1.10726E-4 · z · sqrt(a)
  for p = 1E-300 ... 0.001 (A24-A30; constant to 6 digits, z the normal quantile of p), and the (z^2 - 1)/3 skew term
  is the true gamma's (a plain normal would drift by 456 at z = -37). So Excel's centre is c·a and its spread
  1.0001551·sqrt(a): as a gamma quantile, Q(p, a·1.00022144)·1.0000443, neither factor obviously meaningful. Check the
  same at shape 1E11 (A60-A77; the bracket of ±45 sqrt(a) in the scratch solver fails at 1E-300, narrow it). Next: find what produces c (it is the inversion's: the cdf has no factor) — a shifted
  start, a transformed search variable, or a mapped shape; fit A24-A77 (two p sweeps) to it. scratch tools: mpmath with
  Temme's C_0..C_3 for the reference quantile (mpmath's gammainc does not converge at a = 1E13).
- Done on the branch: the cdf where x / beta underflows is 0 at every shape (A117-A122; was 1 below a shape of about
  1E-15), and the density where x / beta overflows is `#NUM!` (E18). E18, A118, A119 pass.
- Not probed yet: the density #NUM! cut and the cdf cut near 1E76 (excel-graph refused the workbooks holding
  `GAMMA.DIST(2,1E40..1E100,1E10,TRUE)` and densities at shapes 1E10-1E11; try one cell per workbook). Temme's tails at
  the 1E8 cut: A144-A146 (shape 99999999, the series side) are about 1E-15 relative off, A145 (shape 1E8) too.

## Acceptance criteria
- [ ] probes/incomplete-gamma-cap E5, E15, E18 pass, E13, E14, E19 bit-exact or named in the Outcome with their ulp; no
      loop whose length depends on argument values beyond the T-1872 caps.
- [ ] The underflow rule is probed below shape 1 and near 0 and matched or recorded.
- [ ] The owned gen-m9 and t2435 cells are fixed or handed on with a reason; no new known-failure lines.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The last digits at ordinary shapes (T-0971, T-0972, T-1634).
