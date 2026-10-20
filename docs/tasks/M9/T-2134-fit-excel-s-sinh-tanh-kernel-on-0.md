---
id: T-2134
title: "Fit Excel's SINH/TANH kernel on 0 .. 1 (COTH/CSCH inherit it)"
milestone: M9
lane: functions-math
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-0917]
---

## Goal
SINH and TANH give Excel's exact doubles on |x| < 1 (where Excel is up to 1.8 ulp off the true
value), so COTH, CSCH and SECH, which are 1 over them (T-0917), follow.

## Context
T-0917 tried and rejected every simple formulation (docs/excel-semantics/numbers.md "SINH and TANH
on 0.4 .. 1: no formula found yet"): libm, exp forms with libm or the x87 EXP model V6, 80-bit
forms, Taylor series in several summation orders, `expm1` forms. Excel misses the true value by
more than half an ulp at e.g. SINH(0.01), 0.05, 0.43, 0.47, 0.65, 0.7, 0.88 and TANH(0.01),
0.54, 0.85, so some double-precision intermediate is involved. Ideas not yet tried: an x87 F2XM1
based `expm1` stored to a double then `(E + E/(E+1))/2` (fdlibm's shape) or `E*(E+2)/(2(E+1))`;
a fixed-degree minimax polynomial (fit lab, T-1523, docs/research/excel-kernel-playbook.md).
Fitting set: probes-numeric `m9-hyperbolic` A157-A248, `t0917-hyperbolic-switchover` A79-A210,
ironcalc-calc TRIGONOMETRIC.xlsx SINH_COSH_TANH. A dense oracle sweep on (0, 1) is likely needed.
Known failures this owns: TRIGONOMETRIC.xlsx SINH_COSH_TANH!D10, D40, B51 and COTH_CSCH_SECH!B10,
B40, C51, C54, C55 (B10 and C54 were unmasked by T-0917, decision 0084), and POWER.xlsx
inside_a_lambda!C3/C4 (TANH in `gelu`, T-0959).

## Acceptance criteria
- [ ] COTH_CSCH_SECH!B10 and C54 pass (decision 0084); no new known failures.
- [ ] The fitted formulation and its evidence are in `docs/excel-semantics/numbers.md`, including
      negative results.
- [ ] `make check` green; reviewers approve.

## Out of scope
- SINH/COSH overflow near 710 (SINH(710) is `#NUM!` in Excel).
