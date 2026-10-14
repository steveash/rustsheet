---
id: T-1529
title: "GAMMALN and half-integer GAMMA last digits: Excel's gamma routines (time-boxed)"
milestone: M9
lane: functions-stat
status: blocked
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
GAMMALN at all arguments and GAMMA at non-integers match Excel bit for bit on the T-1042 sweeps
(split from T-0973, which keeps the actionable GAMMA-at-integers change).

## Context
Kernel map rank 8 (`docs/research/numeric-kernel-map.md`); decision 0056. Sweeps (excel-graph,
2026-10-13): `eval/suites/probes-numeric/cases/t1042-gamma-int-1-90`, `t1042-gamma-int-91-171`,
`t1042-gamma-half-int`, `t1042-gamma-unit-interval`, `t1042-gamma-negatives-edges`. Fit lab:
`uv run rustsheet-fitlab gammaln-int` and `gamma-half-int`, `tools/src/rustsheet_tools/fitlab/kernels/gamma.py`
(the lab has no high-precision gamma, so only integers and half-integers are scored). Findings:
docs/excel-semantics/statistics.md "GAMMA and GAMMALN sweep (T-1042)". `GAMMALN(3)` =
0.6931471805599454 is one ulp above ln 2, so GAMMALN is not ln of an exact or double factorial. Engine
code: `functions/statistical/special.rs` `ln_gamma`; `functions/statistical/gamma.rs`.

## Acceptance criteria
- [ ] A GAMMALN formulation matches >= 99.9% of the 171 integer cells (today's best: the engine's
      fdlibm `lgamma` 108) and improves the half-integer and unit-interval GAMMALN cells.
- [ ] A GAMMA formulation for non-integers matches >= 99.9% of `t1042-gamma-half-int` GAMMA cells
      (best: 45/62, correctly rounded).
- [ ] Recorded in statistics.md with the sweep cells; implemented with no new baseline lines.
- [ ] `make check` green; reviewers approve.

## Out of scope
GAMMA at integers n <= 88 and the n >= 89 routine (T-0973); incomplete gamma (T-0971).

## Blocked because: time-boxed (decision 0056)
T-1524 made two attempts (2026-10-14) and no candidate rose above the correctly rounded value or the engine:
- GAMMALN at integers (171): ln (libm, or correctly rounded at 64 bits) of the factorial product taken
  upward or downward, each product rounded to a double, twice rounded or kept at 64 bits: at most 82
  (downward, doubles); correctly rounded 80; libm `lgamma` 72; the engine (fdlibm) 108.
- GAMMA at half-integers (62): the recurrence from sqrt(pi) upward (multiplying factor by factor) or
  downward (the factors' product, then sqrt(pi)), sqrt(pi) a double or at 64 bits, per rounding model:
  at most 34 in doubles; 64-bit 45 = correctly rounded 45; starting from Excel's own GAMMA(0.5)
  (= the correctly rounded sqrt(pi)) 33; downward from Excel's GAMMA(1.5) or GAMMA(2.5), 23 of the
  50 and 20 of the 49 cells they reach; libm `tgamma` 21.
Reopens on a new hypothesis recorded here, for example: a named routine (Lanczos with specific
coefficients, Stirling with a stated number of terms, Cody's or Hart's gamma) that also explains the
n >= 89 integer values, or a high-precision gamma in the fit lab so the unit-interval sweep can be scored.
