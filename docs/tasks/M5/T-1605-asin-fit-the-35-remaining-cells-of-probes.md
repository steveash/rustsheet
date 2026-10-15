---
id: T-1605
title: "ASIN: fit the 35 remaining cells of probes/asin-argument-sweep (Excel's ASIN is not odd)"
milestone: M5
lane: functions-math
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
ASIN matches Excel bit-for-bit on every cell of probes/asin-argument-sweep, or the remaining cells are explained in docs/excel-semantics/numbers.md.

## Context
T-1259 swept ASIN with excel-graph (probes/asin-argument-sweep, 268 cells) and switched `fn_asin`
(functions/math_and_trigonometry/mathematical.rs) to `atan(x/sqrt((1-x)*(1+x)))` in doubles. That fits 233 cells; libm `asin` fits
186. The 35 misses are known failures. 26 of them are negative arguments, because Excel's ASIN is not odd: 31 ± pairs in the sweep
differ, e.g. `ASIN(0.09)` = 0.09012194501459526 but `ASIN(-0.09)` = -0.09012194501459525. Also `ASIN(1E-13)` = 1E-13 exactly.
The models tried are listed in numbers.md "ASIN of a small argument". They are double/extended `1-x*x` and `(1-x)(1+x)`, extended sqrt and
division, a correctly rounded atan of the quotient, and an x87 double rounding; none beat 233.
Ideas not yet tried:
- a polynomial or table kernel that is not sign-symmetric (e.g. Excel computing `-x` paths through `acos`, or `π/2 - 2 asin(sqrt((1-x)/2))` near ±1);
- the fit lab's x87 model (T-1524) for FPATAN with its own rounding;
- ACOS evidence (T-0939) for a shared kernel;
- `ASIN(±1E-13)` is ±1E-13 exactly (libm too), but the T-1259 formula gives ±1.0000000000000002E-13, so try a tiny-|x| shortcut;
- `libm::atan` (the `libm` crate is already an engine dependency) in place of the platform `atan`, so the result does not
  depend on glibc; keep it if it also scores 233 (T-1259 review).

## Acceptance criteria
- [ ] probes/asin-argument-sweep loses known failures with no regression in any ASIN cell (ironcalc-calc, gen-m6, gen-m7), or numbers.md records why the residue stays.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
