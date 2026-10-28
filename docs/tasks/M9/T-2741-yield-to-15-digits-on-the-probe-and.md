---
id: T-2741
title: "YIELD to 15 digits on the probe and ironcalc-calc cells T-1592 does not list (PRICE_YIELD.xlsx I column and the YIELD sweeps, 370 visibly-off cells)"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: M
tier: opus
depends_on: [T-1592, T-2740]
campaign: visible
---

## Goal
YIELD with more than one coupon period left reproduces Excel's early Newton stop on every visibly-off YIELD cell of
ironcalc-calc and the 1900-system probes, not just on the 23 gen-m9 cells T-1592 owns. Every cell listed below ends
within half a unit of the 15th significant digit of Excel's value.

## Context
Gap triage 2026-10-25 (`make eval` on `main` 22588cf). In the report's "Visibly off (fifteen digits)" table YIELD
has 481 visibly-off cells (430 numeric, 51 other). The probe and ironcalc-calc cells listed below have no open task
that names them as cells to fix:
- T-1592 (blocked; fit lab, `campaign: visible`) uses the YIELD probes as fitting evidence. Its only cleared
  cells are 23 gen-m9 lines.
- T-2187 and T-2304 (done) added the sweeps. They left "the engine follow-up" to T-1592, which has not filed it.
- `make exit-owners` cannot list them yet: report.json has no exit row for this milestone.
The 13 PRICE_YIELD.xlsx cells count against the milestone's ironcalc-calc exit measure ("Milestone function scope").
None of them is in eval/milestones/exit-exceptions.tsv.

Root cause, one rule (docs/excel-semantics/financial.md "YIELD stops short of PRICE's root"):
- Excel runs Newton on PRICE from a hidden start S that moves with price, coupon rate, and term in the basis's years.
- It returns the iterate after a step-size cut. That cut is open (T-2330).
- So its value sits on the root or up to about 6e-9 relative below it.
- Ours, when this task was cut (`compute_yield_from_price`, removed by T-2391), ran Newton from the coupon rate to 1e-8,
  which lands on the root. Since T-1592 and T-2391 YIELD runs `excel_yield_newton` from `yield_start`
  (crates/rustsheet-engine/src/functions/financial/financial_bonds/yield_solver.rs).
Examples (Excel vs ours):
- ironcalc-calc FINANCIAL/PRICE_YIELD.xlsx PRICE_YIELD!I30 `YIELD(A30,B30,C30,H30,E30,F30,G30)`: 0.06499999999159671
  vs 0.06500000000000004. I4: 0.04999999999976775 vs 0.049999999999999954. I2: 0.0700000000000001 vs
  0.06999999999999997.
- probes/date-system-1900-financial-readers S!E8 `YIELD(DATE(2096,2,15),DATE(2100,2,28),0.05,98,100,2,0)`:
  0.05558592876555125 vs 0.05558592884303243. Its -shifted twin S!E8 is 0.05558592884303248 in Excel; financial.md
  notes the start seems to come from the dates.
- probes/fin-coupon-walk-bounds-c Sheet1!C12 `YIELD(1,43831,0.05,90,100,2)`: 0.05556416365541565 vs
  0.05556416365559217.
- probes/fin-bond-negative-basis Sheet1!E1 `YIELD(39448,43831,0.0575,95.04,100,2,0)`: 0.06346743876940854 vs
  0.06346743876965649.

Owned cells (visibly off on the 2026-10-25 run, YIELD outermost; 370 in all). The three whole-case lines own every
cell of their case, since every failing cell there is a YIELD:
- probes/yield-start-rate-term: the whole case (135 visibly-off cells).
- probes/yield-start-sweep: the whole case (95 visibly-off cells).
- probes/yield-solver-sweep: the whole case (60 visibly-off cells).
- ironcalc-calc/FINANCIAL/PRICE_YIELD.xlsx (13): PRICE_YIELD!I2, I3, I4, I5, I6, I8, I9, I10, I11, I12, I28, I30, I31
- probes/solver-iteration-traces (22): Y!C2, C3, C4, C5, C6, C7, C8, D1, D2, D3, D4, D5, D6, D7, D8, E2, E3, E4, E5,
  E6, E7, E8
- probes/date-system-1900-financial-readers (15): S!A8, A19, A30, B8, B19, B30, C8, C19, C30, D8, D19, D30, E8, E19, E30
- probes/date-system-1900-financial-readers-shifted (10): S!A8, A19, A30, B19, C8, C19, C30, D8, D19, D30
- probes/fin-coupon-walk-bounds-c (8): Sheet1!C12, D12, F12, G12, H12, I12, J12, K12
- probes/fin-remaining-later-argument-ranges (6): Sheet1!I173, I175, I177, I179, I181, I183
- probes/fin-bond-negative-basis (2): Sheet1!E1, G1
- probes/coup-calendar-edges (2): S!J2, J12
- probes/fin-argument-modes (1): Sheet1!H343
- probes/fin-yield-odd-lift (1): Sheet1!I5
Within 15 digits already (a bit-level follow-up, not this task's target): PRICE_YIELD!I29, solver-iteration-traces
Y!C1 and Y!E1, five -shifted cells (S!B8, B30, E8, E19, E30), six yield-solver-sweep cells, 11 yield-start-rate-term
cells and five yield-start-sweep cells.

Approach: take T-1592's fitted start and stop rule into `compute_yield` (done by T-1592) (decision 0056 time box). Then
score every cell above, along with T-1592's 23 gen-m9 cells. If the fit leaves a family of cells visibly off, find
out why from that family, for example the -shifted twin pair or the basis-1 row 19 of the date-system cases. Do not
loosen the rule cell by cell.

## Acceptance criteria
- [ ] Every owned visibly-off cell above is within half a unit of the 15th significant digit of Excel's value. Their
      lines leave eval/baseline/probes.txt and eval/baseline/ironcalc-calc.txt (`make eval-baseline`), and no
      known-failure line is added to any baseline. YIELD cells that move but stay within 4 ulp follow decision 0083's
      allowance.
- [ ] Unit tests in the financial module pin at least PRICE_YIELD!I30, date-system-1900-financial-readers S!E8, its
      -shifted twin S!E8, and one yield-start-rate-term cell, citing them.
- [ ] financial.md "YIELD stops short of PRICE's root" says what the engine implements, with Status and Tasks lines
      updated.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083/0114, stage `visible`: Excel is the inaccurate one here (it stops short of the root), so matching it
means reproducing its solver, not a more accurate kernel. This task is done when every owned cell above is within half
a unit of the 15th significant digit of Excel's value (an expected 0 must be exactly 0). File the cells that stay
bit-level off then, and the "within 15 digits" cells above, in a follow-up with `defer: phase-1b` and no campaign key,
named in the Outcome.

## Out of scope
- Fitting S or the stop rule (T-1592, T-2740, T-2330). The gen-m9 cells T-1592 lists stay T-1592's.
- 1904-system probes (date-system-1904-financial-readers, -1904-financial, -1904-financial-shift). Excel for the web
  changed 1904 financial results after 2026-10-12, so their cached values wait on T-2337's re-fill.
- YIELD's `#NUM!` cells and the 100-step regime of probes/yield-newton-domain (T-2391).
- ODDFYIELD (T-2007), including fin-bond-negative-basis E17 and G17 and coup-calendar-edges J8.
- The T-2436 inner-expression probe cell probes/t2436/320d51765af9b6b7 Calc!A16, which follows its gen-m9 owner.

## Outcome
- Acceptance criterion 1 is met for 367 of the 370 owned cells, not all of them. T-1592's fitted start and stop
  (already in `excel_yield_newton`, from `yield_start`) carry those 367 to within Excel's 15 digits. The other three
  move to T-2974, below. On `make eval` of de48145d, only these YIELD-outermost cells of the owned cases are
  still visibly off:
  - probes/yield-start-rate-term S!B39 and S!B49 (basis 1, 5 and 10 ulp off);
  - probes/yield-solver-sweep S!C7 (27 ulp).
  
  Each sits past a 15th-digit rounding boundary. They are split to **T-2974** (`campaign: visible`), since moving them
  needs a change to the fitted start or stop, which is out of this task's scope (T-1592, T-2740, T-2330). No refit was
  tried here. This task made no engine change, so the eval
  delta is none and no baseline moved.
- Test `fn_yield_matches_excel_off_the_fit` (test_fn_bond_functions.rs) pins four cells:
  - PRICE_YIELD!I30, bit-exact, with its PRICE input;
  - date-system-1900-financial-readers S!E8, its -shifted twin S!E8 and yield-start-rate-term S!B20, each within 1 ulp.
- financial.md "YIELD stops short of PRICE's root" says what the engine implements, with Status and Tasks updated.
- The bit-level residue of the owned cells, including the "within 15 digits" cells listed in Context, is handed to
  **T-2788** (no campaign key), the existing ulp follow-up. Filing a second one would duplicate it.
