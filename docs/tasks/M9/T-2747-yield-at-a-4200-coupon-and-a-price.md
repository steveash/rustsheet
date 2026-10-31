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
      (Met in T-2391. This PR's new probe cases add new-coverage known failures only; see the Outcome.)
- [x] `make check` green; reviewers approve.

## Out of scope
- YIELD above par (T-2391), the Newton start's last digits (T-1592, T-2741).

## Outcome
- probes/yield-below-par-sweep (184 cells: `YIELD(39681,45662,R,P,102,4)`, rates 1 to 100, prices 1 to 20; excel-graph,
  2026-10-31): T-2391's Newton already matched 183 in type (values within 2.5e-12 relative). Excel is `#NUM!` from
  rate 24 up at every price and from about 21.5 (price 1) to 23 (price 20); between, the first step jumps to a
  negative yield and the climb back can outlast 100 steps, so the value is the 100th iterate, a negative non-root
  (A10 -1.853, F13 -0.852, H17 -2.282).
- The 184th, H16 (`YIELD(39681,45662,22.75,20,102,4)`), is `#NUM!` in Excel where our Newton meets its step test on
  the 100th and last check. probes/yield-newton-step-cap (15 cells, three prices, rates where our uncapped Newton
  converges on checks 98 to 102) confirms it: Excel is `#NUM!` on exactly the check-100 cell of each column and
  agrees with ours on the other 12. `compute_yield` now refuses a check-100 stop (`NewtonStop::converged_on`);
  ODDFYIELD, the same Newton, keeps its behavior pending evidence (follow-up T-3485).
- Rule in financial.md "YIELD's #NUM! cells and its 100-step cap". Eval: the two new probe cases add 199 cells; with
  the fix every cell matches in type; probes known failures 2201 -> 2270, 0 regressions, 130 new passing cells
  including all three check-100 cells. The 69 new-coverage failures: 60 NumericUlp cells (last ulps, T-2788) and 9
  NumericClose cells, up to 2.5e-12 relative, all 100th iterates on the negative-yield climb (follow-up T-3486).
