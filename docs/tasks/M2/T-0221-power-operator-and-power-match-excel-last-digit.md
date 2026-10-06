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
gen-m1 (T-0109 triage, oracle excel-graph 2026-10-05) adds 8 NumericUlp cells where only `^` differs:
- `gen-m1/6462bf910ef9f28f` A41 `3^-0.5`: expected 0.5773502691896258, actual 0.5773502691896257;
  A207 `0.5^-0.5`: expected 1.414213562373095, actual 1.4142135623730951 (the correctly rounded value).
- Integer exponents also differ, with base 1E-15: A33 `(-Data!E7)^3` expected -1.0000000000000001e-45,
  actual -1.0000000000000003e-45; A48 `+Data!$D9^3` and A86 `-Data!$C$3^3` are the same value; A57
  `Data!E7^3^2` expected 1.0000000000000002e-90, actual 1.0000000000000006e-90. So the fit must cover
  small integer exponents too (repeated multiplication is one candidate).
- `gen-m1/57fe331cb7611dbf` A54 `((('Q''1'!A$7^Data!B10)^("0.5"^TRUE))^10)` expected
  0.031250000000000035, actual 0.03125000000000002; A104 `('Q''1'!B9^'Q''1'!C1)` expected
  1.54289402744788e+51, actual 1.5428940274478798e+51.
T-0942 sweep: integer exponents are right-to-left square-and-multiply, 154/154 cells
(numbers.md "Integer exponents", probes-numeric/m9-fin-power); non-integer exponents remain open.
docs/excel-semantics/numbers.md has "POWER and `^`" (special cases). Excel's
results look like `exp(y*ln(x))` in double, or a similar decomposition. Fit a formulation against these cells
plus a probe sweep (excel-probe skill), as T-0914 did for the hyperbolics. Check docs/excel-semantics/numbers.md
"Excel for the web and desktop Excel differ" first: these are excel-graph values.

Code: `arithmetic::power` (POWER and `^`).

M1 exit blocker: 8 of its cells are in gen-m1 (`gen-m1/6462bf910ef9f28f`, `57fe331cb7611dbf`),
and M1 needs gen-m1 at 100%. Take it before other M2 work, or split out the gen-m1 part.

## Acceptance criteria
- [ ] A probe sweep pins the formulation. docs/excel-semantics/numbers.md entry updated with evidence.
- [ ] The 15 gen-m2 cells above disappear from eval/baseline/gen-m2.txt and the 8 gen-m1 cells from eval/baseline/gen-m1.txt, with no new F lines in any suite.
- [ ] `make check` green; reviewers approve.

## Out of scope
- -
