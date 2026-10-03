---
id: T-0915
title: "SIN/COS/TAN (and SEC/CSC/COT): Excel's argument reduction near multiples of pi"
milestone: M9
lane: functions
status: todo
priority: P3
size: S
depends_on: []
---

## Goal
For arguments close to multiples of pi, Excel returns values that reflect a limited-precision pi in argument reduction (e.g. `SIN(PI())` is 1.22514845490862E-16 in Excel versus libm's 1.2246467991473532e-16). Reproduce Excel's results.

## Context
Evidence: `ironcalc-calc/MATH_AND_TRIGONOMETRY/TRIGONOMETRIC.xlsx`, sheet SIN_COS_TAN, NumericMismatch cells B44, D44, C45, D45, C46, D46, B47, D47, C48, B49, D49 (inputs A44..A49 are near/at k*pi), plus NumericClose B34/C34/D34, C36/D36, C37/D37, and COT_CSC_SEC C34 (CSC) / SEC cells. Examples: B47 `SIN(A47)` expected 1.22514845490862e-16 got 1.2246467991473532e-16; C48 `COS(A48)` expected 6.1257422745431e-17 got 6.123233995736766e-17; D45 `TAN(A45)` expected 999849864538.9548 got 999849889614.2125.
Suspected cause: x87-style reduction using a pi approximated to ~66 bits (the ratio 1.22514845/1.22464679 ~ 1.0004 matches pi's rounding error at that precision). Hypothesis only: derive the effective pi constant that reproduces all cached values (fit against the 11 mismatch cells), then apply reduction `x - k*pi_eff` only when the argument is near a multiple of pi (check where Excel and libm agree for ordinary arguments so no currently passing cells regress).

## Acceptance criteria
- [ ] The 11 NumericMismatch SIN/COS/TAN cells above plus SEC/CSC cells on the same inputs leave eval/baseline/ironcalc-calc.txt, with no new F lines (NumericClose cells may remain; list them in Outcome).
- [ ] The reduction lives in one helper shared by SIN, COS, TAN, SEC, CSC, COT.
- [ ] Quirk and evidence recorded in docs/excel-semantics/numbers.md.
- [ ] `make check` green.

## Out of scope
Hyperbolic functions (T-0914).
