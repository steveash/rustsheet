---
id: T-2391
title: "Probe and match YIELD at a price far above par (1E6): #NUM!, not a negative yield"
milestone: M9
lane: functions-financial
status: in-progress
priority: P2
size: S
tier: opus
depends_on: [T-1592]
campaign: semantics
---

## Goal
`YIELD` at a price far above par gives Excel's `#NUM!` where Excel has no root, not a negative yield.

## Context
- Found by the T-2248 smoke check (one formula per financial edge-pool entry, excel-graph, 2026-10-22; the scratch
  suite was deleted, so the probe is this task's to commit). docs/excel-semantics/financial.md "Financial edge-pool
  smoke check (T-2248)" lists the observed values.
- Observed: `YIELD(39448,43831,0.05,1E+6,100,2)` and `YIELD(39448,43831,0.05,1E+300,100,2)` are `#NUM!` in Excel; ours
  -0.6330938491144911 for the first. `YIELD(...,200,100,2)` is -0.02224401627496982 in Excel (ours close).
- Probe a price sweep between 200 and 1E6 (and other coupon counts) to find where Excel's solver gives up: a yield bound or
  an iteration cap. financial.md "YIELD stops short of PRICE's root" and "Yield
  and odd-coupon work cap" describe the solver. Code: functions/financial/financial_bonds.

T-2257 sweep (gen-m9 financial; gap triage 2026-10-23): YIELD is `#NUM!` in Excel at a coupon rate far above the yield, where ours finds a root: `YIELD(38930,46538,1E+300,116,0.01,1,1)`, `YIELD(39779,45574,1E+10,86,101,1,3)`, `YIELD(38829,45714,100,0.25,2,4,4)`, `YIELD(36665,44491,42,106,42,4,0)`, `YIELD(59,45670,100,91,103,4,3)` (ours 8.65, 10.32, 19.18, 16.22, 17.12), 18 cells, and at a negative-yield end `YIELD(39572,45740,1E-10,97,1,4.9)` (ours -0.26). The reverse, Excel computing where ours is `#NUM!`: `YIELD(37341,45513,1E-9,105,1E+300,1)` = 78.13551092802992 and `YIELD(43831,45412,10,1E15+1,1E+300,2)` = 88233.09573268576. These widen the task's question from the price to the coupon and redemption; probe both.

Owned gen-m9 cells (T-2257 triage, excel-graph, 2026-10-23): 20 known-failure lines in eval/baseline/gen-m9.txt, 20 visibly off.
Examples:
- gen-m9/5c6a23c4f6597f1c Calc!A348 `=YIELD(59,45670,100,91,103,4,3)`: Excel #NUM!, ours 17.122634462637862
- gen-m9/7d2979fba2c95d64 Calc!A352 `=YIELD(37341,45513,1E-9,105,1E+300,1)`: Excel 78.13551092802992, ours #NUM!
- gen-m9/e168be7f6d3f84c2 Calc!A390 `=YIELD(43831,45412,10,MROUND(999999999999999,7),1E+300,2)`: Excel 88233.09573268576, ours #NUM!
Cells:
- gen-m9/1cd3096e3e87c965.case.json (3): Calc!A212, A261, A310
- gen-m9/1dce4774f5c0e736.case.json (2): Calc!A256, A305
- gen-m9/30e20ba3302fea8b.case.json (1): Calc!A255
- gen-m9/31474c20fc1dafd6.case.json (1): Calc!A321
- gen-m9/3aa168259335be54.case.json (1): Calc!A327
- gen-m9/5c6a23c4f6597f1c.case.json (2): Calc!A152, A348
- gen-m9/762cd0e4ec9a7aa9.case.json (1): Calc!A178
- gen-m9/7d2979fba2c95d64.case.json (1): Calc!A352
- gen-m9/9a1fa5366bc50918.case.json (1): Calc!A133
- gen-m9/abb7d565af89b849.case.json (1): Calc!A25
- gen-m9/b6a02893a8d6b151.case.json (1): Calc!A29
- gen-m9/ba4ddb817a8e700e.case.json (2): Calc!A213, A262
- gen-m9/bc04c1f478c9896c.case.json (1): Calc!A121
- gen-m9/e168be7f6d3f84c2.case.json (2): Calc!A145, A390

## First attempt (T-2391, 2026-10-24)
probes/yield-newton-domain is committed and filled (excel-graph): the price sweep's cut lies between 1E4 (computes, -0.340)
and 1E5 (`#NUM!`), and the 100-step cap returns non-roots at very large redemptions. No solver model fitted all of it.
Newton from the coupon rate with no fallback fixes the 18 `#NUM!` cells but breaks prices 1500 to 10000, which compute
today through the bisection fallback. Step halving and fixed restarts fail the other way. Details and counts:
financial.md "YIELD's #NUM! cells and its suspected 100-step cap". Both groups seem to come from YIELD's unfitted start phase, so
this task now waits on T-1592. Restart from the probe's C column, where about 100 analytic-Newton steps from near the
redemption-100 root land within 1e-6 of Excel.

Moved from T-2482 (2026-10-24): YIELD at a coupon rate of 999999999999995 is `#NUM!` in Excel, ours 10.571526228899263
(probes/t2436, T-2436); its arguments are in range, so only the solver's limit is left.
Owned gen-m9 cells (T-2482, excel-graph, 2026-10-24): 1 known-failure line in eval/baseline/gen-m9.txt, 1 visibly off.
- gen-m9/320d51765af9b6b7 Calc!A329 `=CHITEST(YIELD(36563,45559,SUM(Data!B11:B12,Data!B$8),107,107,2,1),(DAY(43015)-ODDLPRICE(35397,38233,33415,1E-300,Data!B2,109,4)))`: Excel #NUM!, ours #N/A

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2482): the known-failure line listed under "Moved from T-2482" disappears from
      eval/baseline/gen-m9.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Visible parity (gen-m9, T-2257): the 20 visibly-off known-failure lines listed under "T-2257 sweep" in Context disappear
      from eval/baseline/gen-m9.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] A probe case sweeps the price to locate the cut, filled through excel-graph (probes/yield-newton-domain).
- [ ] YIELD matches Excel's `#NUM!` cells; the financial.md entry states the rule (VERIFIED).
- [ ] `make check` green; reviewers approve.

## Out of scope
- YIELD's last-digit misses ("YIELD stops short of PRICE's root").

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2257): the gen-m9 cells listed under "T-2257 sweep" in Context are visibly off (wrong
type, error, text, or a value off by more than half a unit of the 15th significant digit), so the task is in the campaign. Its
target is that every one of them has Excel's type, error and text, or is within half a unit of the 15th significant digit of
Excel's value; the T-2257 visible-parity criterion under Acceptance criteria states it. Bit-level criteria still unmet then move
to a follow-up task filed without the key.
