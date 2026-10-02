---
id: T-0907
title: "Engineering: CONVERT factor constants and IM* trig precision"
milestone: M9
lane: functions
status: todo
priority: P2
size: M
depends_on: []
---

## Goal
Fix the CONVERT (67 baseline lines) and COMPLEX/IM* (38 baseline lines) numeric gaps using the
Excel-cached values already in the suite.

## Context
- CONVERT (`ironcalc-calc/ENGINEERING/CONVERT.xlsx`): e.g. `CONVERT(1,B2,$B$2)` weight expected
  14593.902937206363, got 14593.9029372064; `CONVERT(0.05,..)` expected 8.30269391e-20, got
  8.302693910000001e-20; distance expected 3.0856775812815532e16, got 3.08567758128155e16. Our
  conversion constants are rounded to 15 digits or applied in a different order (value*from/to vs
  value*(from/to)); derive the table of exact factors and the multiplication order from the 67 cells.
- COMPLEXs.xlsx: IMREAL/IMAGINARY/IMABS/IMARGUMENT/IMTAN/IMCOT/IMCSC/IMSEC/IMSECH/IMLOG2 differ in the
  last 1-3 digits, e.g. `IMREAL(IMTAN(D2))` expected -1.11280735120837e-5 got -1.11280735120933e-5
  (Excel output here is 15 significant digits, so formula identity for cancellation-prone cases matters).
Code: `crates/rustsheet-engine/src/functions/engineering/`.

## Acceptance criteria
- [ ] The 67 CONVERT baseline lines in eval/baseline/ironcalc-calc.txt disappear with no new ones.
- [ ] Stretch: the COMPLEXs.xlsx lines drop (record how many; unfixed ones stay in baseline).
- [ ] Findings in docs/excel-semantics/engineering.md; `make check` green.

## Out of scope
BESSEL (T-0900), trig/hyperbolic (T-0906).
