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
probes/yield-deep-discount-sweep (160 cells) sweeps `YIELD(39681,45662,R,P,102,4)` over 20 coupon rates R = 1 to 100
and 8 prices P = 1 to 20; probes/yield-deep-discount-cut (36 cells) bisects R = 20.5 to 24.5 in steps of 0.5 at P = 1,
3.14159, 10 and 20 (both excel-graph, 2026-10-31). Excel computes at every sampled R up to 21 and is `#NUM!` from 23.5;
in between the last R that computes rises with the price (21 at P = 1 to 23 at P = 20), and at three prices it is a
negative 100th iterate, not a root. T-2391's rule (Newton from the fitted start, `#NUM!` on a non-finite step, the 100th
iterate after 100 steps) reproduces all 196 cells in type with no code change; 146 are bit-exact, 48 are 1 to 5 ulp off
and two 100th iterates 5e-14 and 6e-15 relative off, all listed in T-2788 (the open owner of YIELD's last digits). The
rule is in financial.md "YIELD's #NUM! cells and its 100-step cap". Eval delta: probes +196 checked, +146 passing, 50 new
known failures, all in the two new cases (no existing case's cells change).
