---
id: T-2982
title: "GAMMA.INV, CHISQ.INV and the other gamma inversions give Excel's values at a huge shape (T-1891 rest: E5, E15, the gen-m9 cells, the underflow rule)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
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

## Acceptance criteria
- [ ] probes/incomplete-gamma-cap E5, E15, E18 pass, E13, E14, E19 bit-exact or named in the Outcome with their ulp; no
      loop whose length depends on argument values beyond the T-1872 caps.
- [ ] The underflow rule is probed below shape 1 and near 0 and matched or recorded.
- [ ] The owned gen-m9 and t2435 cells are fixed or handed on with a reason; no new known-failure lines.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The last digits at ordinary shapes (T-0971, T-0972, T-1634).
