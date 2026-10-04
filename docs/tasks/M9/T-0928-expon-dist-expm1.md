---
id: T-0928
title: "EXPON.DIST cumulative: use -expm1 for small lambda*x (6 known failures)"
milestone: M9
lane: functions
status: done
priority: P3
size: S
depends_on: []
---

## Goal
EXPON.DIST(x, lambda, TRUE) is accurate for tiny lambda*x like Excel.

## Context
Oracle-free: `ironcalc-calc/STATISTICAL/EXPONDIST.xlsx` `EXPON.DIST!D15` expected 9.999995000001667e-7, actual 9.999994999843054e-7; `D16` expected 9.999999999500002e-11, actual 1.000000082740371e-10 (NumericMismatch); `D17`, `D20`, `D21` similar; `D37` 1 ulp.
Cause: `exponential.rs` computes `1.0 - (-lambda*x).exp()` (catastrophic cancellation). Use `-(-lambda*x).exp_m1()`; verify the PDF branch and the D37 ulp cell against cached values.

## Acceptance criteria
- [ ] The 6 `F` lines for EXPONDIST.xlsx disappear from `eval/baseline/ironcalc-calc.txt`, no new ones (if D37 stays, document why).
- [ ] Unit test; `make check` green.

## Out of scope
FISHERINV small-x (Excel itself is inexact there; needs oracle).

## Outcome
CDF now uses `-exp_m1`; all 6 EXPONDIST.xlsx known failures (incl. D37) fixed, 0 regressions (ironcalc-calc 1632 -> 1626).
