---
id: T-2629
title: "IMCOSH/IMSINH/IMSECH 15th digit at a large imaginary part (eng-im-overflow-edges A9, A17, A58)"
milestone: M9
lane: functions-engineering
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
probes/eng-im-overflow-edges A9, A17 and A58 print Excel's 15th digit.

## Context
Left by T-2231. Each differs in the last printed digit of the imaginary part, and each exact value sits within
about 5E-19 of the 15-digit rounding midpoint, so the cells only tell kernels apart through their last ulp:
- A9 `IMCOSH("1.5+710.4i")`: Excel 0.82929871219349, ours 0.829298712193489 (exact 0.8292987121934895003).
- A17 `IMSINH("1.5+710.4i")`: Excel 0.916202079442849, ours 0.916202079442848 (exact 0.9162020794428484950,
  so Excel's double is above the exact value).
- A58 `IMSECH("1.5+711i")`: Excel -0.371302652691887, ours ...888 (exact -0.3713026526918874831).
Tried by T-2231 (none fits A9 and A17): libm sin/cos, Excel's SIN/COS reduction (`trig_sin_cos`, x87 π),
`(e^x ± e^-x)/2` and complex-exp sums for sinh/cosh. Next candidates: x87 extended products, or a check
whether IM* text rounds a halfway shortest-repr string up (`0.9162020794428485` → ...849) before refitting
kernels. Code: `imcosh`, `imsinh`, `imsech` in crates/rustsheet-engine/src/functions/engineering/complex/trig.rs.

Findings (2026-10-26, opus run after T-2750; data, not a fix):
- The text hypothesis is refuted. Rounding each IM* part half-up from its shortest-repr string fixes A17 but costs 21
  gated cells (probes 2666 -> 2686, ironcalc-calc 644 -> 662). Rounding from the 17-digit string fixes none and costs 2.
  IM* text rounds the double itself, as `to_precision` does.
- In all three cells Excel's double is one ulp above (toward +inf) the correctly rounded exact value (A9 0.8292987121934895
  → Excel's must be ≥ 0.8292987121934896; A17 0.9162020794428485 → ...486; A58 -0.3713026526918875 → -0.37130265269188745).
  Excel's SIN reduction by the 64-bit π moves the exact value the other way (A9 0.82929871219348948), so it is not the cause.
- The products fit both A9 and A17 if Excel's sinh(1.5) and cosh(1.5) are each one ulp above libm's
  (2.1292794550948178, 2.3524096152432477), or if sin(710.4) is 1-2 ulps above libm's 0.38947387117702714. None of
  `(e ± 1/e)/2`, `(e ± exp(-x))/2`, `e/2 ± 0.5/e` or the expm1 forms moves sinh or cosh. No oracle cell has
  SINH(1.5), COSH(1.5), EXP(1.5) or SIN(710.4) (m9-hyperbolic has ACOSH(1.5) only), so next: probe
  `=SINH(1.5)-2.1292794550948173`, `=COSH(1.5)-2.352409615243247`, `=EXP(1.5)-4.4816890703380645` and
  `=SIN(710.4)-0.38947387117702714` (excel-probe skill) before refitting.

## Acceptance criteria
- [ ] The three lines leave eval/baseline/probes.txt (`make eval-baseline`) with no new known-failure lines,
      or the candidates tried are recorded in docs/excel-semantics/engineering.md (time-box, decision 0056).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
