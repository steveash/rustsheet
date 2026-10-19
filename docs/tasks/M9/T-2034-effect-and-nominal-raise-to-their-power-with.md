---
id: T-2034
title: "EFFECT and NOMINAL raise to their power with Excel's ^ kernel (NOMINAL_EFFECT.xlsx, fin-argument-modes H125)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: sonnet
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
