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
Each is a few ulp off, but sits past a rounding boundary of the 15th digit, so it counts as visibly off.
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
