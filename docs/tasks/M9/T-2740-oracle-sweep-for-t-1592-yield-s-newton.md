---
id: T-2740
title: "Oracle sweep for T-1592: YIELD's Newton start one input at a time (price, coupon rate, term) on the T-2304 bond"
milestone: M9
lane: research
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Excel YIELD values that move one input at a time (price alone, coupon rate alone, term alone) around one bond. Each
cell that stops below PRICE's root gets its Newton back-chain in the fit lab. T-1592 can then fit the hidden start S
per input instead of on the coarse T-2304 grid. T-1592's "Blocked because" names this evidence as the way to unblock
it. No task asked for it until now.

## Context
Gap triage 2026-10-25 (`make eval` on `main` 22588cf). YIELD is the largest visibly-off financial function: 481
visibly-off cells in the report's "Visibly off (fifteen digits)" table. Of these, 370 are probe and ironcalc-calc cells
(T-2741) and 23 are gen-m9 cells (T-1592). All come from one rule, docs/excel-semantics/financial.md "YIELD stops short
of PRICE's root": Excel runs Newton on PRICE from a start S and returns an iterate that is short of the root.
- T-2187 (probes/yield-start-sweep) showed that S does not depend on redemption or frequency, but does depend on
  price and on the basis's term in years.
- T-2304 (probes/yield-start-rate-term, 150 cells) showed that S also moves with the coupon rate and the term. It
  sampled each input at only 2 to 5 values (rates 0, 0.02, 0.05, 0.08, 0.12; maturities 2098, 2100, 2104, 2110,
  2126; prices 98 and 90). That is too coarse to fit S as a function of any one input.
- T-1592 (blocked 2026-10-24, third round) tried 560 finite-difference variants, annuity roots at every compounding,
  and Newton, secant, regula falsi, Illinois and bisection iterates. None reproduces S. Its "Blocked because" says:
  "Unblock with evidence that separates the start from the stop rule: T-2330's re-probe of the stop rule, or a probe
  that fixes everything but one input (price alone, rate alone, term alone) on the T-2304 grid so S can be fitted per
  input." T-2330 is the stop-rule route. This task is the other route.
- On price alone, the cached cells leave the step count ambiguous (financial.md, T-1592 second round): with one or
  two steps back, price 80 (E4) gives 0.109134476 or 0.10825933. A dense, smooth price axis, with the frequency-2 and
  frequency-1 twins sharing S, removes that ambiguity. In T-2187, A11, B11 and C11 met at one S two steps back.

This is a numeric-fitting group that needs more Excel points than the cached workbooks give. Following the
gap-triager's rule, it is an oracle sweep, not a blind fit.

Sweep design: one probe case, `eval/suites/probes/cases/yield-start-one-input.case.json`, tag `M9`, `"expected": {}`.
Follow the `excel-probe` skill and write the layout into the case description. The base bond is T-2304's A8 row,
`YIELD(DATE(2096,3,1),M,R,P,100.5,F,0)`: basis 0, settlement 1 March 2096, redemption 100.5 so that most cells stop
below the root, and M = DATE(2100,6,30), R = 0.05, P = 98 unless the row varies them. Column A is F = 2 and column B is
F = 1. The twins share S, which pins the step count. Rows:
- 1-41: price P from 80 to 100 by 0.5.
- 42-66: coupon rate R from 0 to 0.12 by 0.005.
- 67-96: maturity M on 30 June of each year from 2097 to 2126.
That gives 192 cells, all one formula shape. Fill them with `cd tools && uv run rustsheet-oracle fill -b excel-graph
--suite probes`; the oracle reported excel-graph OK on 2026-10-25. If no backend is available, leave the case pending
and set this task `blocked` on the oracle (excel-probe step 3).

Analysis:
- Run `solvers.newton_back_chain` (tools/src/rustsheet_tools/fitlab/kernels/solvers.py) on every cell below PRICE's
  root, taking the step count at which the A and B twins agree.
- Tabulate S against each input. Say whether S is smooth in that input, and give the simplest form that fits each
  axis to 1e-9 relative, or report that none does.
- Cells at the root (|PRICE(y) - P| below 1e-11) show no S. List them.

## Acceptance criteria
- [x] probes/yield-start-one-input.case.json and its `.oracle.json` (excel-graph, canary-verified) are committed with
      the 192 cells above. Their known failures are recorded in eval/baseline/probes.txt with `make eval-baseline`, and
      no other baseline line changes.
- [x] The case is in the fit lab's `YIELD_CASES` (fitlab/kernels/solvers.py). A lab test pins the back-chained S of at
      least one row per axis (for example rows 1, 42 and 67), with both twins.
- [x] financial.md "YIELD stops short of PRICE's root" gains a T-2740 paragraph that cites cells. It gives S per
      axis, the step counts, the cells at the root, and the form fitted per axis (or the forms rejected, with how far
      off each is).
- [x] T-1592's "Blocked because" points at this evidence. T-1592 goes back to `todo` if the evidence supports a new
      hypothesis for S.
- [x] `make check` green; reviewers approve.

## Out of scope
- Fitting S across all YIELD cells, or any engine change (T-1592, then T-2741).
- The stop rule (T-2330), YIELD's `#NUM!` cells and suspected 100-step cap (T-2391), ODDFYIELD (T-2007).

## Outcome
probes/yield-start-one-input (192 cells, excel-graph, canary-verified) is committed and in the fit lab's `YIELD_CASES`;
its 191 failing cells (ours is at PRICE's root, Excel short of it; 1 cell passes) are new-coverage lines in
eval/baseline/probes.txt, no other baseline line changed (the visible-parity lock rises 4519 -> 4640: the case adds 173 visibly-off cells and the old lock's 52 of slack are absorbed).
Findings (financial.md "YIELD stops short of PRICE's root", T-2740 paragraph): S is two Newton steps back on both
twins (rows 22-96, to 1.3e-8); it equals the coupon rate at price 100 and is linear in the coupon rate (0.0046704412 +
1.01128042·R, to 6e-9); its price and term shapes are tabulated but unfitted (time-box). Rows 1-5 and 67 are at the root
on both twins, rows 6-21 on B. `test_yield_start_one_input_two_steps_back` pins rows 41, 42, 52, 66 and 96. T-1592 is
back to `todo` with the new hypothesis (a current-yield or yield-approximation start). Eval delta: no fixes; probes
known failures 2666 -> 2857 (all new coverage), 0 regressions.
