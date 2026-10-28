---
id: T-2185
title: "Fit VDB's switching schedule from a fractional start period"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: S
tier: opus
campaign: visible
depends_on: []
---

## Goal
VDB with no_switch FALSE and a fractional `start_period` returns Excel's doubles.

## Context
T-1975 fitted VDB's charge order for every no_switch TRUE call and for switching calls from a whole start period
(financial.md "VDB charge order"). Switching from a fractional start still uses the old running-total model
(`vdb_total_depr` in functions/financial/financial_depreciation.rs) and misses probes/fin-vdb-schedule A42-A46.

Lead from the T-1975 lab: a restart model passes 89 of the 92 numeric VDB cells. It takes the depreciation through
`start` (the whole-start schedule, prorating the partial period) as D0, then runs the same schedule from cost `cost - D0`
with straight-line denominator `life - start - p` over the window `[0, end - start]`. It fixes A42, A45, A46 and
A48-A49 but misses A43 (one ulp), A44 (factor 1.5, 244.0757706382978 vs 242.0635...) and fin-vdb-modes A9
(`VDB(2400,300,10,9.5,10)` is 0; the old model passes it with its cap rule). LibreOffice's half-life shift
(start >= life/2 with factor > 1) makes it worse (87/92). More probes around A44 and A9 (factor 1.5, start past the
switch, end at life) should separate the hypotheses.

Visibly-off cells (gap triage 2026-10-21, decision 0083): 3 cells, 3 numeric past half a unit in the 15th digit and 0 with a wrong type, error or text, blamed on VDB 3. Every one is listed above.

T-2256 sweep (gen-m9 part 2, statistical M to Z; gap triage 2026-10-23): VDB passed through SKEW (visibly off).

Owned gen-m9 cells (T-2256 triage, excel-graph, 2026-10-23): 1 known-failure lines in eval/baseline/gen-m9.txt, 1 visibly off.
Examples:
- gen-m9/873f30d2bf76f8db Calc!A360 `=SKEW(Data!C9:C12,(Data!B9+VDB(52287,815,17,Data!$A1,9)),((Data!$E6+Data!A9)/Data!B9))`: Excel 1.7320506771649753, ours 1.732050676707419
Cells:
- gen-m9/873f30d2bf76f8db.case.json (1): Calc!A360

T-2257 sweep (gen-m9 financial; gap triage 2026-10-23): VDB from a fractional start period: `VDB(59720,2.5,26,0.5,6,2)` Excel 20419.595231230724, ours 20478.80080193768; `VDB(41879,589,1023,1.5,9)` and `VDB(97969,872,13,0.0077,0.5)` likewise.

Owned gen-m9 cells (T-2257 triage, excel-graph, 2026-10-23): 3 known-failure lines in eval/baseline/gen-m9.txt, 3 visibly off.
Examples:
- gen-m9/0a957119475760c5 Calc!A102 `=VDB(59720,2.5,26,0.5,6,2)`: Excel 20419.595231230724, ours 20478.80080193768
- gen-m9/1cd3096e3e87c965 Calc!A300 `=VDB(41879,589,1023,1.5,9)`: Excel 608.3659774136107, ours 608.4053728488113
- gen-m9/9a1fa5366bc50918 Calc!A357 `=VDB(97969,872,13,(7^(Data!C12-2.5)),0.5)`: Excel 7411.011840943374, ours 7419.816949901913
Cells:
- gen-m9/0a957119475760c5.case.json (1): Calc!A102
- gen-m9/1cd3096e3e87c965.case.json (1): Calc!A300
- gen-m9/9a1fa5366bc50918.case.json (1): Calc!A357

Last-digit cells of the same sweep (not visibly off; re-check them after the fix):
Owned gen-m9 cells (T-2257 triage, excel-graph, 2026-10-23): 1 known-failure lines in eval/baseline/gen-m9.txt, 1 last-digit (not visibly off).
Examples:
- gen-m9/bc04c1f478c9896c Calc!A336 `=VDB(42790,242,100,(1e-15/(1000000000000000.0*0.5)),7,2)`: Excel 5642.90843237285, ours 5642.908432372851
Cells:
- gen-m9/bc04c1f478c9896c.case.json (1): Calc!A336

Handed over by T-2492. Once T-2492 read the text no_switch, these cells are left with only the fractional-start value,
visibly off:
- gen-m9/762cd0e4ec9a7aa9 Calc!A379 `=VDB(17334,389,36,0.2,9,3,"FALSE")`: Excel 9114.101951893414, ours 9123.7036444456
- probes/t2492-vdb-ddb-arguments A1, A9, A10 (the same call, no_switch FALSE: 9114.101951893414) and A11, A12
  (`VDB(17334,389,36,0.2,9,TRUE,FALSE)`: Excel 4147.934913793104, ours 4148.285714285714)

## Acceptance criteria
- [x] Visible parity (gen-m9, T-2257): the 3 visibly-off known-failure lines listed under "T-2257 sweep" in Context disappear
      from eval/baseline/gen-m9.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] The 1 T-2257 last-digit lines in Context are re-checked after the fix; the Outcome says how many pass.
- [x] Visible parity (gen-m9, T-2256): these 1 known-failure lines (the T-2256 cells in Context) disappear from
      eval/baseline/gen-m9.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] Visible parity: the 3 visibly-off cells listed under Context (2026-10-21) have Excel's value to 15 significant digits, or its type, error and text: the 3 lines in eval/baseline/probes.txt disappear (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] probes/fin-vdb-schedule A42-A46 pass, with fin-vdb-modes still 20/20.
- [x] `make check` green; reviewers approve.

Replacing that path should delete `vdb_total_depr`, whose `straight_line` parameter is now always true (T-1975 review).

## Out of scope
- The no_switch TRUE path and whole start periods (T-1975).

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (VDB from a fractional start period differs from the 3rd digit), so this task is in the campaign. Its target is that every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0), or has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it. Bit-level criteria still unmet when that target is reached move to a follow-up task filed without the key and named in the Outcome (decision 0083).

## Phase 1 triage (T-0090, 2026-10-27)
It owns all 12 visibly-off VDB cells and alone lifts VDB. Priority raised so it leads its stage in M9.

## Outcome
Fitted (2026-10-28) on two new excel-graph probes, fin-vdb-fractional-start (45 cells) and fin-vdb-fractional-start-edges
(14 cells), chosen where the candidate models disagree visibly. Neither the restart model nor LibreOffice's half-life
shift is Excel's: past the switch, `VDB(2400,100,10,6.3,7.3)` is `(591.396864 - 100)/3.7` exactly, which shows Excel
lays the whole schedule on the start's fraction (financial.md "VDB charge order"). The rule: the first `f` of a period
charges `f·min(cost·rate, cost - salvage)` floored at 0, whole periods follow from `f`, and the switch test compares the
straight-line charge `(b - salvage)/(life - t)` with the unclipped `b·rate`. `vdb_switching_between` now takes any
start (whole starts are `f = 0`, unchanged) and `vdb_total_depr` is deleted. The fit was done in a Python model of the
kernel, which the engine reproduces cell for cell.

Results: all 161 switching-mode VDB cells with literal arguments are within visible parity, 153 to the bit. Six gen-m9
lines leave gen-m9.txt: the 3 visibly-off T-2257 lines, the T-2256 line (873f30d2bf76f8db A360), the T-2492 line
(762cd0e4ec9a7aa9 A379) and the T-2257 last-digit line (bc04c1f478c9896c A336, which now passes to the bit). fin-vdb-schedule A42-A46 pass (78/78), fin-vdb-modes 20/20,
t2492-vdb-ddb-arguments 64/64 (its 5 lines leave probes.txt). The 8 cells left one ulp off (fin-vdb-fractional-start
A1, A4, A6, A11, A14, A26, A29, A35, all with a start fraction of .2 or .3) are new-coverage known failures in
probes.txt, owned by the follow-up T-3022 (`defer: phase-1b`, decision 0083). No regressions. Eval delta: gen-m9
-6, probes -10 fixed and +8 new-coverage lines, visible parity 2564 cells visibly off.
