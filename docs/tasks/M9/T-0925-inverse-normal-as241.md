---
id: T-0925
title: "Inverse normal (Wichura AS241) for NORM.S.INV/NORM.INV/LOGNORM.INV/CONFIDENCE.NORM (~120 known failures)"
milestone: M9
lane: functions
status: todo
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
NORM.S.INV, NORM.INV, LOGNORM.INV, CONFIDENCE.NORM (and the NORM-based part of CONFIDENCE.T inputs where applicable) return Excel's values, using one shared accurate inverse-normal routine.

## Context
Oracle-free: Excel's cached values are the exact quantiles, ours (statrs `Normal::inverse_cdf`, which goes through a lower-accuracy erf_inv) are off by ~1e-11.
- `ironcalc-calc/STATISTICAL/NORM_DIST.xlsx` `NORM.S.DIST NORM.S.INV!D2` `=NORM.S.INV(C2)` expected 1.3333330000000008, actual 1.3333329999747054; `D4` expected 0.8000000000000002 actual 0.7999999999297425. NORM.S.INV 35 cells, NORM.INV 29 cells (incl. 2 NumericMismatch).
- `ironcalc-calc/STATISTICAL/LOGNORM.xlsx` `LOGNORM.DIST LOGNORM.INV!F2` expected 4.000000000000003 actual 4.000000000091908 (31 cells).
- `ironcalc-calc/STATISTICAL/CONFIDENCE.xlsx` CONFIDENCE.NORM 23 cells, all NumericUlp (`D2` expected 0.2771807648699355 actual 0.27718076486993554).
Code: `functions/statistical/{normal,log_normal}.rs`. Implement AS241 (PPND16) once, in the same module as the T-0924 kernel if that has landed (else a new `normal_kernel` module that T-0924 extends), and route all four functions through it. Where AS241 alone leaves ulp-level diffs, test whether one Newton/Halley refinement step against the T-0924 CDF reproduces the cached cells and keep what matches.

## Acceptance criteria
- [ ] All NumericClose/NumericMismatch lines for NORM.S.INV, NORM.INV, LOGNORM.INV disappear from `eval/baseline/ironcalc-calc.txt`; the CONFIDENCE.NORM ulp lines disappear or the PR documents which remain and why. Exact before/after counts in the PR; no new lines.
- [ ] No other call sites of statrs `inverse_cdf` for Normal (rg).
- [ ] Unit tests incl. p near 0/1 and p = 0.5; `make check` green.

## Out of scope
erf/CDF accuracy (T-0924); T.INV/GAMMA.INV/BETA.INV iterations.
