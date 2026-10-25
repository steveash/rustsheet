---
id: T-2487
title: "TBILLEQ, TBILLPRICE and TBILLYIELD: a settlement below serial 1 is valid, and their error order (gen-m9, 14 cells)"
milestone: M9
lane: functions-financial
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
TBILLEQ, TBILLPRICE and TBILLYIELD accept a settlement serial below 1 (0.1, 0.3, 1E-15), as Excel does, and give Excel's error where an argument is an error and another is out of range.

## Context
Gap triage of the T-2257 gen-m9 financial sweep. financial.md "Analysis ToolPak securities functions check frequency and
dates first" says each date must floor to a serial from 0 to 2958465 and serial 0 passes. Ours is `#NUM!` for a fractional
settlement below 1 where Excel computes: `TBILLPRICE(1e-15,1,100)` = 72.22222222222221, `TBILLYIELD(0.3,1,98)` =
7.346938775510203, `TBILLEQ(0.5,1,0.004166666666666667)` = 0.004224585932707554, `TBILLPRICE(0.1014159,1,12.0242..)` =
96.65993265993265 (8 cells). The other 5 are error order: `TBILLPRICE(0.03,59,<#VALUE!>)` and `TBILLYIELD(1,<#VALUE!>,94)` are
`#NUM!` (ours `#VALUE!`), `TBILLEQ(39717,<#NUM!>,<#DIV/0!>)` is `#VALUE!` and `TBILLEQ(<2E17>,<#N/A>,TRUE)` `#N/A` (ours `#NUM!`),
`TBILLYIELD(36586,1,MAP(..)<#REF!>)` `#VALUE!`; their inner calls (T.TEST, STDEV.S over QUARTILE, COVAR, MAP) may be what
differs, so probe them before changing the order, and move a cell to T-2496 if so. docs/excel-semantics/financial.md "Financial functions in the gen-m9 financial sweep (T-2257)".
Code: `tbill_arguments` in crates/rustsheet-engine/src/functions/financial/discount.rs.

Owned gen-m9 cells (T-2257 triage, excel-graph, 2026-10-23): 13 known-failure lines in eval/baseline/gen-m9.txt, 13 visibly off.
Examples:
- gen-m9/0a957119475760c5 Calc!A309 `=TBILLPRICE(1e-15,1,100)`: Excel 72.22222222222221, ours #NUM!
- gen-m9/5c6a23c4f6597f1c Calc!A32 `=TBILLEQ(FISHERINV((Data!$C$9+Data!A11)),3.14159,0.1)`: Excel 0.10147345009730331, ours #NUM!
- gen-m9/b73c6cc96e093f03 Calc!A145 `=TBILLYIELD(Data!$E7,GETPIVOTDATA("Sales",Pivot!$H$3,"Rep","ANN"),96)`: Excel 15.0, ours #NUM!
- gen-m9/ba4ddb817a8e700e Calc!A135 `=TBILLEQ(Data!A3,1,0.05/12)`: Excel 0.004224585932707554, ours #NUM!
- gen-m9/d0854366830dcdee Calc!A137 `=TBILLPRICE(COUNT(Data!B7:B10)%,59,SUM(Data!E6:E8,BIN2DEC(Data!A6)))`: Excel #NUM!, ours #VALUE!
- gen-m9/e8d55ec393460bb7 Calc!A287 `=TBILLYIELD(0.3,1,98)`: Excel 7.346938775510203, ours #NUM!
Cells:
- gen-m9/0a957119475760c5.case.json (1): Calc!A309
- gen-m9/3c06f9298f65a3ba.case.json (1): Calc!A312
- gen-m9/5c6a23c4f6597f1c.case.json (1): Calc!A32
- gen-m9/6376e27be7a05871.case.json (1): Calc!A297
- gen-m9/81bcdc0bec0c8e6b.case.json (2): Calc!A117, A251
- gen-m9/b72d952c5f52cb4d.case.json (1): Calc!A241
- gen-m9/b73c6cc96e093f03.case.json (1): Calc!A145
- gen-m9/ba4ddb817a8e700e.case.json (1): Calc!A135
- gen-m9/d0854366830dcdee.case.json (1): Calc!A137
- gen-m9/e168be7f6d3f84c2.case.json (1): Calc!A11
- gen-m9/e8d55ec393460bb7.case.json (2): Calc!A91, A287

gen-m9 engineering sweep (gap triage 2026-10-23, T-2258): `IMSIN(DROP(SEQUENCE(1,1),TBILLYIELD(1e-15,100,1E+300),0))` is `#CALC!`, so `TBILLYIELD(1E-15,100,1E+300)` is a number (ours `#NUM!`; gen-m9/bf979a17ab5aafcb A52).
Owned gen-m9 cells (T-2258 triage, excel-graph, 2026-10-23): 1 known-failure lines in eval/baseline/gen-m9.txt, 1 formulas, 1 visibly off.
Examples:
- gen-m9/bf979a17ab5aafcb Calc!A52 `=IMSIN(DROP(SEQUENCE(1,1),TBILLYIELD(1e-15,Data!$C$6,1E+300),0))`: Excel #CALC!, ours #NUM!
Cells:
- gen-m9/bf979a17ab5aafcb.case.json (1): Calc!A52

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2258): these 1 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Visible parity (gen-m9, T-2257): these 13 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Unit tests for a settlement of 1E-15, 0.3 and 0.5 in each of the three functions.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2257): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.
