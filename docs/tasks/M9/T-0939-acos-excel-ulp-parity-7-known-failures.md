---
id: T-0939
title: "ACOS: Excel ulp parity (7 known failures)"
milestone: M9
lane: functions-math
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
ACOS matches Excel bit-for-bit on the evidenced inputs.

## Context
libm `acos` is 1 ulp off Excel on several inputs (ASIN and ATAN are already 65/65):
- `poi/48539.xlsx` (excel-graph, T-0019) Instance1!E23 `=ACOS(B20)` with B20 = 0.5: expected
  1.0471975511965976 (the double nearest pi/3), actual 1.0471975511965979.
- `poi/FormulaEvalTestData_Copy.xlsx` EverythingTests!Q104 `ACOS("0.5")` (same values), I104
  `ACOS(C17)` expected 1.5708063267948966, actual 1.5708063267948968.
- `ironcalc-calc/MATH_AND_TRIGONOMETRY/TRIGONOMETRIC.xlsx` ASIN_ACOS_ATAN!C10 expected
  1.4474840516030245 (actual ...247), C18 expected 1.5707963267938965 (actual ...967), C40 expected
  0.9220766484543185 (actual ...186), and C15 (NumericClose) expected 1.4142136208761258e-05, actual 1.4142136208911564e-05.
For 0.5, `atan2(sqrt(1-x*x), x)`, `2*atan(sqrt((1-x)/(1+x)))` and `pi/2 - asin(x)` all give the
expected value. Fit one formulation against all 7 cells, including C15 near x = 1, where
cancellation in `1-x` matters.
T-1797 found that Excel's ACOS is exactly `PI()/2 - ASIN(x)` in all 240 ACOS cells of
probes/asin-atan-sign-source (columns H and I), with ASIN being `fn_asin`'s kernel (numbers.md "ASIN's formulation").
Reuse that kernel; those 83 H/I known failures should leave the probes baseline too.

## Acceptance criteria
- [ ] The 6 NumericUlp lines leave eval/baseline (poi.txt and ironcalc-calc.txt) with no new F lines;
      C15 leaves too, or the Outcome records its residual.
- [ ] Formulation and evidence recorded in `docs/excel-semantics/numbers.md`.
- [ ] `make check` green; reviewers approve.

## Out of scope
ACOSH (T-0917). SIN/COS argument reduction (T-0915).
