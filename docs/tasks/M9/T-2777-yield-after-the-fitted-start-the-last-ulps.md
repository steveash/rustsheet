---
id: T-2777
title: "YIELD after the fitted start: the basis 1 and 2 terms and two 15th-digit cells (gen-m9, 4 cells)"
milestone: M9
lane: research
status: in-progress
priority: P2
size: M
tier: opus
depends_on: [T-1592]
campaign: visible
---

## Goal
YIELD (more than one coupon left) matches Excel to 15 significant digits on the four gen-m9 cells T-1592's fitted
Newton still leaves visibly off: the start's term on bases 1 and 2, and two 15th-digit cells. The ulp leftovers at the
root (no campaign key, decision 0083) are T-2788.

## Context
T-1592 fitted the start (100R + (100-P)/T)/((100+P)/2 - (100-P)/(4T)) and the stop (a step below 1e-10 is not
taken). The engine follows it (`excel_yield_newton`, `yield_start`). financial.md "YIELD stops short of PRICE's root",
T-1592 fourth-round paragraph.
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
- [ ] Visible parity: the four cells above are within half a unit of the 15th significant digit of Excel's value (their
      known-failure lines in eval/baseline/gen-m9.txt disappear, or turn into ulp-level misses handed to T-2788).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The ulp-level cells at the root (T-2788).
- YIELD's `#NUM!` cells and its 100-step cap (T-2391).

## Owned cells
Cut from T-1592 on 2026-10-26 (decision 0083 "Done at visible parity": the four owned cells T-1592's fit left visibly
off move here with the campaign key; T-1592's other 19 owned gen-m9 cells pass or are within 4 ulp).
- gen-m9/81bcdc0bec0c8e6b Calc!A189, gen-m9/ba4ddb817a8e700e Calc!A17, gen-m9/a91bad4e84f49739 Calc!A102,
  gen-m9/e168be7f6d3f84c2 Calc!A194.
