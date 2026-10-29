---
id: T-2974
title: "YIELD basis-1 long terms and the 4-year sweep row miss Excel's 15th digit by 5 to 27 ulp (yield-start-rate-term B39, B49; yield-solver-sweep C7)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
The three YIELD cells that T-1592's fitted Newton still leaves visibly off come within half a unit of the 15th
significant digit of Excel's value.

## Context
Split from T-2741 on 2026-10-28. T-2741 owned 370 visibly-off YIELD cells, and after T-1592's fitted start and stop
all but these three are within 15 digits (`make eval` on de48145d). Excel vs ours:
- probes/yield-start-rate-term S!B39 `YIELD(DATE(2096,3,1),DATE(2104,6,30),0.08,90,100.5,1,0)`: 0.09840382425115844
  vs 0.09840382425115837 (5 ulp).
- probes/yield-start-rate-term S!B49 `YIELD(DATE(2096,3,1),DATE(2126,6,30),0.08,90,100.5,1,0)`: 0.08964236972351812
  vs 0.08964236972351826 (10 ulp).
- probes/yield-solver-sweep S!C7 `YIELD(DATE(2096,3,1),DATE(2100,6,30),0.06,98,100,2,0)`: 0.0653481629462539 vs
  0.06534816294625427 (27 ulp; also one of T-2788's lab cells).
Each is only a few ulp off, but its gap is larger than half a unit in the 15th significant digit, so it counts
as visibly off. B39 shows the same 15 digits as Excel, but its gap of 7e-17 still exceeds the half unit of 5e-17.
The fit lab's "Newton, fitted start" family (`uv run rustsheet-fitlab yield-caret`) scores all three. First find
whether the miss is in the start, in the stop, or in PRICE's sum near the root. T-2788 records the PRICE summation
orders already tried.
Semantics: financial.md "YIELD stops short of PRICE's root". Code: `excel_yield_newton` and `yield_start` in
crates/rustsheet-engine/src/functions/financial/financial_bonds/yield_solver.rs.

## Acceptance criteria
- [ ] The three cells are within half a unit of Excel's 15th digit, and no known-failure line is added to any
      baseline (`make eval-baseline`), or the task records the candidates tried with their lab counts (decision
      0056 time box).
- [ ] financial.md "YIELD stops short of PRICE's root" records the finding.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Bit-level residue within 15 digits (T-2788).
- YIELD's `#NUM!` cells and its 100-step cap (T-2391).

## Cells added by the T-0087 sweep (2026-10-28)
Visibly-off Excel cells no open task owned on the eval of 2026-10-28 at `main` c2b6143a (`python3 scripts/exit_owners.py --all-off`; calipers, poi and corpus-bench not fetched): YIELD 1. This task now owns them:
- probes/yield-newton-domain: P!B2

## Cells added by probes/yield-price-at-iterates (2026-10-29)
Its column A repeats owned YIELD cells, one row per bond: P!A1 (yield-solver-sweep S!C7), P!A2 and P!A3
(yield-start-rate-term S!B39, S!B49) and P!A4 (yield-newton-domain P!B2) belong to this task; P!A5 to P!A12 repeat
T-2788's ulp-level cells (yield-start-one-input B54, B62, B69, B11, A54, A62; yield-solver-sweep C11;
solver-iteration-traces Y!E2). P!B4, C3, C7 and D4 (PRICE) belong to T-3144.

## Cells added by probes/yield-one-step-start (2026-10-29)
This task owns the case's YIELD cells the engine misses (158 known failures, 1 to 67 ulps). The visibly-off one:
- probes/yield-one-step-start: S!A68

## Outcome (partial, attempt 1, 2026-10-29; the task stays open, decision 0056)
probes/yield-price-at-iterates (59 PRICE cells, excel-graph) asks Excel's PRICE at the exact iterates of our Newton and
at Excel's YIELD values. The lab's `^`-priced PRICE equals Excel's to the bit on all 59 (the engine's misses 4 by 1 to 2 ulps: follow-up T-3144), so the three cells (and P!B2) are not off through PRICE:
Excel's iterates differ from ours. On C7, which steps once from the start, Excel's bits need a start 2e-10 to 1.5e-9
relative below the fitted one. The others ride on the rounding of the iterate before the last step. Tried in the lab
and not kept (counts in financial.md "YIELD stops short of PRICE's root", T-2974 paragraph): 51 PRICE summation and
slope forms (best 240/370, the engine's), 8 residual structures, and about 20 arrangements of the start and its term.
The best of these, `pr + (0.5 - 0.25/T)(100 - pr)`, fixes P!B2 but nets +2 of 570 and makes ODDFYIELD sweep S!C92
visibly off. Next attempt: fit the start to 1e-9 on cells that step once from it. Eval delta: one new probe case,
73 cells, 57 passing. The 16 new known failures are 4 PRICE cells 1 to 2 ulps off (T-3144) and 12 YIELD cells in column
A, which repeat existing cells. No regressions.

## Outcome (partial, attempt 2, 2026-10-29; the task stays open, decision 0056)
- The one-step cells show the start's last bit, so the start was refit on its rounding. Excel's start is the fitted
  formula rounded once over the exact term: lab bit-exact 503/761, against 441 for the formula in doubles. Every
  arrangement and its score is in financial.md "YIELD stops short of PRICE's root", T-2974 attempt 2.
- The engine's `yield_start` computes that rounded-once start in double-double. Eval delta: 256 YIELD/ODDFYIELD known failures fixed,
  148 YIELD/ODDFYIELD cells regressed (9 visibly off, decision 0162, follow-up T-3152). In-scope visibly-off cells
  go from 2,885 to 2,862.
- Still off: yield-solver-sweep C2, C7 and C8 and yield-start-rate-term B39 need a start one ulp above the correctly rounded one. B49 has Excel's start but stays off on
  the engine's PRICE (T-3144). P!B2 is unchanged.
- Next: rounding up fits C7, B39 and B49. Look for what moves those cells up and not the others, for example a second
  rounding of the denominator.

## Outcome (partial, attempt 3, 2026-10-29; the task stays open, decision 0056)
- New evidence: probes/yield-one-step-start (360 cells, excel-graph). Its bonds take exactly one Newton step, so each
  value shows the start's last bit (280 distinct starts). Frequency twins always agree on the start.
- The lab's bit-exact count rose with a named candidate. The best fit for Excel's start is the term-scaled formula in doubles with one
  fused multiply-add, `((r·T)·100 + q) / (fma(pr, T, 100·T)/2 - q/4)`: 229 of the 290 one-step starts, against 152 for the
  engine's rounded-once start. The full lab has 787 of 1,101 YIELD probe cells bit-exact against 704. On the groups
  whose term is a double exactly it matches all 63 starts. Every candidate and its counts are in financial.md "YIELD
  stops short of PRICE's root", T-2974 attempt 3.
- Not ported to the engine: on the older probe cells the form scores 488 against 503. Its losses are on the
  4 + 119/360 bond, one of the two groups where `100·T` is a rounding tie. A tie rule fixes yield-solver-sweep C2, C7,
  C8 and yield-newton-domain P!B2 (lab 936 of 1,101), but its mechanism is unknown, so it was not kept.
  yield-start-rate-term B39 stays 5 ulps off under every candidate.
- Next: find what the tie groups do to `pr·T`, then put the FMA form in `yield_start`. ODDFYIELD shares the start.
- Eval delta: one new probe case, 360 cells. The engine (rounded-once start) passes 202. The 158 new known failures
  are YIELD cells of that case 1 to 67 ulps off. No other cell changed and nothing regressed.
