---
id: T-2747
title: "YIELD at a 4200% coupon and a price of 3.14159 is #NUM! in Excel (ours 19.04): the Newton domain below par"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
YIELD gives Excel's `#NUM!` where Excel's solver gives up far below par at a huge coupon, instead of our converged yield.

## Context
Split from T-2620 (its probe cell; the cut is unknown, so it needs evidence before code).
- probes/t2475-stat-subexpressions P!A214: `YIELD(39681,45662,42,3.14159,102,4)` (rate 42, price 3.14159,
  redemption 102, frequency 4) is `#NUM!` in Excel (excel-graph, 2026-10-24); ours is 19.040243161919385.
- gen-m9/44a8841352dc48e9 Calc!A318 isolates it: `PERCENTRANK(Data!D5:D6,YIELD(...),5)`. Excel `#NUM!`, ours `#N/A`.

The price is far below par and the coupon far above any pool T-2304 or T-2187 swept, so the iteration's start, step cap
or tolerance may fail (financial.md "YIELD's #NUM! cells and its 100-step cap"). T-2391 owns the price-above-par
side (1E6), and T-2740 sweeps the Newton start. Use the excel-probe skill to locate the cut with a sweep over rate (1 to
100) and price (1 to 20), then match it in `fn_yield` (functions/financial/financial_bonds.rs).

T-2391 (2026-10-26): YIELD's Newton lost its fallback, so an iterate off PRICE's domain is `#NUM!`; both cells
(P!A214, gen-m9/44a8841352dc48e9 A318) left the baselines there. What is left is the probe sweep that confirms the rule
over rate and price.

## Acceptance criteria
- [x] A probe case locates where Excel's YIELD turns to `#NUM!` over rate and price at this bond, and the rule is in
      docs/excel-semantics/financial.md.
- [x] P!A214 and gen-m9/44a8841352dc48e9 A318 leave the baselines (`make eval-baseline`), with no new known failures.
- [x] `make check` green; reviewers approve.

## Out of scope
- YIELD above par (T-2391), the Newton start's last digits (T-1592, T-2741).

## Outcome
probes/yield-deep-discount-sweep (160 cells, excel-graph 2026-10-31) sweeps `YIELD(39681,45662,R,P,102,4)` over coupon
rate R = 1 to 100 (20 values) and price P = 1 to 20 (8 values). Excel computes for every R up to 20 and is `#NUM!` for
every R from 25 at every price: far below par the cut follows the coupon rate, not the price. T-2391's rule (Newton
from the fitted start, `#NUM!` on a non-finite step) reproduces all 160 in type with no code change; 123 cells are
bit-exact and 37 are 1 to 5 ulp off, listed in T-2788 (the open owner of YIELD's last digits). The rule is in
financial.md "YIELD's #NUM! cells and its 100-step cap". Eval delta: +160 probe cells checked, +123 passing, 37 new
known failures, all in the new case (no change to any existing case's cells).
