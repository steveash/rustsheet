---
id: T-2492
title: "VDB and DDB: a text no_switch, a boolean factor, factor 0, and their error order (gen-m9, 10 cells)"
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
VDB and DDB treat a text or boolean switch and factor as Excel does, accept a factor of 0 and a cost below the salvage where Excel does, and give Excel's error order.

## Context
Gap triage of the T-2257 gen-m9 financial sweep.
- VDB's `no_switch` as text: `VDB(17334,389,36,0.2,9,3,"FALSE")` = 9114.101951893414 (ours `#VALUE!`); with "TRUE", "true"
  or "FALSE" and another argument out of range Excel is `#NUM!` (`VDB(36122,1,26,-7E11,10,3,"TRUE")`,
  `VDB(1,-0.0001,23,1,9,711,"true")`, `VDB(17709,493,0,1E+6,0.693,3,"FALSE")`; ours `#VALUE!`).
- DDB's factor from a TRUE cell: `DDB(3.14159,866,13,7,<TRUE>)` = 0 (ours `#VALUE!`). The salvage is above the cost there,
  so probe a TRUE factor with the cost above the salvage before deciding whether the boolean is read as 1.
- VDB with a factor of 0, `VDB(43478,488,1E+9,3,7,0)` = 0.00017196, and a start and end of 0 with cost below salvage,
  `VDB(0.01,5,25,0,0,3)` = 0 (ours `#NUM!` for both).
- Error order: `VDB(-1,0,39,<#NUM!>,9,10,0)` is `#VALUE!` (ours `#NUM!`), `VDB(73409,713,<#DIV/0!>,<#NUM!>,10)` `#NUM!`
  (ours `#DIV/0!`), `VDB(51470,333,34,2,9,1,<#VALUE!>)` `#NUM!`.
The order and the text rule are inferred from these cells (docs/excel-semantics/financial.md "Financial functions in the gen-m9 financial sweep (T-2257)"); probe the bare calls before changing them.
Code: `fn_vdb`, `fn_ddb` in crates/rustsheet-engine/src/functions/financial/financial_depreciation.rs.

Owned gen-m9 cells (T-2257 triage, excel-graph, 2026-10-23): 10 known-failure lines in eval/baseline/gen-m9.txt, 10 visibly off.
Examples:
- gen-m9/3aa168259335be54 Calc!A237 `=VDB((-1),0,39,ACCRINT(34880,36414,TDIST(40,1,2),-Data!A2,6730,1,4,"FALSE"),9,MONTH(5033),0)`: Excel #VALUE!, ours #NUM!
- gen-m9/5c6a23c4f6597f1c Calc!A192 `=VDB(1,(-0.0001),23,1,9,DATE(ABS(Data!E7),12,11),"true")`: Excel #NUM!, ours #VALUE!
- gen-m9/762cd0e4ec9a7aa9 Calc!A379 `=VDB(17334,389,36,0.2,9,3,"FALSE")`: Excel 9114.101951893414, ours #VALUE!
- gen-m9/7d2979fba2c95d64 Calc!A245 `=VDB(0.01,5,25,Data!B12,Data!C3,3)`: Excel 0.0, ours #NUM!
- gen-m9/81bcdc0bec0c8e6b Calc!A8 `=DDB(3.14159,866,13,7,Data!A$9)`: Excel 0.0, ours #VALUE!
- gen-m9/a91bad4e84f49739 Calc!A323 `=VDB(43478,488,1E+9,3,7,(Data!$C9%-(Data!A$1*Data!$C10)))`: Excel 0.00017196, ours #NUM!
Cells:
- gen-m9/3aa168259335be54.case.json (1): Calc!A237
- gen-m9/5c6a23c4f6597f1c.case.json (1): Calc!A192
- gen-m9/762cd0e4ec9a7aa9.case.json (1): Calc!A379
- gen-m9/7d2979fba2c95d64.case.json (2): Calc!A98, A245
- gen-m9/81bcdc0bec0c8e6b.case.json (1): Calc!A8
- gen-m9/89f0aee953a55a45.case.json (1): Calc!A71
- gen-m9/a91bad4e84f49739.case.json (1): Calc!A323
- gen-m9/bc04c1f478c9896c.case.json (1): Calc!A238
- gen-m9/d0854366830dcdee.case.json (1): Calc!A218

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2257): these 10 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Unit tests for a text no_switch, a boolean factor from a cell, a factor of 0 and the error order cases.
- [ ] `make check` green; reviewers approve.

## Out of scope
- VDB fractional-period values (T-2185) and DDB last digits (T-2033).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2257): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.
