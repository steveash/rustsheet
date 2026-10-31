---
id: T-3385
title: "FITTED range unknown: write down the input range of the five fits tagged `range unknown`, and settle EXPM1_FORM_LIMIT"
milestone: M0
lane: quality
status: done
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
- T-3361 review (excel-fidelity, MINOR) asked for the same pass to:
  - decide whether statistical/f_density.rs `LARGE_DF_MIN` (16, r5-f-bigconst `BIG_MIN`) is a fitted cut and tag it
    (the probe design edge per r5-f-bigconst §4.5, or the bcmodel switch);
  - trace the f_density.rs counts to a named file: 5752 cells on `SMALL_SHAPE_OFFSET`, `STIRLING_ERROR_HALF`,
    `STIRLING_ERROR_ONE` and 4232 on `STIRLING_ERROR` and its slope and intercept (docs/research/t2464/r4-f-small-df
    says both max(d) <= 10 and "fitted at max(d) <= 30");
  - name the bracketing probe (probes/t2778-imsinh-sign-sweep, T-2797) on engineering/complex/trig.rs
    `INVERSE_EXP_LOW` and `INVERSE_EXP_HIGH`;
  - settle f_density.rs `LOG1P_SERIES_LIMIT` (0.375) and the RATE secant tolerances and IRR model in
    financial/financial_util.rs (T-2005, T-1525, T-1533): tag them or record why they are not fits.
- math_and_trigonometry/hyperbolic.rs `EXPM1_FORM_LIMIT` (1.0) is a form switch the T-2134 evidence does not bracket
  (its sweep covers only |x| < 1): it may be an untagged fit.

## Acceptance criteria
- [x] Each of the five tags names the input range of its cells, read from the case files or computed from them; none
      says `range unknown` (`rg 'FITTED\(.*range unknown' crates` is empty).
- [x] `EXPM1_FORM_LIMIT` is either shown to follow from math (and its doc says so) or bracketed by Excel cells (with
      `excel-probe` if needed) and tagged; quality/fitted.json updated with `--update` if the count changes.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing any fitted value or adding out-of-range fallbacks (T-3364, T-3365).

## Outcome
No value or behavior change: doc comments, tags and one semantics note. Method: a temporary `eprintln!` at each fitted
branch, a full `rustsheet-eval run`, the logged inputs reduced to ranges (reverted); the XIRR and YIELD sets read from
the fit lab's `xirr`, `xirr-oos` and `yield` sweeps.
- `irr_fallback` (T-2727): probe A1-A24 are 3..6 flows, |c| 0.3..400, roots -0.75 and 6.91; gen-m9's 68 fallback calls
  (the 15 cells among them) have 2..29 flows, |c| 1E-15..1E+15, 4 roots in -0.93..661, the rest `#NUM!`.
- `compute_xirr` (T-3268): the 267 cells have 2..6 flows over 0.5..124 years, |value| 5E-6..1.4E+21, roots -0.93..22.05.
- `excel_yield_newton` (T-1592): the 370 are the fit lab's `yield` set: basis 0, term 1.3..40.3 years, rate 0..0.5,
  price 40..160, redemption 50..200, yield -0.081..0.51.
- `SERIES_MIN_SHAPE_SUM` (T-2219): half-integer a + b 2..9.5 below the cut, 10.5..110 above it; no cell in (9.5, 10.5).
- `F_QUANTILE_REFINE_P_MIN` (T-0974): 61 cells reach it (count corrected from 1): 25 unrefined at p 1E-20..1.1E-16, 36
  refined at 1E-15..1E-10; no cell strictly between.
- `EXPM1_FORM_LIMIT`: not from the math; bracketed by t2178-hyperbolic-unit-sweep at 0.99505..1.0375 (exp forms miss
  `TANH(0.995054655025377)`, expm1 forms miss `SINH(1.0375)`), tagged `FITTED(T-2134, 2070 cells, ...)`;
  quality/fitted.json 80 -> 81 by `--update`. It tags an existing fit (no value change), so no new hold-out applies.
  numbers.md "SINH and TANH below 1" notes the bracket.
- Also named the bracketing probes (t2535, t2778) on `INVERSE_EXP_LOW`/`INVERSE_EXP_HIGH`.
- Not done (the optional "worth correcting" items): t_dist weak counts, f_density `LARGE_DF_MIN`/`LOG1P_SERIES_LIMIT`
  and Stirling counts, RATE secant and IRR model: follow-up T-3395.
Eval delta: none (comments only).
