---
id: T-3012
title: "Probe GAMMA.INV and GAMMA.DIST at a shape below 1 and p near 1E-16 (gen-m9 984cba67bbe6972b A43: Excel 4.1E-15 above the exact quantile)"
milestone: M9
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence (excel-graph) shows how GAMMA.INV forms a lower-tail quantile at a shape below 1 and a tiny p. The
evidence also shows whether that quantile is the crossing of Excel's own GAMMA.DIST cdf, so T-3013 can match it
without guessing.

## Context
- Gap triage 2026-10-28 (origin/main 1a0b616f): gen-m9/984cba67bbe6972b.case.json Calc!A43,
  `GAMMA.INV(1E-16,ABS(TIMEVALUE("6:00")),Data!A10)` = `GAMMA.INV(1E-16,0.25,1E+15)`, is visibly off. It is a
  known-failure line in eval/baseline/gen-m9.txt. No open task owned it: T-1891 listed it, and T-2982 hands such
  small-shape cells to T-0972, but T-0972 does not list it.
  - Excel 6.749697893111757E-50, ours 6.74969789311173E-50.
  - The exact quantile is (p·Γ(1.25))^4·1E15 = 6.7496978931117296E-50, from a 40-digit Decimal evaluation. The
    series' second term is about 1E-64 relative, so it does not matter at this x. Ours is correctly rounded.
  - Excel is 4.1E-15 relative high, which is about 1E-15 relative in P at a = 0.25. That fits a few-ulp error in
    Excel's cdf, for example x^a by EXP(a·LN x) or a GAMMALN-based Γ(a + 1).
  - Recorded in docs/excel-semantics/statistics.md ("Excel's CHISQ.INV and GAMMA.INV are exact in the lower tail"
    bullet, T-3012 note).
- Engine: `incomplete_gamma/mod.rs` `gamma_quantile` (the lower tail inverts `lower_series`).
- Probe with the `excel-probe` skill into a new `probes/t3012-gamma-inv-small-shape` case:
  - `GAMMA.INV(p,a,1)` and `GAMMA.INV(p,a,1E15)` for a in {0.05, 0.1, 0.25, 0.5, 0.75, 0.9} and p in {1E-300, 1E-100,
    1E-30, 1E-16, 1E-10, 1E-5}. The scale pair tests whether the scale is a plain final product.
  - `GAMMA.DIST(x,a,1,TRUE)` at Excel's quantile of each row, and at that quantile one ulp either way (the 17-digit
    literals the first fill returns). This needs a second fill.
  - `GAMMA.DIST(x,0.25,1,TRUE)` against `EXP(0.25*LN(x))/EXP(GAMMALN(1.25))` and `x^0.25/GAMMA(1.25)` at the same x,
    to name Excel's cdf form.

## Acceptance criteria
- [x] probes/t3012-gamma-inv-small-shape exists with excel-graph expectations (canary-verified, `oracle.json`
      sidecar). Its known-failure lines are recorded with `make eval-baseline`, so no case of any other suite moves.
- [x] docs/excel-semantics/statistics.md records, with cell citations, whether each GAMMA.INV value is the crossing
      of Excel's GAMMA.DIST and which cdf form matches Excel's GAMMA.DIST at those x.
- [x] T-3013's Context is updated with the result.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing the engine (T-3013).
- Huge shapes (T-2982) and ordinary-shape last digits (T-0972).

## Outcome
Two probe cases (probes/t3012-gamma-inv-small-shape, probes/t3012-gamma-dist-at-quantile) with excel-graph values;
findings in docs/excel-semantics/statistics.md (Excel's GAMMA.INV is not the crossing of its GAMMA.DIST at shapes
0.1, 0.25, 0.75, 0.9) and T-3013's Context. Eval delta: two new probe cases, known-failure lines added to
eval/baseline/probes.txt only; no other suite moved. Five unrelated probe cases failed to fill (graph
unsupportedWorkbook), left pending.
