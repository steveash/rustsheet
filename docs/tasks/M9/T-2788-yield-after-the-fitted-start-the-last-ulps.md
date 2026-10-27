---
id: T-2788
title: "YIELD after the fitted start: the last ulps at the root (3 regressed probe cells, 9 lab cells)"
milestone: M9
lane: research
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-1592]
---

## Goal
YIELD (more than one coupon left) matches Excel bit for bit on the cells T-1592's fitted Newton leaves a few ulps off
at the root, including the three cells T-1592 regressed by 1 ulp under decision 0083's allowance.

## Context
Split from T-2777 on 2026-10-26 (T-1592 review): the ulp part of the follow-up, with no campaign key (decision 0083).
T-1592 fitted the start (100R + (100-P)/T)/((100+P)/2 - (100-P)/(4T)) and the stop (a step below 1e-10 is not
taken). The engine follows it (`excel_yield_newton`, `yield_start` in
crates/rustsheet-engine/src/functions/financial/financial_bonds/yield_solver.rs). financial.md "YIELD stops short of
PRICE's root", T-1592 fourth-round paragraph; lab fit `uv run rustsheet-fitlab yield-caret`.
- Ulp regressions T-1592 took under decision 0083's allowance, at the root, 1 ulp low in the engine and the lab
  alike (no PRICE summation order tried moves them): probes/yield-start-one-input S!A5
  (Excel 0.10352912862429842, ours 0.1035291286242984), probes/yield-start-rate-term S!B34 (0.11099781701293797,
  ours ...795) and S!B35 (0.153546637344011, ours 0.15354663734401097).
- Lab cells 7 to 33 ulp off: solver-iteration-traces Y!E2; yield-solver-sweep S!C7, S!C11;
  yield-start-one-input S!B11, B17, B31, B54, B62, B69.
- gen-m9 cells within 4 ulp after T-1592: A229 (3aa168259335be54), A333 (5ded7594e597e31e), A376 (6376e27be7a05871),
  A231 (9a1fa5366bc50918), A122 and A269 (b73c6cc96e093f03), A23 (bc04c1f478c9896c), A223 (c0681a8b150ce301).
- Handed over by T-2777 (2026-10-26) once its term fix left them ulp-level: gen-m9/81bcdc0bec0c8e6b A189
  (basis 2, Excel -0.0008265806588318691, ours ...682, 8 ulp), gen-m9/a91bad4e84f49739 A102 (-0.0008665128023479636,
  ours ...645) and gen-m9/e168be7f6d3f84c2 A194 (0.05369271319672304, ours ...308), which miss Excel's 15th
  significant digit by a rounding boundary; and probes/yield-start-term-basis A9, C11, C20, A26 and A38, at the
  root and 4e-15 to 1e-14 off, plus B50 and A52 (sub-year, 10 to 12 ulp in the engine, 1-2 ulp in the lab) and the
  probe's 73 cells 1-2 ulp off.
- Gap triage 2026-10-27 (`make eval` on 7dba3bfe): one more cell 8 ulp off at the root. It is just past half a
  unit of the 15th digit and no task named it: probes/yield-start-one-input S!B67
  `YIELD(DATE(2096,3,1),DATE(2097,6,30),0.05,98,100.5,1,0)`, Excel 0.06966784545910434, ours 0.06966784545910445.

## Acceptance criteria
- [ ] The three regressed probe cells pass bit for bit (their lines leave eval/baseline/probes.txt), or the task
      records the PRICE kernels and stop rules tried with their lab bit-exact counts (decision 0056).
- [ ] The lab's "Newton, fitted start" family raises its bit-exact count above 240/370, or the task is blocked with
      the candidates tried.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The visibly-off basis 1 and 2 cells (T-2777); YIELD's `#NUM!` cells and its 100-step cap (T-2391).
