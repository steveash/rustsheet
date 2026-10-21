---
id: T-2240
title: "Fit Excel's expm1 under SINH and TANH below 1 (PI()/4, 0.9999999999, 0.88)"
milestone: M5
lane: functions-math
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-2134]
---

## Goal
SINH and TANH below |x| = 1 give Excel's exact doubles, by reproducing the expm1 that Excel's
`(a - b)/2` and `(a - b)/((a + b) + 2)` are built from (a = expm1(x), b = expm1(-x)). COTH and CSCH
follow.

## Context
T-2134 found the shape (docs/excel-semantics/numbers.md "SINH and TANH below 1 are built from
expm1"). With libm's expm1 it matches 773 SINH and 781 TANH cells of the 1015 below 1 in
probes-numeric `t2178-hyperbolic-unit-sweep`. For every argument some (a, b) within 3 ulp of libm's
pair fits both cells, so what is left is Excel's expm1 itself.
- Already rejected (they are about as close to correctly rounded as libm): x87 F2XM1 of RN64(x log2 e)
  or of t rounded to a double, with or without a reciprocal for t < 0; expm1 kept in 64 bits.
- The fitting data: each argument's set of admissible (a, b) offsets, from the sweep's SINH and TANH
  pairs. Where they are unique, they give Excel's expm1 to the ulp. Ideas: a table-driven expm1
  (AMD/MSVC-style, 2^(j/32) table plus a polynomial); EXP V6 (`x87_exp.rs`) minus 1 above some |x|,
  with an accurate form below it; a probe of SINH and TANH at more arguments, so that the (a, b) sets
  narrow.

Owned cells. They fail because libm's expm1 is an ulp off Excel's, under T-2134's form:
- Decision 0092 regressions:
  - `TRIGONOMETRIC.xlsx` SINH_COSH_TANH!D51, B15 and B17, and COTH_CSCH_SECH!B51, C15 and C17;
  - gen-m5 0362d48f56108faa A10, 4d099bea7898ebf1 A361, 651e713f5ae232ed A239, b0d597ec5af5fb92
    A180, b0ecb7ca78460b77 A8 and A294, ee1667411bbe2ccc A141.
- Still failing from T-2134's list: COTH_CSCH_SECH!C54 and t0917-hyperbolic-switchover A159
  (CSCH(0.88)), and SINH_COSH_TANH!B51 (SINH(PI()/4)).

## Acceptance criteria
- [ ] The cells above pass; no new known failures in any baseline.
- [ ] The t2178 sweep's SINH and TANH cells below 1 do not lose against T-2134 (773 and 781 of 1015).
- [ ] The expm1 formulation and its evidence are in `docs/excel-semantics/numbers.md`, including
      negative results.
- [ ] `make check` green; reviewers approve.

## Out of scope
- COSH (already `(e^x + e^-x)/2`, 1033 of 1035 sweep cells) and |x| >= 1.
