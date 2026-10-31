---
id: T-3256
title: "HYPGEOM.DIST and HYPGEOMDIST last digits on 13 gen-m9 cells no task names (1-2 ulp, after T-2481's pmf)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-2481]
defer: phase-1b
---

## Goal
The 13 gen-m9 HYPGEOM.DIST and HYPGEOMDIST cells listed below, all within 15 significant digits of Excel, match Excel
to the bit once T-2481 has Excel's pmf formulation, or the Outcome names the cells that formulation still misses.

## Context
- Gap triage 2026-10-30 (`make eval` on origin/main 6a26b992a, excel-graph). An ownership check over every failing cell
  (`scripts/exit_owners.py`'s `owned` rule over the task files of origin/main and of every pushed branch) found these
  cells named by no open task. T-2481 (campaign `visible`) is fitting Excel's pmf and owns the visibly-off cells; the
  deferred HYPGEOM task on the unmerged `tasks/m9-opus-fitting-deferred` branch owns the probe sweeps' last digits;
  neither lists these gen-m9 cells.
- Root cause: the same as T-2481's. The engine returns the exact rational rounded once (T-0977, `hypegeom.rs`), and
  Excel's pmf is a few ulp off the exact value (statistics.md "HYPGEOM.DIST is not exact for small integer
  arguments"). Several of these cells show it plainly: `HYPGEOMDIST(0,1,9,10)` (A109, MONTH(1369) = 9) is exactly 1/10 and Excel gives
  0.10000000000000002; A41's pmf of exactly 3/16 (K truncates to 3, cumulative is FALSE) is 0.18750000000000006 in Excel. So the cells pass once T-2481's model is
  in, and this task only checks them and closes the residue. It stays deferred (Phase 1b) because none is visibly off.
- Most cells nest the call (the arguments come from SORT, MONTH, SECH, FLOOR.MATH and the like). The inner calls are
  exact integers or plain values here, so the miss is the pmf, not the arguments: check by evaluating the arguments
  alone (`make eval-show`).

Examples, Excel vs ours (`make eval-show CASE=gen-m9/<case>.case.json`):
- gen-m9/bcb24385baafb2f0 Calc!A109 `=HYPGEOMDIST(0,1,MONTH(1369),10)`: 0.10000000000000002 vs 0.1
- gen-m9/4ab2ba98d26c8a5e Calc!A41 `=HYPGEOM.DIST(1,1,3.14159,16,((""<="ü")>ISERROR(#NUM!)))`: 0.18750000000000006 vs
  0.1875
- gen-m9/8dd2312c96effb1b Calc!A363 (a HYPGEOMDIST of Data cells): 0.9999999999999999 vs 1.0
- gen-m9/eced4bb6a0234c9e Calc!A160 `=HYPGEOMDIST(1,SORT(Data!A1:C1),7,11)`: 0.6363636363636366 vs 0.6363636363636364

Owned cells:
- gen-m9/0a9226016e8f5f33.case.json (1): Calc!A349
- gen-m9/2801c1c11c21e134.case.json (1): Calc!A86
- gen-m9/4ab2ba98d26c8a5e.case.json (1): Calc!A41
- gen-m9/623663cacea53bbc.case.json (1): Calc!A258
- gen-m9/7c20351562a2d71d.case.json (1): Calc!A281
- gen-m9/8d3588d8df0812df.case.json (1): Calc!A20
- gen-m9/8dd2312c96effb1b.case.json (1): Calc!A363
- gen-m9/abc2939ebd1e2365.case.json (1): Calc!A94
- gen-m9/b6593d7dd13f2dac.case.json (1): Calc!A377
- gen-m9/bcb24385baafb2f0.case.json (1): Calc!A109
- gen-m9/e6cf3b20a014e6a3.case.json (1): Calc!A66
- gen-m9/eced4bb6a0234c9e.case.json (1): Calc!A160
- gen-m9/fec39efaba9284c3.case.json (1): Calc!A177

## Acceptance criteria
- [ ] These 13 known-failure lines disappear from eval/baseline/gen-m9.txt (`make eval-baseline`) with no new ones in
      any baseline, or the Outcome lists each cell still off with T-2481's model value next to Excel's.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fitting the pmf (T-2481); the probe sweeps' last digits (the `tasks/m9-opus-fitting-deferred` branch's HYPGEOM task).

## Deferred (decision 0114)
`defer: phase-1b`: the cells this task owns are already within 15 significant digits of Excel, so it waits until Phase 1
exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells visibly off
(then give it a `campaign:` stage instead).
