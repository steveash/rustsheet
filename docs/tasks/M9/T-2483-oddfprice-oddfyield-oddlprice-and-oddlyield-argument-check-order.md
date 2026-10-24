---
id: T-2483
title: "ODDFPRICE, ODDFYIELD, ODDLPRICE and ODDLYIELD: argument check order, and ODDLPRICE before the last coupon is #NUM! (gen-m9, 70 cells)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
ODDFPRICE, ODDFYIELD, ODDLPRICE and ODDLYIELD give Excel's result when an argument is an error, text or a boolean and another is out of its domain (Excel's argument order), and ODDLPRICE is `#NUM!` when the settlement is before the last coupon date (it computes a number today).

## Context
Gap triage of the T-2257 gen-m9 financial sweep. Most cells are the argument order: an earlier date out of range, a
settlement after the maturity, a negative rate or yield is `#NUM!` before a later text or error argument
(`ODDLYIELD(-0.0001,39378,-2.5," x ",92,106,0)`, `ODDFPRICE(35391,46118,32997,-1,-0.9999999999,100," x ",1,0)`,
`ODDFYIELD(45412,44474,..,<TRUE>,-1,..)` are `#NUM!`; ours `#VALUE!`). Four cells are ODDLPRICE with the settlement before
`last_interest`, which Excel rejects and ours prices: `ODDLPRICE(59,38163,33447,0,10,110,1,4)`, `ODDLPRICE(4,36674,34791,
-0,100,100,1)`, `ODDLPRICE(36111,38608,36584,..)` and `ODDLPRICE(59,37822,34339,..)` are `#NUM!` (ours 0.85, 0.19, 1.2E-6 and
1938.27). The order is inferred from the generated cells (docs/excel-semantics/financial.md "Financial functions in the gen-m9 financial sweep (T-2257)"): confirm it with a probe of the bare call per
argument position (excel-probe skill) before changing the reader. Cells whose other argument nests a call that may differ
in Excel (SLOPE, GETPIVOTDATA, STDEV.S over a boolean array, FILTER) move to T-2496 if a probe shows that.
Code: `fn_oddfprice`, `fn_oddfyield`, `fn_oddlprice`, `fn_oddlyield` in crates/rustsheet-engine/src/functions/financial/
financial_bonds.rs.

Owned gen-m9 cells (T-2257 triage, excel-graph, 2026-10-23): 68 known-failure lines in eval/baseline/gen-m9.txt, 68 visibly off.
Examples:
- gen-m9/0a957119475760c5 Calc!A351 `=ODDLPRICE(59,38163,33447,0,10,110,1,4)`: Excel #NUM!, ours 0.8456117873158231
- gen-m9/67a4c03afd63f988 Calc!A227 `=ODDLPRICE(2958466,39448,33080,SIN((Data!A$2+0.1)),1E+10,107,1,0)`: Excel #NUM!, ours #VALUE!
- gen-m9/67a4c03afd63f988 Calc!A329 `=ODDFPRICE(35391,46118,32997,(-1),(-0.9999999999),100,Data!$D2,1,0)`: Excel #NUM!, ours #VALUE!
- gen-m9/81bcdc0bec0c8e6b Calc!A389 `=ODDLYIELD(36459,(-1),Data!B4,Data!$C2,94,107,2,YEAR(16575))`: Excel #NUM!, ours #VALUE!
- gen-m9/b72d952c5f52cb4d Calc!A379 `=ODDFPRICE(35327,45347,34426,SLOPE(Data!B$10,Data!$C$11),GETPIVOTDATA("Average of Units",Pivot!$A$1,"Rep","Bo"),(42+PERCENTRANK(Data!A2,Data!$C$11,1.005)),100,1,(GETPIVOTDATA("units",Pivot!$H$3:$I$5,"Rep",Pivot!$H$4)+Data!B$4))`: Excel #VALUE!, ours #DIV/0!
- gen-m9/ba4ddb817a8e700e Calc!A372 `=ODDFYIELD(Data!$E$5,Data!$A1,34025,39712,1E-9,92,107,2)`: Excel #NUM!, ours #VALUE!
Cells:
- gen-m9/0a957119475760c5.case.json (1): Calc!A351
- gen-m9/1b60bf99d71424e7.case.json (5): Calc!A64, A80, A191, A207, A387
- gen-m9/1cd3096e3e87c965.case.json (2): Calc!A43, A223
- gen-m9/1dce4774f5c0e736.case.json (1): Calc!A28
- gen-m9/30e20ba3302fea8b.case.json (3): Calc!A94, A290, A373
- gen-m9/31474c20fc1dafd6.case.json (7): Calc!A6, A55, A130, A136, A202, A277, A332
- gen-m9/3aa168259335be54.case.json (2): Calc!A114, A297
- gen-m9/3c06f9298f65a3ba.case.json (1): Calc!A325
- gen-m9/5422bdbf173fc233.case.json (2): Calc!A156, A254
- gen-m9/5c6a23c4f6597f1c.case.json (4): Calc!A109, A207, A214, A256
- gen-m9/5ded7594e597e31e.case.json (2): Calc!A238, A241
- gen-m9/6376e27be7a05871.case.json (3): Calc!A41, A255, A361
- gen-m9/67a4c03afd63f988.case.json (4): Calc!A145, A227, A329, A341
- gen-m9/762cd0e4ec9a7aa9.case.json (2): Calc!A24, A52
- gen-m9/79d87fc527927626.case.json (3): Calc!A36, A192, A195
- gen-m9/81bcdc0bec0c8e6b.case.json (2): Calc!A211, A389
- gen-m9/89f0aee953a55a45.case.json (5): Calc!A8, A122, A135, A136, A380
- gen-m9/9d9ceba257f8001b.case.json (1): Calc!A121
- gen-m9/a91bad4e84f49739.case.json (1): Calc!A264
- gen-m9/abb7d565af89b849.case.json (6): Calc!A59, A104, A300, A304, A348, A398
- gen-m9/b72d952c5f52cb4d.case.json (3): Calc!A191, A379, A387
- gen-m9/b73c6cc96e093f03.case.json (1): Calc!A397
- gen-m9/ba4ddb817a8e700e.case.json (2): Calc!A102, A372
- gen-m9/bc04c1f478c9896c.case.json (1): Calc!A246
- gen-m9/c0681a8b150ce301.case.json (2): Calc!A19, A54
- gen-m9/d912671baa1364aa.case.json (2): Calc!A94, A257

gen-m9 engineering sweep (gap triage 2026-10-23, T-2258): `IMCOT(MAKEARRAY(1,ODDLPRICE(35184,39240,-0.5,1.005,<" x ">,109,42),..))` and `OCT2DEC(ODDFYIELD(35376,59,..,RANK.AVG(<"ABC">,..)<#VALUE!>,0))` are `#NUM!` (the domain before the text or error; ours `#VALUE!`).
Owned gen-m9 cells (T-2258 triage, excel-graph, 2026-10-23): 2 known-failure lines in eval/baseline/gen-m9.txt, 2 formulas, 2 visibly off.
Examples:
- gen-m9/62c886e21a1042af Calc!A88 `=IMCOT(MAKEARRAY(1,ODDLPRICE(35184,39240,Data!C8,1.005,Data!C10,109,42),_xleta.MAX))`: Excel #NUM!, ours #VALUE!
- gen-m9/7dc52cfcbc8b4448 Calc!A133 `=OCT2DEC(ODDFYIELD(35376,59,VAR.P(Data!D2:D12,Data!$A8,Data!A2),38745,3.14159,80,Data!E$8,RANK.AVG(Data!A9,Data!E6:E7),0))`: Excel #NUM!, ours #VALUE!
Cells:
- gen-m9/62c886e21a1042af.case.json (1): Calc!A88
- gen-m9/7dc52cfcbc8b4448.case.json (1): Calc!A133

T-2436 probe (excel-graph, 2026-10-24; probes/t2436/<case>, each gen-m9 cell's sub-expressions on its Data sheet): `ODDFPRICE(2958466,46506,33984,39398,-1,CONVERT(42,"F","yd")<#N/A>,104,4,2)` is `#NUM!`, a range check before the yld's error (ours `#N/A`).
Owned gen-m9 cells (T-2436 probe, excel-graph, 2026-10-24): 1 known-failure line in eval/baseline/gen-m9.txt, 1 formula.
Examples:
- gen-m9/623663cacea53bbc Calc!A256 `=AVERAGEA(Data!E10:E11,FIXED(COUNTA(Data!A$2),3,(Data!D$3>(-0.5))),ODDFPRICE(2958466,46506,33984,39398,Data!C6,CONVERT(42,"F","yd"),104,4,2))`: Excel #NUM!, ours #N/A; inner `ODDFPRICE(2958466,46506,33984,39398,Data!C6,CONVERT(42,"F","yd"),104,4,2)` Excel #NUM!, ours #N/A
Cells:
- gen-m9/623663cacea53bbc.case.json (1): Calc!A256

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2436): this known-failure line disappears from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Visible parity (gen-m9, T-2258): these 2 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Visible parity (gen-m9, T-2257): these 68 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Unit tests for ODDLPRICE with the settlement before, on and after `last_interest`, and for the argument order of each function.
- [ ] `make check` green; reviewers approve.

## Out of scope
- ODDFYIELD's solver digits (T-2007).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2257): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.
