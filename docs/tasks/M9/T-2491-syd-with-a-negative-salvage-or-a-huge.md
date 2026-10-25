---
id: T-2491
title: "SYD with a negative salvage or a huge life, and DB with a huge life or past the period cap (gen-m9, 18 cells)"
milestone: M9
lane: functions-financial
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
SYD and DB return Excel's `#NUM!` (and `#DIV/0!`) for a negative salvage, a huge life and a cost of 0 with a negative salvage, where ours computes a number, and DB returns Excel's value past the period cap where Excel has one.

## Context
Gap triage of the T-2257 gen-m9 financial sweep.
- SYD with a negative salvage is `#NUM!` (`SYD(36746,-100,22,3)`, `SYD(31861,-1E+300,24,2)`, `SYD(0,-439103,37,9)`; ours a
  number, 10 cells); `SYD(34664,230,1E+300,5)` is `#DIV/0!` (ours 0).
- DB with a negative salvage or a huge life is `#NUM!`: `DB(0,-439103,27,10)` (ours 0), `DB(6912,0,1E+300,2,6)` (ours 3456),
  `DB(67486,100,1E15+2,6,8)` (ours 0), `DB(100,843,999999999999994,3,12)` (ours -0).
- `SYD(15023,3.378E13,33,1)` is -1987062826671.328 (ours ...335): SYD's operation order.
- `DB(0.01,488,32,1E+200,10)` is -0.0033416666666666673 in Excel; ours `#NUM!` from the period cap, a DEVIATION
  (financial.md "DB and VDB period cap", decision 0014). Excel answers without walking 1E200 periods, so check whether the
  capped path can return Excel's value (cost below salvage); if not, leave the cell with the DEVIATION entry and say so.
docs/excel-semantics/financial.md "Financial functions in the gen-m9 financial sweep (T-2257)".
Code: `fn_syd`, `fn_db` in crates/rustsheet-engine/src/functions/financial/financial_depreciation.rs.

Owned gen-m9 cells (T-2257 triage, excel-graph, 2026-10-23): 18 known-failure lines in eval/baseline/gen-m9.txt, 18 visibly off.
Examples:
- gen-m9/01523d247594952e Calc!A377 `=SYD(15023,(AVEDEV(Data!A11:A11,{1;2;3;4;5})*F.INV.RT(1E-20,9,3)),33,1)`: Excel -1987062826671.328, ours -1987062826671.3352
- gen-m9/5ded7594e597e31e Calc!A38 `=SYD(34664,230,1E+300,5)`: Excel #DIV/0!, ours 0.0
- gen-m9/762cd0e4ec9a7aa9 Calc!A151 `=DB(0.01,488,32,LARGE({1E+200,1E+200,-1E+200},2),10)`: Excel -0.0033416666666666673, ours #NUM!
- gen-m9/b72d952c5f52cb4d Calc!A303 `=DB(0,(-439103),27,10)`: Excel #NUM!, ours 0.0
- gen-m9/e8d55ec393460bb7 Calc!A338 `=SYD(0,(-439103),37,9)`: Excel #NUM!, ours 18113.779516358463
Cells:
- gen-m9/01523d247594952e.case.json (3): Calc!A13, A41, A377
- gen-m9/23d82b62e9a85bdd.case.json (2): Calc!A131, A173
- gen-m9/31474c20fc1dafd6.case.json (2): Calc!A11, A207
- gen-m9/5c6a23c4f6597f1c.case.json (1): Calc!A270
- gen-m9/5ded7594e597e31e.case.json (1): Calc!A38
- gen-m9/67a4c03afd63f988.case.json (1): Calc!A66
- gen-m9/762cd0e4ec9a7aa9.case.json (2): Calc!A151, A353
- gen-m9/a91bad4e84f49739.case.json (1): Calc!A125
- gen-m9/b72d952c5f52cb4d.case.json (1): Calc!A303
- gen-m9/ba4ddb817a8e700e.case.json (2): Calc!A85, A208
- gen-m9/e168be7f6d3f84c2.case.json (1): Calc!A70
- gen-m9/e8d55ec393460bb7.case.json (1): Calc!A338

T-2436 probe (excel-graph, 2026-10-24; probes/t2436/<case>, each gen-m9 cell's sub-expressions on its Data sheet): `SYD(10156,-439103,25,8)` and `SYD(20252,TRUNC(-0.0001,5),12.5,5)` (negative salvage) are `#NUM!` (ours a number).
Owned gen-m9 cells (T-2436 probe, excel-graph, 2026-10-24): 2 known-failure lines in eval/baseline/gen-m9.txt, 2 formulas.
Examples:
- gen-m9/35e8168aae2b00f8 Calc!A162 `=LOGNORM.DIST(42,SYD(10156,(-439103),25,8),HSTACK((Data!A7-(-2.5)),TEXTJOIN("a",Data!B$7,Data!D8),FIXED(Data!B12,Data!D9,Data!$D2)),2)`: Excel #NUM!, ours #VALUE!; inner `SYD(10156,(-439103),25,8)` Excel #NUM!, ours 24882.036923076925
- gen-m9/f2bd5f380ec002bf Calc!A254 `=GAMMADIST(37.5,SYD(20252,TRUNC(Data!E3,5),12.5,5),(EFFECT(1E+10,1)-(Data!D10-42)),INFO("origin"))`: Excel #NUM!, ours #VALUE!; inner `SYD(20252,TRUNC(Data!E3,5),12.5,5)` Excel #NUM!, ours 2040.2014915555558
Cells:
- gen-m9/35e8168aae2b00f8.case.json (1): Calc!A162
- gen-m9/f2bd5f380ec002bf.case.json (1): Calc!A254

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2436): these 2 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Visible parity (gen-m9, T-2257): these 18 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Unit tests for SYD and DB with a negative salvage, a life of 1E+300 and 1E15, and the SYD value above.
- [ ] `make check` green; reviewers approve.

## Out of scope
- DB and DDB last digits (T-2033).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2257): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.

## Progress notes (T-2491, 2026-10-25)
Data from the run that started this task, to check against the acceptance criteria. Pushed in flight at the session budget.
- Probe probes/fin-syd-db-domain (filled, excel-graph) and its rules in financial.md "DB and SYD argument domains" are on
  this branch; `fn_db` and `fn_syd` implement them. `make eval-baseline` then fixes 21 gen-m9 lines (19 of this task's 20;
  the other two are the A377 below) and 14 probes lines; the probes baseline is recorded, gen-m9's is not yet.
- Open regression, which blocks the gen-m9 baseline: gen-m9/2138b909d8f810ea Calc!A146
  `=PROB(<scalar>,Data!B3:D8,SYD(85972,-439103,18,9),1)`. It is Excel `#VALUE!`; ours is now `#NUM!` (SYD is `#NUM!` as in
  Excel). PROB checks its limits before its data (prob.rs `prob_element`, probes/stat-test-lift P14), but here a limit
  that is an error value loses to the data's `#VALUE!`. That is PROB's error order: probe `PROB(<bad data>,...,NA())` and
  `PROB(<bad data>,...,"a")` to split the error-value and text cases, fix `prob_element`, then `make eval-baseline`.
- gen-m9 01523d247594952e A377 (`SYD(15023,AVEDEV(..)*F.INV.RT(1E-20,9,3),33,1)`) is not SYD: the 30-ulp gap needs a
  salvage 30 ulp off, so it is F.INV.RT's far tail (T-2435). Leave its line, and say so in the Outcome.
- Still to do: unit tests (acceptance), `make check`, review, PR.
