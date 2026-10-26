---
id: T-1592
title: "Fit YIELD's Newton start: a frequency-independent first iterate"
milestone: M9
lane: research
status: in-progress
priority: P2
size: M
tier: opus
campaign: visible
depends_on: [T-2187, T-2304, T-2740]
---

## Goal
A fit-lab model that reproduces YIELD (basis 0, more than one coupon left) bit for bit on more than
the 3/90 cells any candidate reaches today, starting from the first-iterate evidence T-1583 found.

## Context
docs/excel-semantics/financial.md "YIELD stops short of PRICE's root"; `uv run rustsheet-fitlab yield`.
T-1583: Excel's values are either at PRICE's root or below it with |PRICE(y) - pr| up to 5e-8, as a
Newton iteration on the convex PRICE that stops on a residual test without its last step would leave
them. Stepping analytic Newton backwards from Excel's below-root values (scratch analysis, reproduce it in
the lab) gives a shared earlier iterate that does not depend on frequency or redemption:
0.0552344627 for yield-solver-sweep S!A1 (frequency 2, 9 coupons) and S!F1 (frequency 1, 5 coupons),
the same within 2e-9 for S!E10 (redemption 100.5), and about 0.133352 for all three frequencies of
solver-iteration-traces Y!C2:E2 (price 50, ten years). The approximate-yield formula
(C + (100 - P)/n) / ((100 + P)/2), annual-coupon IRRs over whole or fractional years, the coupon
rate, 0, 0.1 and the current yield do not give these values.

T-2207 (2026-10-23): the engine's PRICE now discounts with Excel's `^`, but the YIELD lab's
`^`-priced copy of each candidate (`uv run rustsheet-fitlab yield-caret`) reaches at
most 4/178 bit-exact, as the libm-priced ones do (Newton from 0.1 to 1e-12: 4/178, 10 within 4 ulp, libm; 4/178, 7,
`^`); the union of all candidates goes from 12/178 to 15/178. The price kernel is not the missing piece. The
engine's YIELD solver still prices with libm (financial.md "PRICE and ODDFPRICE discount with Excel's `^`").

T-2257 sweep (gen-m9 financial; gap triage 2026-10-23): YIELD values of the financial sweep 1e-14 to 1e-10 relative off Excel's (`YIELD(38173,44696,0.004166666666666667,110,101,2,2)` Excel -0.0008265806588318691, ours -0.0008265805991797088), the T-1583 below-the-root pattern.

Owned gen-m9 cells (T-2257 triage, excel-graph, 2026-10-23): 23 known-failure lines in eval/baseline/gen-m9.txt, 23 visibly off.
Examples:
- gen-m9/0a957119475760c5 Calc!A312 `=YIELD(36930,45446,0.05/12,85,103,4)`: Excel 0.012717226922592558, ours 0.012717226922592995
- gen-m9/5ded7594e597e31e Calc!A333 `=YIELD(36555,44332,0.05,91,104,4)`: Excel 0.05836858293381488, ours 0.05836858293381556
- gen-m9/6376e27be7a05871 Calc!A82 `=YIELD(39467,46541,1,105,110,4,4)`: Excel 0.9464072633334488, ours 0.9464072633337517
- gen-m9/9a1fa5366bc50918 Calc!A231 `=YIELD(39148,45562,Data!D3,81,104,1)`: Excel 3.4746998470918595, ours 3.4746998470918977
- gen-m9/b73c6cc96e093f03 Calc!A122 `=YIELD(36788,46373,1,94,105,2)`: Excel 1.0351927620946566, ours 1.0351927620970889
- gen-m9/d912671baa1364aa Calc!A245 `=YIELD(36607,44044,1E-9,117,102,2)`: Excel -0.006727967356185666, ours -0.006727967356185589
Cells:
- gen-m9/0a957119475760c5.case.json (1): Calc!A312
- gen-m9/1cd3096e3e87c965.case.json (1): Calc!A65
- gen-m9/1dce4774f5c0e736.case.json (1): Calc!A207
- gen-m9/31474c20fc1dafd6.case.json (1): Calc!A272
- gen-m9/3aa168259335be54.case.json (1): Calc!A229
- gen-m9/5ded7594e597e31e.case.json (1): Calc!A333
- gen-m9/6376e27be7a05871.case.json (2): Calc!A82, A376
- gen-m9/81bcdc0bec0c8e6b.case.json (1): Calc!A189
- gen-m9/89f0aee953a55a45.case.json (1): Calc!A193
- gen-m9/9a1fa5366bc50918.case.json (1): Calc!A231
- gen-m9/a91bad4e84f49739.case.json (1): Calc!A102
- gen-m9/b72d952c5f52cb4d.case.json (1): Calc!A81
- gen-m9/b73c6cc96e093f03.case.json (3): Calc!A122, A171, A269
- gen-m9/ba4ddb817a8e700e.case.json (1): Calc!A17
- gen-m9/bc04c1f478c9896c.case.json (1): Calc!A23
- gen-m9/c0681a8b150ce301.case.json (1): Calc!A223
- gen-m9/d0854366830dcdee.case.json (1): Calc!A226
- gen-m9/d912671baa1364aa.case.json (1): Calc!A245
- gen-m9/e168be7f6d3f84c2.case.json (2): Calc!A47, A194

Calipers evidence (gap triage 2026-10-25; desktop goldens of real bond models, ungated suite, no other owner): 42 visibly-off
YIELD cells, the same below-the-root pattern, 1e-14 to 1e-9 relative:
- `calipers/pv_goldens/pv23_004` Bonds-Calls-Puts!K13 `YIELD(Settlement_Date,Maturity_Date,Coupon_Rate,...,100,Num_Coupons)`:
  Excel 0.05999999999993105, ours 0.05999999999999983. Also K16, K20:Y20 (except R20), K23:Y23 (except R23), K27, M27,
  O27, Q27, S27, U27, W27 and Y27. S20 and S23 are off by 1e-9 relative (Excel 0.057696257180238915, ours
  0.05769625724582727).
- `calipers/pv_goldens/pv23_003` Bond-Refinancing!K15, F39 and K18 (`=+F39`), and K20, an MDURATION fed by the YIELD
  (Excel 0.251101533693552, ours 0.2511015337115343 at K15).
These are desktop values (decision 0039: excel-graph is the target), so use them as a check on a fitted rule, not to fit it.

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2257): the 23 visibly-off known-failure lines listed under "T-2257 sweep" in Context disappear
      from eval/baseline/gen-m9.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] A backward-iterate report in the lab (or a test) that reproduces the starting values above.
- [x] Start formulas and stop rules tried, each with its bit-exact and within-4-ulp counts as lab fits
  (decision 0056); a model that raises the bit-exact count, or the task set `blocked` with the candidates
  tried. Findings in financial.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- YIELD's `#NUM!` cells and its 100-step cap (T-2391).

## Evidence (first round, before T-2187)
No start formula fits, so the model cannot be raised from 3/90 with the evidence at hand. The lab now has
`newton_back_chain` (solvers.py). A test pins the shared one-step-back iterate of S!A1 and S!F1:
0.05523446268, equal to 1.1e-11 across frequencies 2 and 1. Candidates tried, with counts in financial.md
"YIELD stops short of PRICE's root":
- Newton with the residual stop from the approximate yield: 1/90.
- Six closed-form families fitted for a span T: none consistent.

The iterate sits about 1.4e-6 below the redemption-100 root and moves by only about 2e-9 at redemption
100.5 (S!E10). That points to an earlier solver phase, not a closed form. Unblocking needs per-iterate
evidence: Excel cells whose YIELD is forced to stop after k iterations do not exist. Two routes:
- A sweep that moves only redemption (100 to 101 in 0.05 steps) and only the frequency at fixed dates,
  to map how the shared iterate depends on each.
- Or the same back-chain analysis on PRICE's inverse with other day-count bases (probe cells with
  basis 1 to 4 and more than one coupon).

Unblocked by T-2187 (2026-10-21). probes/yield-start-sweep is now in `YIELD_CASES`. On its 63 basis-0 cells across
redemption and frequency, analytic Newton run forward from the fixed start 0.0552344627 matches Excel to 1.2e-14 relative.
It returns the iterate after the first step below about 7e-6 (the cut lies between 6.66e-6 and 7.11e-6).
financial.md "YIELD stops short of PRICE's root" has the T-2187 paragraph. Next hypothesis: fit the start S as a
function of rate, price and the basis's time to maturity (it moves with price, column E, and with basis 1/3 against
0/4, column D, but not with redemption or frequency), then this step-size stop.

## New evidence (T-2391)
probes/yield-newton-domain column C (redemptions 1E+50 to 1E+300) gives YIELD values where Newton stops at what looks like
a 100-step cap far from the root. In that regime each step is about (1 + y/f) * f / T, so the final value shows the start.
Back-chaining 100 analytic steps from C1 lands at -0.002179, the redemption-100 root (100/105)^(1/22.4) - 1. Forward from
that root, C1, C6 and C7 come within 8e-7, but rates 0.01 to 0.5 and price 1 are off by 1e-5 to 1e-2. Fitting S there
also decides T-2391's `#NUM!` cells (financial.md "YIELD's #NUM! cells and its suspected 100-step cap").

## Visible-parity stage
Decision 0083: this task is done when every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 must be exactly 0). Criteria above that ask for bit-exact or ulp-level matches and are still unmet then move to a follow-up task (no campaign key), named in the Outcome. Regressions that stay within 4 ulp follow decision 0083's allowance.

Visibly-off cells on the 2026-10-20 eval run (`main` 35a5e71), blamed on this function rather than on glue around it: 139 in total.
- YIELD: 139 cells (13-14 digits right 48, 10-12 66, 6-9 25, fewer or 0 vs nonzero 0); most in `probes/yield-solver-sweep.case.json` 60; `probes/solver-iteration-traces.case.json` 22. e.g. `YIELD(A3,B3,C3,H3,E3,F3,G3)` Excel 0.06499999999999766, ours 0.06499999999999986

Excel stops below the root by up to ~1e-9 relative (financial.md "YIELD stops short of PRICE's root"); 15 digits means reproducing that stop.

## Was blocked because
The block is lifted by T-2740's evidence (below). Was blocked because: no candidate fits YIELD's hidden Newton start S, and the evidence at hand cannot pin it.
Third round (2026-10-24, opus): finite-difference Newton, six variants in the lab's `yield` fit (family "Newton, finite
difference", from a scratch grid of 560), reaches at most 5/178 bit-exact (analytic Newton from 0.1: 4/178), so the
analytic back-chain's S is not an artefact of the derivative. S is not an iterate of Newton on the frequency-free annuity
model at any compounding either (financial.md "YIELD stops short of PRICE's root", third-round paragraph). The 23 gen-m9
visible-parity cells stay. Unblock with evidence that separates the start from the stop rule: T-2330's re-probe of the
stop rule, or a probe that fixes everything but one input (price alone, rate alone, term alone) on the T-2304 grid so S
can be fitted per input.
Evidence since (T-2740, 2026-10-25): probes/yield-start-one-input fixes all but one input. S is two Newton steps back
on both twins, equals the coupon rate at price 100 and is linear in the coupon rate to 6e-9 (financial.md "YIELD stops
short of PRICE's root", T-2740 paragraph). That supports a new hypothesis (a current-yield or yield-approximation
start), so the task is back to `todo`.

## Outcome
Blocked (2026-10-24). The lab's `yield` fit gains the "Newton, finite difference" family (six variants, best 5/178 bit-exact,
14/178 within 4 ulp). No model raises the bit-exact count beyond noise. The findings are in financial.md "YIELD stops short of
PRICE's root" (third-round paragraph). Eval delta: none; the 23 gen-m9 cells stay.

## Progress (fourth round, 2026-10-26, opus)
Fitted: the start S = (100R + (100 - P)/T)/((100 + P)/2 - (100 - P)/(4T)) and the stop (a Newton step below 1e-10 is
not taken). Lab fit `yield-caret`, family "Newton, fitted start": 240/370 bit-exact, 361/370 within 4 ulp (the old best was 4/178).
The engine follows (`yield_start`, `excel_yield_newton`). Eval: 391 known failures fixed (probes 372, gen-m9 13,
ironcalc-calc 6). 3 cells regress by 1 ulp at the root (yield-start-one-input S!A5, yield-start-rate-term S!B34
and S!B35). Those fall under decision 0083's standing allowance, and T-2777 names them. Of the 23 owned gen-m9 cells, 11 are
bit-exact and 9 within 4 ulp. A189 (basis 2), A17 (basis 1), A102 and A194 remain (T-2777).
Resumed 2026-10-26 by an opus run (branch idle 3 h). Pending: recording the baseline with `update-baseline --allow-regressions` under decision 0083's allowance. The
session's permission policy refused that command, so the run needs an explicit go-ahead.
