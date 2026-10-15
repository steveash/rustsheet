---
id: T-1563
title: "Fit Excel's SIN/COS/TAN/COT/SEC/CSC range reduction for angles near 2^27 (last digits differ)"
milestone: M5
lane: functions-math
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
SIN, COS, TAN, COT, SEC and CSC match Excel to the last bit for angles below 2^27, including near the cut.

## Context
probes/trig-huge-argument A1, A15, A29, A43, A57, A71 (`f(2^27-1)`): Excel differs from libm in the 10th-12th digit, so its argument reduction is inexact for large angles. Fit it with the fit lab (docs/research/numeric-kernel-map.md); see docs/excel-semantics/numbers.md "Trig of a huge angle". Code: `fn_sin` etc. in functions/math_and_trigonometry/mathematical.rs.

## Acceptance criteria
- [ ] The 6 known failures of probes/trig-huge-argument are fixed and removed from eval/baseline/ without regressing any other trig case.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Notes (in flight, 2026-10-15)
- Model found: Excel reduces by π₆₄ (π rounded to a 64-bit significand, x87 FLDPI), so its
  f(x) is the true f(x + kδ), k = round(x/π), δ = π − π₆₄ = -5.016557612668332e-20. With
  results correctly rounded from that, all 6 `f(2^27-1)` cells match bit for bit (checked with
  exact rationals), and SEC/CSC equal 1 / round(sin or cos of x + kδ).
- On this branch: `excel_sin_cos` / `excel_tan` (first-order corrections `sin + e·cos`, ...;
  bit-identical for k = 0). Gate: probes −6 and ironcalc-calc −17 known failures, 0
  regressions (baseline updated). Non-gate probes-numeric: 8,747 → 9,014 passing cells, but one
  new 1-ulp miss: m9-trig-reduction E53 `CSC(1000*PI()+0.25)` (4.0419725012261605 vs
  4.04197250122616), because `sin + e·cos` in doubles is not the correctly rounded sin(x + kδ).
  An FMA-compensated reciprocal fixed E53 but broke SEC D62/D69, so drop that idea.
- Next: compute sin/cos of x + kδ correctly rounded (double-double reduction of x by π₆₄ split
  Cody-Waite style, then sin(r_hi) + cos(r_hi)·r_lo), re-run gate and non-gate suites (compare
  failing sets of `rustsheet-eval run --suite probes-numeric` before/after), document the quirk
  in docs/excel-semantics/numbers.md "Trig of a huge angle", unit tests, `make check`, review.
