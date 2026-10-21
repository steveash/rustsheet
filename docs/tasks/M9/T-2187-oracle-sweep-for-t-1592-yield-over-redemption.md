---
id: T-2187
title: "Oracle sweep for T-1592: YIELD over redemption, frequency and day-count basis at fixed dates, with the Newton back-chain per cell"
milestone: M9
lane: research
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Excel values that show how YIELD's hidden earlier iterate depends on redemption, frequency and day-count
basis when the dates stay fixed. Each new cell gets its Newton back-chain in the fit lab, so T-1592 can try
start and stop models again with evidence it does not have today.

## Context
Gap triage 2026-10-20. YIELD is the largest visibly-off group with no ready task: 148 visible cells of 169
failing (report "Functions with failing cells"), most in probes/yield-solver-sweep (60) and
probes/solver-iteration-traces Y!C:E (22), plus probes/date-system-1900-financial-readers(-shifted) and
-1904-financial-readers rows 8, 19, 30 (basis 0, 1, 4). Example: `YIELD(DATE(2096,3,1),DATE(2100,6,30),0.05,98,100,2,0)`
is 0.05523587030251456 in Excel and 0.055235870307021616 for us. The fitting task T-1592 (`campaign: visible`) is
`blocked`: no start formula fits the 90 basis-0 cells (at most 3/90 bit-exact). Its "Blocked because" names
the evidence it needs, and no research task asked for it until now.
- The one-step-back iterate is the same for S!A1 (frequency 2) and S!F1 (frequency 1), 0.05523446268, equal to
  1.1e-11. It moves by only about 2e-9 at redemption 100.5 (S!E10), and three steps back the price-50 bonds
  Y!C2:E2 meet near 0.1333515 (financial.md "YIELD stops short of PRICE's root").
- The cached cells sample redemption only at 90, 95, 98, 99, 100, 100.5, 101, 105, 110, 120, 50 and 200
  (S!E1:E12), frequency only at 1 and 4 for three years (S!F1:F6), and bases 1 and 4 only on the date-system rows.
  That is too coarse to tell an earlier solver phase (a bracket or bisection that stops near the redemption-100
  root) from a start that depends on the inputs. So this is an oracle sweep, not a blind fit (the gap-triager's
  numeric-fitting rule).

Sweep design (one probe case, `eval/suites/probes/cases/yield-start-sweep.case.json`, tag `M9`, `"expected": {}`;
follow the `excel-probe` skill). The base bond is S!A12's, `YIELD(DATE(2096,3,1),DATE(2100,6,30),0.05,98,R,F,B)`:
- A: redemption R from 100 to 101 in steps of 0.05 (21 cells), F = 2, B = 0.
- B: the same 21 redemptions at F = 1, and C: at F = 4 (the shared-iterate test across frequencies).
- D: basis B = 1, 2, 3, 4 at F = 1, 2, 4 with R = 100 (12 cells, all with more than one coupon left).
- E: price 50, 60, 70, 80, 90, 95, 97, 98, 99, 99.5 at R = 100, F = 2, B = 0 (10 cells). This joins S!B and
  Y!C:E at the same dates.
- F: the S!A1 dates, `YIELD(DATE(2000,3,1),DATE(2004,6,30),...)`, with R = 100, 100.25, 100.5, 100.75 and 101 at F = 1, 2 and 4 (15 cells).
That is about 100 cells, all one formula shape, so the oracle run is cheap. Fill them with
`cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes`. If no backend is available, leave the
case pending and set this task `blocked` on the oracle (excel-probe step 3).

Analysis: run `newton_back_chain` (tools/src/rustsheet_tools/fitlab/kernels/solvers.py) on every new cell
that is below PRICE's root, 1 to 3 steps back. Tabulate, per column, the earliest iterate and how it moves
with R, F, B and price. Add the table, or its summary with the lab command that prints it, to
docs/excel-semantics/financial.md "YIELD stops short of PRICE's root". The lab's `yield` family then scores
the new cells as well (`uv run rustsheet-fitlab yield`).

## Acceptance criteria
- [ ] probes/yield-start-sweep.case.json and its `.oracle.json` (excel-graph, canary-verified) are committed
      with columns A to F as above. Its known failures are recorded in eval/baseline/probes.txt with
      `make eval-baseline`, and no other baseline line changes.
- [ ] The fit lab's `yield` cell set includes the new case (`YIELD_CASES` in fitlab/kernels/solvers.py), and a lab test pins the back-chain iterate of at
      least A1 (R = 100, F = 2) and the matching B and C cells.
- [ ] financial.md's YIELD entry states, citing cells, whether the earliest back-chain iterate depends on R,
      F, B and price, and names the next hypothesis for T-1592. T-1592's "Blocked because" is updated to point
      at this evidence, and it is set back to `todo` if the evidence supports a new hypothesis.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fitting YIELD or any engine change (T-1592 and the engine follow-up it files).
- ODDFYIELD's solver (T-2007) and the discount securities (T-2032).
