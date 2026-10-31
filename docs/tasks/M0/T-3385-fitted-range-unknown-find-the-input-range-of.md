---
id: T-3385
title: "FITTED range unknown: write down the input range of the five fits tagged `range unknown`, and settle EXPM1_FORM_LIMIT"
milestone: M0
lane: quality
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Every `FITTED(...)` tag in the engine names the input range its cells cover: none says `range unknown`, so decision
0188 R4.3's "not applied outside the range it was measured on" can be checked for each fit.

## Context
- T-3361 tagged the known fitted constants (`/// FITTED(T-NNNN, <n> cells, <range>)`, scripts/check_fitted.py). Five
  tags say `range unknown` because their task's evidence does not state the domain of its cells
  (crates/rustsheet-engine/src/functions/):
  - financial/financial_util.rs `irr_fallback` (T-2727, 39 cells: probes/irr-fallback-roots A1-A24 and 15 gen-m9
    cells whose flow sets have no stated domain);
  - financial/financial_util.rs `compute_xirr` stop tolerances (T-3268, 267 cells of the `xirr` and `xirr-oos` cases);
  - financial/financial_bonds/yield_solver.rs `excel_yield_newton` 1e-10 stop (T-1592, "the lab's 370 YIELD cells");
  - statistical/incomplete_beta.rs `SERIES_MIN_SHAPE_SUM` (T-2219, 232 cells; the a + b span is not recorded);
  - statistical/fisher.rs `F_QUANTILE_REFINE_P_MIN` (T-0974, 1 cell plus an eval-wide count with no cell list).
- Weak counts noted while tagging (worth correcting in the same pass if the evidence allows): `T_TAIL_P` and
  `T_STATRS_MIN_P` (t_dist.rs; probe t1531-t-tiny-p covers p 1E-300..1E-10, which does not reach the cuts at 1E-5 and
  1E-8), `HUGE_SHAPE_MIN` (only the 1E9 side counted), `T_CENTRE_C` (only the c = 0.1 cells listed).
- math_and_trigonometry/hyperbolic.rs `EXPM1_FORM_LIMIT` (1.0) is a form switch the T-2134 evidence does not bracket
  (its sweep covers only |x| < 1): it may be an untagged fit.

## Acceptance criteria
- [ ] Each of the five tags names the input range of its cells, read from the case files or computed from them; none
      says `range unknown` (`rg 'FITTED\(.*range unknown' crates` is empty).
- [ ] `EXPM1_FORM_LIMIT` is either shown to follow from math (and its doc says so) or bracketed by Excel cells (with
      `excel-probe` if needed) and tagged; quality/fitted.json updated with `--update` if the count changes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing any fitted value or adding out-of-range fallbacks (T-3364, T-3365).
