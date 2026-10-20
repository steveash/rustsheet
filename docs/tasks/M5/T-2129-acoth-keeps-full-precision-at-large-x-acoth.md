---
id: T-2129
title: "ACOTH keeps full precision at large |x| (ACOTH(1.5E7) has 9 correct digits)"
milestone: M5
lane: functions-math
status: todo
priority: P1
size: S
tier: sonnet
campaign: visible
depends_on: []
---

## Goal
ACOTH at large |x| is within half a unit of the 15th significant digit of Excel's value. Today
it loses up to 8 digits there.

## Context
- The engine evaluates `0.5*ln((x+1)/(x-1))`, which cancels at large x:
  - `ACOTH(1.5E7)`: ours 6.666666662802374E-8, Excel 6.666666666666676E-8;
  - `ACOTH(1.5E10)`: ours 6.666667217824696E-11, Excel 6.666666666666667E-11.
- docs/excel-semantics/numbers.md has the evidence. For x = 15 to 1.5E10, Excel tracks libm
  `atanh(1/x)` within 1 ulp: bit for bit on A105 and A107–A110 of
  `probes-numeric/m9-hyperbolic`, and 1 ulp off on A101–A104 and A106.
- On the 2026-10-20 run, 14 ACOTH cells are visibly off:
  - `probes-numeric/m9-hyperbolic`: 9;
  - `ironcalc-calc/MATH_AND_TRIGONOMETRY/TRIGONOMETRIC.xlsx`: 4;
  - `gen-m5/2e604cf8d308920e` Calc!A36 `ACOTH(FACT(14))`: Excel 1.1470745597729725E-11.
- T-0917 keeps ACOTH's remaining ulp parity.

## Acceptance criteria
- [ ] For |x| above a threshold chosen from the probe cells, ACOTH evaluates `atanh(1/x)`. The
      threshold and its evidence go in numbers.md.
- [ ] All 14 cells above are within half a unit of the 15th significant digit. A105 and
      A107–A110 of m9-hyperbolic pass bit for bit.
- [ ] No ACOTH cell near 1 changes; the `ACOTH(1 + 1E-12)` cells still pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The 1-ulp cells A101–A104 and A106, and the rest of T-0917.

## Visible-parity stage
This whole task is the visible-parity stage (decision 0083).
