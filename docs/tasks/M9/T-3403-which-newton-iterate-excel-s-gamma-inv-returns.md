---
id: T-3403
title: "Which Newton iterate Excel's GAMMA.INV returns once the steps from the leading-factor root cycle (A169, A338, A355 left by T-3311)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-3311]
campaign: visible
---

## Goal
The three gated gen-m9 cells T-3311 left visibly off reach Excel's value within half a unit of the 15th digit, by a
rule for which Newton iterate from the leading-factor root Excel returns, or the Outcome lists the rules tried with
their counts (decision 0056).

## Context
T-3311 found that Excel's lower-tail GAMMA.INV/CHISQ.INV at a shape above 1 is Newton steps from the leading-factor
root on the leading-factor series form, a staircase in x (docs/excel-semantics/statistics.md, "Deep lower tail at a
shape above 1", T-3311 paragraphs; probes-numeric/t3311-gamma-inv-lower-tail-cdf, t3311-fresh-sweep). The steps
cycle between two or three levels of the staircase, and which iterate Excel returns is not pinned. The engine
(`incomplete_gamma::closed_form::excel_newton_quantile`) keeps the first iterate whose cdf is nearest p, only where
the form at the root is within 1E-8 of p (`NEWTON_START_MAX`). A wider cut (1E-6, tied on T-3311's hold-out train
part) would fix A169 and A338 but moves gen-m9/c18dbf21f11093f3 A107 `CHISQ.INV(POISSON.DIST(9,100,FALSE),8)`
(p = 1.025153212086874E-31, shape 4) one ulp off: no iterate is Excel's value there.

Cells (gen-m9, excel-graph):
- gen-m9/a612e89321a6df9c.case.json (1): Calc!A169 `CHISQ.INV(1E-20,6)`, form at the root 2.9E-7 off p;
- gen-m9/a5828d3925575713.case.json (1): Calc!A338 `GAMMAINV(1E-16,2.5,TRUE)`, 4.6E-7 off (the second iterate is
  Excel's bit for bit);
- gen-m9/c2974c061384fdd9.case.json (1): Calc!A355 `CHISQ.INV(1E-20,10)`, 2.2E-4 off, where the steps are no better
  than the search.

Leads:
- The 1-ulp residue of the form against Excel's GAMMA.DIST (T-3242) moves the levels; at A107 the nearest iterate is
  1 ulp above Excel.
- A probe of Excel's GAMMA.DIST at each Newton iterate of A107, A169 and A355 (built with formulas, never 16-digit
  literals: statistics.md T-3060 "Evidence trap") would show whether Excel's own form puts its value on a level the
  engine's model does not.
- The step's density may not be the engine's `gamma_density`: T-3311 tried the form's own derivative and
  `EXP((a-1)*LN(x)-x-GAMMALN(a))`, with the same iterates at the probe points.

## Acceptance criteria
- [ ] A169, A338 and A355 are within half a unit of Excel's 15th digit, or the Outcome lists the rules tried with
      their bit-exact and visible counts on a hold-out split (decision 0188 R4).
- [ ] No gated cell regresses (A107 included); no new known-failure line in eval/baseline.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The deep tail below p = 1E-12 and shapes above 5, where the root test is T-3294's question (its Context has
  T-3311's fresh-sweep evidence).
