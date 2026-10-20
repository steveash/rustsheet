---
id: T-2034
title: "EFFECT and NOMINAL raise to their power with Excel's ^ kernel (NOMINAL_EFFECT.xlsx, fin-argument-modes H125)"
milestone: M9
lane: functions-financial
status: todo
priority: P1
size: S
tier: opus
campaign: visible
depends_on: []
---

## Goal
EFFECT and NOMINAL return Excel's bits by computing `(1 + r/n)^n` and `(1 + e)^(1/n)` the way Excel's
`^` operator does, instead of libm `powf`.

## Context
Known failures (all NumericUlp/NumericClose): ironcalc-calc/FINANCIAL/NOMINAL_EFFECT.xlsx Sheet1!C8,
D2, D3, D4, D17, E4, E15, E17, F2, F3, F4, F8, F17 in eval/baseline/ironcalc-calc.txt (13 lines,
Excel-cached), and probes/fin-argument-modes Sheet1!H125 `EFFECT(0.05,4)` (Excel 0.050945336914062445,
ours 0.05094533691406222) in eval/baseline/probes.txt. Columns: C = NOMINAL(A,B), D = EFFECT(A,B),
E = NOMINAL(D,B), F = EFFECT(C,B), so E and F also depend on our own D and C.

Fit (gap triage 2026-10-19, scratch Python in IEEE doubles):
- EFFECT: `(1 + r/n)^n - 1` with the power by right-to-left square-and-multiply on the floored integer
  `n` (docs/excel-semantics/numbers.md "Integer exponents: right-to-left square-and-multiply") matches
  all 10 numeric D cells and H125; libm `pow` (ours) matches 6 of the 10.
- NOMINAL: `((1 + e)^(1/n) - 1) * n` with Excel's `^` rules for the exponent 1/n matches all 10
  numeric C cells: n = 1 is an integer power (exact), n = 2 is the square root, and the other n go
  through `exp(ln(1+e) * (1/n))` (C8, n = 4: 0.4267276788012868, which libm `pow` misses with
  ...859). The engine's `^` kernel already implements these rules: `arithmetic::power` (and
  `power_with_libm_exp`) in crates/rustsheet-engine/src/arithmetic.rs.
Code: `fn_effect` and `fn_nominal` in crates/rustsheet-engine/src/functions/financial/rate_conversions.rs.
Call the existing power helper; do not write another square-and-multiply. The engine's helper rounds
each product twice (`extended_product`) where the scratch fit used plain doubles: try `power` first,
then `power_with_libm_exp`, and keep the one that clears the lines below. Map an `Err` from the helper
to the error EFFECT/NOMINAL return today for an out-of-range result (`rate_result`).

## Acceptance criteria
- [ ] The 13 NOMINAL_EFFECT.xlsx lines leave eval/baseline/ironcalc-calc.txt and fin-argument-modes H125
      leaves eval/baseline/probes.txt (`make eval-baseline`), with no new F lines in any baseline.
- [ ] Unit tests for D2 `EFFECT(0.5,10)` = 0.6288946267774416 and C8 `NOMINAL(0.5,4.5)` = 0.4267276788012868.
- [ ] A line in docs/excel-semantics/financial.md records that EFFECT and NOMINAL use Excel's `^`, citing D2, C8, H125.
- [ ] `make check` green; reviewers approve.

## Out of scope
- If neither helper clears every line, stop and hand the task up (work-loop step 5b) with the cells
  each one misses; do not try other formulations.

## Visible-parity stage
Decision 0083: this task is done when every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 must be exactly 0). Criteria above that ask for bit-exact or ulp-level matches and are still unmet then move to a follow-up task (no campaign key), named in the Outcome. Regressions that stay within 4 ulp follow decision 0083's allowance.

Visibly-off cells on the 2026-10-20 eval run (`main` 35a5e71), blamed on this function rather than on glue around it: 4 in total.
- EFFECT: 3 cells (13-14 digits right 3, 10-12 0, 6-9 0, fewer or 0 vs nonzero 0); most in `ironcalc-calc/FINANCIAL/NOMINAL_EFFECT.xlsx` 2; `probes/fin-argument-modes.case.json` 1. e.g. `EFFECT(C2,B2)` Excel 0.5000000000000013, ours 0.5000000000000004
- NOMINAL: 1 cells (13-14 digits right 1, 10-12 0, 6-9 0, fewer or 0 vs nonzero 0); most in `ironcalc-calc/FINANCIAL/NOMINAL_EFFECT.xlsx` 1. e.g. `NOMINAL(A8,B8)` Excel 0.4267276788012868, ours 0.4267276788012859

Against a 50-digit reference, Excel is close to correctly rounded for this family and the engine is the inaccurate side, so an accurate kernel (not a fit) should reach this stage.
