---
id: T-3285
title: "Upper incomplete gamma at shapes 1.5 and 2.5: CHISQ.DIST.RT(10,5) and CHIDIST(10,5) are 6 ulp off where Excel is correctly rounded (accuracy)"
milestone: M9
lane: functions-stat
status: todo
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
