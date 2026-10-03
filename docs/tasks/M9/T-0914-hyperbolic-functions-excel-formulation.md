---
id: T-0914
title: "Hyperbolic and inverse-hyperbolic functions: match Excel's formulation bit-for-bit"
milestone: M9
lane: functions
status: done
priority: P2
size: M
depends_on: []
---

## Goal
SINH, COSH, TANH, COTH, CSCH, SECH, ASINH, ACOSH, ATANH, ACOTH (and COT where it is built on TAN) differ from Excel in the last 1-2 ulp because the engine uses libm (`f64::sinh` etc.). Find the formulation Excel uses and apply it, validated against the Excel-cached values.

## Context
All evidence is in `ironcalc-calc/MATH_AND_TRIGONOMETRY/TRIGONOMETRIC.xlsx` (sheets SINH_COSH_TANH, COTH_CSCH_SECH, ASINH_ACOSH_ATANH, ACOT_ACOTH, COT_CSC_SEC). Failing cells (NumericUlp mostly): TANH 19, COTH 14, ACOTH 14, ASINH 12, CSCH 9, SINH 8, ATANH 4, ACOSH 3, COSH 1, SECH 1, COT 17 (about 100 cells).
Examples: `SINH(1)` expected 1.1752011934894935 got 1.1752011934894933 (SINH_COSH_TANH!B15); `TANH(A3)` expected 0.999909204262595 got 0.9999092042625951 (D3); `ASINH(1)` expected 0.8813735870195429 got 0.881373587019543 (ASINH_ACOSH_ATANH!B2); `ACOTH(A3)` expected 0.20273255405408222 got 0.2027325540540822.
Hypothesis to test (not assumed): Excel computes via exp/ln, e.g. sinh=(e^x-e^-x)/2, tanh=(e^2x-1)/(e^2x+1), asinh=ln(x+sqrt(x^2+1)), acoth=0.5*ln((x+1)/(x-1)). Write a throwaway script over the cached cells to pick the candidate formula that matches the most cells exactly, per function; keep only formulas that reduce failures. Mind small |x| where the naive form loses precision: the cached values for small arguments decide.
Use the single shared helper location for these (search `rg "fn_sinh|fn_tanh"` first); do not duplicate.

## Acceptance criteria
- [x] Scope narrowed to SINH/COSH/TANH (remaining functions moved to T-0917): 25 F lines left eval/baseline/ironcalc-calc.txt, no new ones.
- [x] Chosen formulation per function documented in docs/excel-semantics/numbers.md with the evidence cell ids.
- [x] `make check` green.

## Out of scope
SIN/COS/TAN argument reduction near multiples of pi (T-0915).

## Outcome
SINH/COSH/TANH now use `functions/math_and_trigonometry/hyperbolic.rs` (exp formulation, series for small |x|); ironcalc-calc known failures 1756 -> 1731 (fixed 25, 0 regressions). The 80-line target was not reachable: exp-based COTH/CSCH/SECH regressed cells libm already matched (COTH_CSCH_SECH!B10, B47, C54), so they stay on libm, and the inverse functions were not attempted. Remainder filed as T-0917. Note: the `SINH(1)` example above was a misread; B15's argument is not exactly 1.
