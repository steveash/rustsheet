---
id: T-2031
title: "ASINH and ACOSH are Excel's plain log formulas, ln(x + sqrt(x*x +- 1)), not libm"
milestone: M5
lane: functions-math
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
ASINH and ACOSH return Excel's bits by evaluating the textbook log formulas in doubles, as Excel does,
instead of libm's accurate `asinh`/`acosh`.

## Context
docs/excel-semantics/numbers.md, "Inverse hyperbolics, COTH/CSCH and the SINH/TANH thresholds: oracle
sweep (T-1005)": all 65 ASINH/ACOSH cells of probes-numeric/m9-hyperbolic equal, bit for bit in IEEE
doubles, `ln(x + sqrt(x*x + 1))` for ASINH (negative x: `-ln(-x + sqrt(x*x + 1))`) and
`ln(x + sqrt(x*x - 1))` for ACOSH. So small arguments lose precision exactly as Excel does:
`ASINH(1E-10)` = 1.000000082690371E-10 (ours 1E-10, A1), `ACOSH(1.000000000001)` = 1.414276423083533E-6
(A76). Gap triage 2026-10-19 checked the same two formulas (`math.log`/`math.sqrt`) against
ironcalc-calc/MATH_AND_TRIGONOMETRY/TRIGONOMETRIC.xlsx sheet ASINH_ACOSH_ATANH: 33 of 33 ASINH cells and
17 of 17 ACOSH cells match, including every passing one, and gen-m5/21b9ae9affd6b38c Calc!A40
(`ACOSH(2.5)` 1.5667992369724109) matches too.
Known failures that should go (eval/baseline/ironcalc-calc.txt, `ironcalc-calc/MATH_AND_TRIGONOMETRY/TRIGONOMETRIC.xlsx`):
ASINH_ACOSH_ATANH!B2, B5, B10, B14, B16, B18, B19, B21, B40, B42, B51, B52 (ASINH) and C12, C14, C48
(ACOSH): 15 lines. probes-numeric/m9-hyperbolic is not gated; its 37 failing ASINH/ACOSH cells (A1-A18,
A20, A21, A38, A39, A43, A46, A49, A52, A55, A58, A61, A64, A67, A70, A73, A76, A80, A88, A89) should pass
in the report.
Code: `fn_asinh` and `fn_acosh` in crates/rustsheet-engine/src/functions/math_and_trigonometry/mathematical.rs
(today `f64::asinh` / `f64::acosh`). Use plain `f64` `ln` and `sqrt`; the operator-level helpers in
arithmetic.rs (`extended_*`) are not part of this fit. Where `x*x` overflows (|x| above about 1.3E154) no
Excel value exists: keep today's libm result there, with a comment saying so.
This splits ASINH and ACOSH out of T-0917 (ATANH, ACOTH, COTH, CSCH and the TANH/SINH thresholds stay there).

## Acceptance criteria
- [ ] The 15 TRIGONOMETRIC.xlsx lines above leave eval/baseline/ironcalc-calc.txt (`make eval-baseline`),
      with no new F lines in any baseline.
- [ ] In `make eval`, probes-numeric/m9-hyperbolic has no failing ASINH or ACOSH cell.
- [ ] Unit tests for `ASINH(1E-10)`, `ASINH(-1E-10)` and `ACOSH(1.000000000001)` with the bit patterns above; a code
      comment cites the numbers.md entry.
- [ ] The numbers.md entry says ASINH/ACOSH are implemented (T-2031).
- [ ] `make check` green; reviewers approve.

## Out of scope
- ATANH, ACOTH, COTH, CSCH, TANH (T-0917).
- Excel's ASINH/ACOSH of arguments whose square overflows (unprobed).
