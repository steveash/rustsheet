---
id: T-2744
title: "COSH one ulp off Excel at x = 0.9291959915265351 and 8.697490026177835E-07 (t2178-hyperbolic-unit-sweep D634, D832)"
milestone: M5
lane: functions-math
status: todo
priority: P3
size: S
tier: opus
defer: phase-1b
depends_on: []
---

## Goal
COSH gives Excel's double at the two points of the T-2178 sweep where it is one ulp off, without losing any
of the other 1033 COSH cells.

## Context
Gap triage 2026-10-25. probes-numeric/t2178-hyperbolic-unit-sweep (excel-graph, T-2178) has 1035 COSH cells;
the engine's `(e^x + e^-x)/2` (numbers.md "SINH / COSH / TANH are exp-based, not libm") matches 1033. T-2240
put the other 2 out of scope ("COSH (already `(e^x + e^-x)/2`, 1033 of 1035 sweep cells)") and no open task
owns them:
- D634 `COSH(0.9291959915265351)`: Excel 1.4636716460960737, ours 1.463671646096074 (exact
  1.46367164609607386819, so Excel is 0.77 ulp below it, ours 0.23 ulp above: ours is correctly rounded).
- D832 `COSH(8.697490026177835E-07)`: Excel 1.0000000000003784, ours 1.0000000000003781 (exact
  1.00000000000037823166; Excel 0.60 ulp above, ours 0.40 ulp below).
Both cells are within 15 significant digits, so the task is deferred (decision 0114). Excel is the inaccurate
one, so this is a fit of Excel's formulation (time-boxed, decision 0056), not an accuracy fix.
Candidates, scored on all 1035 sweep cells: `excel_exp` (crates/rustsheet-engine/src/x87_exp.rs) in place of
libm `exp` for one or both terms (numbers.md near line 608 records that switching COSH to `excel_exp` changed no
gated cell; the sweep was not scored then), `e^x/2 + e^-x/2`, `(e^x + 1/e^x)/2`, and `1 + expm1`-based forms
near 0 (D832 is at |x| < 1E-6, where `e^x + e^-x` is 2 plus a tiny difference).
Code: `cosh` in crates/rustsheet-engine/src/functions/math_and_trigonometry/hyperbolic.rs. SECH is 1/COSH,
so check SECH's cells (ironcalc-calc TRIGONOMETRIC.xlsx COTH_CSCH_SECH) for regressions.

## Acceptance criteria
- [ ] D634 and D832 of probes-numeric/t2178-hyperbolic-unit-sweep pass bit-exact and every COSH cell of that
      sweep that passes on main still passes (`make eval-show CASE=probes-numeric/t2178-hyperbolic-unit-sweep.case.json`),
      or, at the time-box, the candidates and their sweep scores are recorded in numbers.md "SINH / COSH / TANH".
- [ ] No new known-failure lines in any eval/baseline file (`make eval-baseline`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- SINH and TANH (T-2134, T-2240 fitted them).
- COSH overflow near 710.
