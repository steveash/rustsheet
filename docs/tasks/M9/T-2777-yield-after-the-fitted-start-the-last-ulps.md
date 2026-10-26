---
id: T-2777
title: "YIELD after the fitted start: the last ulps, basis 1 and 2 terms"
milestone: M9
lane: research
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-1592]
---

## Goal
YIELD (more than one coupon left) matches Excel bit for bit on the cells T-1592's fitted Newton still misses: a few
ulps at the root, and the start's term on bases 1 and 2.

## Context
T-1592 fitted the start (100R + (100-P)/T)/((100+P)/2 - (100-P)/(4T)) and the stop (a step below 1e-10 is not
taken). The engine follows it (`excel_yield_newton`, `yield_start`). financial.md "YIELD stops short of PRICE's root",
T-1592 fourth-round paragraph.
- Ulp regressions T-1592 took under decision 0083's allowance, at the root, 1 ulp low in the engine and the lab
  alike (no PRICE summation order tried moves them): probes/yield-start-one-input S!A5
  (Excel 0.10352912862429842, ours 0.1035291286242984), probes/yield-start-rate-term S!B34 (0.11099781701293797,
  ours ...795) and S!B35 (0.153546637344011, ours 0.15354663734401097).
- Lab cells 7 to 33 ulp off: solver-iteration-traces Y!E2; yield-solver-sweep S!C7, S!C11;
  yield-start-one-input S!B11, B17, B31, B54, B62, B69.
- Basis 2 term (no probe yet; the engine takes days/360): gen-m9/81bcdc0bec0c8e6b Calc!A189
  `=YIELD(38173,44696,0.05/12,110,101,2,2)` Excel -0.0008265806588318691, ours -0.0008265806323880697 (days/360);
  days/365 gives -0.0008265806589638128 and the coupon-schedule term (N - 1 + DSC/E)/f -0.0008265806607058653.
- Basis 1: gen-m9/ba4ddb817a8e700e Calc!A17 `=YIELD(38743,46332,Data!C9,116,Data!D5,4,1)` Excel 0.7268375931030532,
  ours 0.7268375931031176.
- 15th digit: gen-m9/a91bad4e84f49739 A102 (-0.0008665128023479636, ours ...645), gen-m9/e168be7f6d3f84c2 A194
  (0.05369271319672304, ours ...308).

## Acceptance criteria
- [ ] A probe of basis 2 (and basis 1 at frequency 4) YIELD cells with more than one coupon left fixes the start's
      term on those bases; financial.md records it.
- [ ] The cells above pass, or the task records why each cannot with the candidates tried (lab fits).
- [ ] `make check` green; reviewers approve.

## Out of scope
- YIELD's `#NUM!` cells and its 100-step cap (T-2391).
