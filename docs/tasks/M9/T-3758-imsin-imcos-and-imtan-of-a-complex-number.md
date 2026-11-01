---
id: T-3758
title: "IMSIN, IMCOS and IMTAN of a complex number reduce a large real part as SIN does"
milestone: M9
lane: functions-engineering
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The complex IMSIN, IMCOS and IMTAN (and any other IM* trig path that still takes libm's sine) print the same last digits as Excel for a large real part, as the real-number paths already do.

## Context
T-3642's grid (probes/im-trig-magnitude-grid, excel-graph) has 74 cells that are text in both but differ in the
last digits, mostly IMSIN, IMCOS and IMTAN at |x| = 700 to 720 (its 41 IMCSCH cells at x = 700 to 709, where ours prints 0, are T-2668's and T-2914's): `IMSIN(COMPLEX(710,1E-300))` is
0.0000602887066915739 in Excel, the same as the real `IMSIN("710")` (x87 reduction, `excel_sin_cos`), while ours,
via libm `x.sin()` in `imsin`, prints ...853; `IMTAN(COMPLEX(710,1))` real part ...901 (ours ...949). Code:
crates/rustsheet-engine/src/functions/engineering/complex/trig.rs (`imsin`, `imcos`, `tan_parts`, ); engineering.md "IM* trig and hyperbolic functions past SIN's argument limit (T-3642)".
probes/im-trig-argument-cut adds 37 such text cells near 2^27, where libm and the x87 reduction differ by more than the
last digit.
The existing last-digit fits (T-2538, T-3609, fit lab `im-hyperbolic`) must not move.

## Acceptance criteria
- [ ] The text cells of probes/im-trig-magnitude-grid outside T-3616's y = 709.78 band match, or the remainder is listed with its owner.
- [ ] No currently passing IM* cell changes (probes, probes-numeric, fit lab im-* sweeps).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The `#NUM!` cuts (T-3642, T-3616).
