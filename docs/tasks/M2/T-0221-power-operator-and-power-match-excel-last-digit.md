---
id: T-0221
title: "Power operator and POWER: match Excel last-digit results (15 gen-m2 known failures)"
milestone: M2
lane: engine
status: in-progress
priority: P1
size: M
tier: opus
depends_on: [T-1302, T-1421]
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
plus a probe sweep (excel-probe skill), as T-0914 did for the hyperbolics. These are excel-graph values. They are the
fitting target: POWER and `^` are not in numbers.md's list "Excel for the web and desktop Excel
differ in the last digits" (decision 0030 rule 4), and excel-graph is the oracle of record for
Phases 1–3 (decision 0039). The desktop confirmation of the fit is item 2 of T-1075 (M14), not a gate.

Code: `arithmetic::power` (POWER and `^`).

M1 exit blocker: 8 of its cells are in gen-m1 (`gen-m1/6462bf910ef9f28f`, `57fe331cb7611dbf`),
and M1 needs gen-m1 at 100%. Split on 2026-10-10: T-1302 (M1, sonnet) implements the verified
integer-exponent rule. On the planner's check, that rule alone clears 6 of the 8 cells (A33, A48,
A57, A86, A54, A104) and the m9-fin-power `^`/POWER cells. After T-1302, this task owns the
non-integer exponents. Two of their cells still block M1: 6462bf910ef9f28f A41 `3^-0.5` and
A207 `0.5^-0.5`. Take those first.
Split again on 2026-10-12: T-1421 (M1, opus) owns negative non-integer exponents, including those
two gen-m1 cells and the `1/(x^|y|)` lead below. This task keeps positive non-integer exponents.
Lead to fit (a hypothesis, not verified): for a negative exponent, Excel's value equals
`1/(x^|y|)`. That gives `1/(3^0.5)` = 0.5773502691896258, `1/(0.5^0.5)` = 1.414213562373095 and,
for gen-m2 1b6ceed7489e0800 A62, `1/(2^0.5)` = 0.7071067811865475. All three are Excel's values,
and correctly rounded `powf` misses all three. Positive non-integer exponents, such as A155
`100^0.3`, are not explained by this lead.

More cells (T-1289's gen-m9 suite, excel-graph, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
- gen-m9/b5352ebdb9c5f1a1 Calc!A1 `=DROP(((Data!$C$10/Data!D3)+(Data!B3^Data!B3)),..)` (3.14159^3.14159): Excel 36.46195209318106, engine 36.461952093181075.
- gen-m9/b5352ebdb9c5f1a1 Calc!A16 `=(AVERAGE(WEEKNUM(46206,1))^Data!C4)` (27^-2.5): Excel 0.0002639918926335737, engine 0.00026399189263357376.

Update (T-1319, 2026-10-12): non-integer exponents are now `exp(y·ln x)`, see numbers.md "Non-integer exponents". Two items remain here:
- gen-m2/fa437879253827be A23;
- `(-8)^(-1/3)`, which is -0.5 in Excel and -0.5000000000000001 in ours (probes-numeric/power-non-integer-sweep A183).
- `(-0.1)^-401` is unprobed. The positive power underflows, so numbers.md's rule predicts #DIV/0!; ours gives #NUM!, because `non_integer_power` of a negative base is NaN.

## Acceptance criteria
- [ ] A probe sweep pins the formulation. docs/excel-semantics/numbers.md entry updated with evidence.
- [ ] The gen-m2 cells above that T-1302 did not clear disappear from eval/baseline/gen-m2.txt, with no new F lines in any suite (gen-m1 A41 and A207 are T-1421's).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Integer exponents: T-1302.
- Negative non-integer exponents and gen-m1 A41/A207: T-1421.
