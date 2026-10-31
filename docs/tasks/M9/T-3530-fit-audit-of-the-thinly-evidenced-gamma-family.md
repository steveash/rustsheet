---
id: T-3530
title: "Fit audit of the thinly evidenced gamma-family FITTED constants (decision 0188 R4)"
milestone: M9
lane: functions-stat
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Every gamma-family `FITTED` constant with 6 or fewer evidence cells is either confirmed on fresh oracle evidence
(bracket narrowed, count raised), restricted or replaced where Excel disagrees inside its applied range, replaced by
accurate math where that does at least as well, or reclassified as one of Excel's own boundaries.

## Context
- Decision 0188 R4 (anti-overfitting guards): a fitted piece is not applied outside the range it was measured on.
- The constants (on `main` 744815d06): gamma.rs `HUGE_SHAPE_DENSITY_CUT`, `HUGE_SHAPE_CDF_CUT` (T-3271),
  `LOG_FORM_SHAPE_MAX`, `LOG_FORM_SHAPE_MIN` (T-2852); incomplete_gamma/huge_shape.rs `HUGE_SHAPE_MIN`,
  `BAND_UPPER_EDGE`, `BAND_LOWER_EDGE`, the closed-form density cut of `huge_shape_quantile` (T-3070);
  incomplete_gamma/mod.rs `RATIO_FORM_MIN` (T-2638), `CLOSED_FORM_QUANTILE_MAX`, `CLOSED_FORM_STEP_MIN` (T-3013;
  moved unchanged to closed_form.rs by T-3311); incomplete_gamma/density.rs `SMALL_RATIO_SHAPE_MIN` (T-3147).
- Probes: probes-numeric `fit-audit-*` (12 cases, 579 cells; 11 filled from excel-graph, `fit-audit-t3271-cdf-cut`
  pending: createSession failed four times, even on its 18-cell first half).
- Report: docs/research/fit-audit/README.md.

## Acceptance criteria
- [ ] Each constant has a verdict (a)-(d) backed by the fit-audit probes and 50-digit math, in the report.
- [ ] Verdicts (a) and (c) land with updated `FITTED` tags (quality/fitted.json via `check_fitted.py --update`);
      big changes are their own tasks.
- [ ] No gated regression and no cell newly visibly off (decisions 0083, 0111).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Constants with more than 6 evidence cells.
- T-3265's fix of GAMMA.INV at a shape below 1 (this audit only supplies evidence for it).

## Handoff
Session ended mid-audit (WIP commit). Done:
- 12 probe cases written; 11 oracle-filled (excel-graph) and scored against the engine at `main` 744815d06 and
  50-digit mpmath. Provisional verdicts are in docs/research/fit-audit/README.md.
- Clear refutations (Excel disagrees inside the applied range): `HUGE_SHAPE_DENSITY_CUT`, `HUGE_SHAPE_MIN`,
  `BAND_UPPER_EDGE`, `BAND_LOWER_EDGE`. Confirmed as Excel's own boundary: the closed-form density cut.

Left:
- Fill `fit-audit-t3271-cdf-cut` (bisect the createSession failure; try 5-cell slices).
- Score the bit-level constants (`LOG_FORM_SHAPE_MAX/MIN`, `SMALL_RATIO_SHAPE_MIN`, `RATIO_FORM_MIN`,
  `CLOSED_FORM_*`) on both sides of each bracket: build a scratch binary whose cut-points are read from the
  environment (never committed), run each case at the current value and at the bracket's other end, and count
  bit-exact cells per side. Not yet done, so none of these has a verdict.
- Second-round probes for the refuted rules (the x/a hypothesis for the density cut; the shape-min and band edges),
  then file the replacement tasks with `make new-task`. No code is changed on this branch yet.
- statistics.md entries for the new findings (drafted in the report, not yet moved).
