---
id: T-0221
title: "Power operator and POWER: match Excel last-digit results (15 gen-m2 known failures)"
milestone: M2
lane: engine
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
`^` and POWER produce Excel's double for non-integer and large exponents, where Excel differs from
correctly rounded `powf` in the last digits.

## Context
gen-m2 (T-0203): 15 NumericUlp/NumericClose cells where the only difference is a `^`. Evidence (excel-graph):
- `gen-m2/1b6ceed7489e0800` Calc!A62 `=(2^(-0.5))`: expected 0.7071067811865475, actual 0.7071067811865476 (the correctly rounded value).
- `gen-m2/f0d1bacec5de8146` Calc!A196 `=(MAX(...)^100)` with base 100: expected 1.0000000000000005e+200, actual 1e+200.
- `gen-m2/a6d4377c7b1b92cd` Calc!A155 `(100^0.3)`: expected 3.9810717055349727, actual 3.9810717055349722.
- `gen-m2/b4392a6514a36ade` Calc!A50 `(Data!A$3^Data!E2)`: expected 3.1622776601683885e-38, actual 3.16227766016838e-38.
- Other cells: 1b6ceed7489e0800 A387, 629e6e8b76fab8b9 A133, 893302d6c5b99e2e A299, 942bdade381b7537 A83,
  caa981b0753f455c A108, d0c29dd5c530e28c A192 A344, dd99be1f2454159f A22, ef5c55a81783b7d5 A33,
  f7e6d351c8538ef5 A271, fa437879253827be A23.
docs/excel-semantics/numbers.md has "POWER and `^`" (special cases) but nothing on accuracy. Excel's
results look like `exp(y*ln(x))` in double, or a similar decomposition. Fit a formulation against these cells
plus a probe sweep (excel-probe skill), as T-0914 did for the hyperbolics. Check docs/excel-semantics/numbers.md
"Excel for the web and desktop Excel differ" first: these are excel-graph values.

Code: `arithmetic::power` (POWER and `^`).

## Acceptance criteria
- [ ] A probe sweep pins the formulation. docs/excel-semantics/numbers.md entry updated with evidence.
- [ ] The 15 cells above disappear from eval/baseline/gen-m2.txt with no new F lines in any suite.
- [ ] `make check` green; reviewers approve.

## Out of scope
- -
