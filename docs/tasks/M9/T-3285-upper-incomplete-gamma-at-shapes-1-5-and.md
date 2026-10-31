---
id: T-3285
title: "Upper incomplete gamma at shapes 1.5 and 2.5: CHISQ.DIST.RT(10,5) and CHIDIST(10,5) are 6 ulp off where Excel is correctly rounded (accuracy)"
milestone: M9
lane: functions-stat
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
CHISQ.DIST.RT, CHIDIST and the other upper-tail callers of the incomplete gamma are correctly rounded (so Excel's
double) at small half-integer shapes with x a few times the shape, where Excel is within half an ulp of the 50-digit
value and ours is 5 to 18 ulp off.

## Visible-parity stage
`accuracy`: a 50-digit reference (mpmath `gammainc(a, x, inf, regularized=True)`) puts Excel within 0.3 ulp at
`CHISQ.DIST.RT(10,5)` and ours 5.7 ulp below it, so the fix is a more accurate kernel, never a fit to Excel's bits.
Target: the four cells listed below within half a unit of the 15th significant digit (bit-exact expected), with no cell of
any suite regressing.

## Context
- Gap triage 2026-10-30 (`make eval` on origin/main 73b187224; `scripts/exit_owners.py --all-off` lists the four cells
  with no open owner). Q(2.5, 5), in ulps of Excel's value against mpmath at 60 digits:
  - `CHISQ.DIST.RT(10,5)` / `CHIDIST(10,5)`: Excel 0.07523524614651218 (+0.3), ours 0.0752352461465121 (-5.7).
- The same shape class, owned by deferred tasks (fix them too if the kernel change reaches them, and say so in the
  Outcome; do not take them over):
  - probes-numeric/m9-chisq-dist-rt A14 `CHISQ.DIST.RT(6,3)` = Q(1.5, 3): Excel 0.11161022509471255 (-0.7), ours +18.3
    (T-2367, which notes it as "the one cdf cell left where Excel is accurate and ours is not, at a = 1.5");
  - gen-m9/85f48d5cdfb24555 Calc!A84 `CHIDIST(40,3)` = Q(1.5, 20): Excel +1.2, ours -4.8 (T-2695).
- Path: `statistical/incomplete_gamma/mod.rs` `pair_with` with `gamma_prefactor`'s direct branch below
  `DIRECT_SHAPE_MAX` = 20 (x.powf(a) * exp(-x) / libm::tgamma(a)), then the continued fraction for Q. At a half-integer
  shape Q has a closed form, Q(1/2 + n, x) = erfc(sqrt x) + e^-x sum_{j=1..n} x^(j - 1/2) / Gamma(j + 1/2), which a
  mpmath unit test can check; find which factor loses the ulps (tgamma at a half-integer, the prefactor product or
  the continued fraction's stopping rule) before choosing a remedy.
- Excel's deep upper tail (x well above a) is the inaccurate side and belongs to T-3269; keep that region's results
  unchanged unless they move toward Excel.

Cells this task owns:
- probes-numeric/m9-chisq-dist-rt.case.json (1): Calc!A19
- probes-numeric/m9-legacy-chidist.case.json (2): Calc!A45, A46
- probes-numeric/t1049-chisqdistrt.case.json (1): Calc!A19

## Acceptance criteria
- [ ] The four cells above match Excel (`make eval-show` on each case; probes-numeric is ungated).
- [ ] A unit test in `incomplete_gamma/tests.rs` checks Q at shapes 0.5, 1.5, 2.5 and 3.5 against 50-digit values for x
      from a to 30a, within 1 ulp.
- [ ] No Excel-computed cell of CHISQ.DIST.RT, CHIDIST, CHISQ.DIST, CHISQ.TEST, GAMMA.DIST, GAMMADIST, POISSON or
      POISSON.DIST goes from pass to fail (main-vs-HEAD per cell); no new line in eval/baseline/.
- [ ] docs/excel-semantics/statistics.md "Incomplete gamma: accurate kernel (T-0971)" cites the four cells as evidence
      that Excel is correctly rounded there.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The deep upper tail where Excel is 10 to 405 ulp off (T-3269, T-3289). The near-mode window at large shapes (T-3286).
  The shapes below 20 at a tiny x / a (T-3219).

## Outcome
- Cause: the forward (modified Lentz) continued fraction, 5.1 ulp at Q(2.5, 5) and 16 at Q(1.5, 3); the prefactor
  (`powf * exp / tgamma`) was within 1.3. `incomplete_gamma/half_integer.rs` takes Q at a half-integer shape in
  double-double: the prefactor sqrt(x) e^-x / sqrt(pi) * prod 2x / (2j - 1) (exact in the shape) and the fraction evaluated
  backward from twice the forward depth plus 8, rounded once. The unit test checks it against 50-digit mpmath at shapes
  1/2, 3/2, 5/2, 7/2 for x from a to 30a: 44 of 44 correctly rounded. 1/sqrt(pi) moved from bratio/bgrat.rs to
  half_integer_beta.rs (one definition).
- Attempt 1 applied it to CHISQ.DIST.RT for every x >= a and lost 3 gated cells: `CHIDIST(5,5)` and `CHIDIST(7,7)`
  (x = a, where Excel is the series' 1 - P bit for bit) and `CHIDIST(42,7)` = Q(3.5, 21), where Excel is 1 ulp above the
  correctly rounded Q. Excel is not correctly rounded at half-integer shapes in general: on the fraction's range up to
  x = 30 the correctly rounded Q is Excel's at 9 of 92 cells, the old kernel at 7, Excel's ERFC plus the series at 9.
  Attempt 2 (shipped) applies the accurate Q only on [a + 1, 3a] (`ACCURATE_MAX_RATIO`,
  `FITTED(T-3285, 17 cells, x/a 1.05..13.3)`; quality/fitted.json via `scripts/check_fitted.py --update`).
- Eval, main (report of 2026-10-31, 1248554a; no statistical change since) vs HEAD, per cell, CHI*/POISSON/GAMMA*:
  - gated fixed 1: ironcalc-calc STATISTICAL/CHISQ.xlsx G3 (11 ulp to 0); closer 1: G4 (5 to 1 ulp, now within 15
    digits); farther 2: gen-m9/320d51765af9b6b7 A249 and 7674c89c824aca43 A332 (`CHIDIST(..,5)`, 2 to 3 ulp, both still
    within 15 digits); lost bit-exact 0; newly visibly off 0; visibly fixed 2 (G3, G4).
  - ungated (probes-numeric): fixed 4, the cells this task owns (`CHISQ.DIST.RT(10,5)` and `CHIDIST(10,5)`,
    m9-chisq-dist-rt A19, m9-legacy-chidist A45, A46, t1049-chisqdistrt A19); closer 1, m9-chisq-dist-rt A14
    `CHISQ.DIST.RT(6,3)` (T-2367's cell, 19 to 1 ulp; Excel is -0.7 ulp from the truth, so not bit-exact).
  - `CHIDIST(40,3)` (T-2695) is at x/a 13.3, outside the applied range: unchanged.
  - eval/baseline: ironcalc-calc one line fewer (G3).
- Decision 0188 R4: the only fitted piece is the 3a cut, chosen on attempt 1's 17 changed gated cells; it is applied
  only inside that measured range (outside, the old kernel). The accurate Q is within 1e-9 of the math everywhere
  (Gate A envelope). No hold-out split: the kernel is not fitted to Excel's bits.
- docs/excel-semantics/statistics.md "Incomplete gamma: accurate kernel (T-0971)" records the cells and why the range
  is narrow. T-3269 found Excel's own form past x = 30 (the Stirling ratio prefactor), which also gives
  `CHISQ.DIST.RT(6,3)` and G4 bit for bit; that suggests the [a + 1, 3a] cells are that form too, a lead for T-3269's
  follow-up rather than this task.
