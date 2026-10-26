---
id: T-2491
title: "SYD with a negative salvage or a huge life, and DB with a huge life or past the period cap (gen-m9, 18 cells)"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: S
tier: opus
depends_on: [T-2468]
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

## Outcome
- Probe probes/fin-syd-db-domain (excel-graph, A1:M6) fixes the rules, recorded in financial.md "DB and SYD argument
  domains": SYD and DB are `#NUM!` for a negative salvage (DB even at cost 0); SYD's `life*(life+1)` overflow is
  `#DIV/0!` and an overflowing numerator `#NUM!`; DB reads life and period as 32-bit integers (a period above i32::MAX is
  period 1, so `DB(0.01,488,32,1E+200,10)` now has Excel's value instead of the period-cap DEVIATION; a life above it is
  `#NUM!`, except the odd u32::MAX "wrapped last year" case for a period below 1 and a month other than 12). The
  "DB and VDB period cap" entry is narrowed to DB periods 100,001 to 2147483647.
- `fn_db`, `fn_syd` implement them; unit test `fn_syd_db_argument_domains`; two older tests that encoded the
  pre-probe values now cite the probe.
- Eval delta: gen-m9 -22 known-failure lines (all 20 owned but A377, plus cb61169f4c033d4b A272, ec2a1d9194a84396 A65 and f066968376660329 A244), probes -12 lines (t2436/t2475
  cells the negative-salvage rule fixes). Depends on T-2468 (PR #1771): gen-m9/2138b909d8f810ea A146
  `PROB(...,SYD(85972,-439103,18,9),1)` is Excel `#VALUE!`; with SYD now `#NUM!` it needs PROB's data-first error order,
  which #1771 lands (checked: the two branches merged pass A146). #1771 merged, A146 still passes, and the gen-m9 baseline is recorded on top of it.
- Left: gen-m9 01523d247594952e A377 (`SYD(15023,AVEDEV(..)*F.INV.RT(1E-20,9,3),33,1)`) is F.INV.RT's far tail, not SYD
  (the 30-ulp gap needs a salvage 30 ulp off; T-2435), so the "SYD value above" criterion belongs there. Probe C1:C2 (DB's
  partial last year) are one ulp off; an order that fixes them regresses DB_DDB G42, filed as T-2697.
