---
id: T-2308
title: "MAKEARRAY: rows or columns that are not whole numbers are #VALUE! (gen-m7)"
milestone: M7
lane: functions-logic
status: in-progress
priority: P2
size: S
tier: sonnet
campaign: visible
depends_on: []
---

## Goal
MAKEARRAY is #VALUE! when rows or columns is not a whole number (2.005, 2.5, 3.14159, 1.005, 11111.11, 38.66), as Excel gives, instead of truncating it and building the array.

## Context
Found by the T-2225 triage of eval/suites/gen-m7 (38 cases, 10,991 Excel-scored cells, excel-graph). Quirk entry: docs/excel-semantics/arrays.md "MAKEARRAY rows and columns must be whole numbers".

Ours truncates: `MAKEARRAY((1.005-MIN(..))=2.005,2,LAMBDA(x,acc,ROUNDDOWN(acc*acc,-6)))` spills 2x2 zeros where Excel is `#VALUE!` (18b80d3fd292d63a A963), `MAKEARRAY(1,Data!E$12=2.5,COUNT)` spills 1x2 (7307132bb8820f17 A1), and where the truncated array does not fit ours is `#SPILL!` (`MAKEARRAY(SUM(B4:B10)=139,Data!E5=1.005,..)`, c36ff5f42a1a4d47 A285). Every MAKEARRAY row below has a non-integer rows or columns, from a direct value, arithmetic, a cell, a 1x1 BYCOL result (18b80d3fd292d63a A395, SQRT of it 11111.11) or a lifted WRAPCOLS whose first element is 3.14159 (c36ff5f42a1a4d47 A1705). Whole numbers keep passing (MAKEARRAY's other gen-m7 cells). Code: `fn_makearray` in functions/logical/bycol_byrow.rs. Check whether a lifted rows/columns element follows the same rule per element (A1705 has Excel `#VALUE!` as its single cell).

Visibly-off cells (gap triage 2026-10-21, decision 0083): 38 cells, every one with a wrong type, error, text or value (none is a last-digit difference).

Owned gen-m7 cells (T-2225 triage, excel-graph, 2026-10-21): 38 known-failure lines in eval/baseline/gen-m7.txt, 11 formulas. One line per formula: case, anchor, the anchor formula, Excel vs ours at the first failing cell, then every failing cell (sheet Calc; a range is consecutive cells of one row).

- gen-m7/18b80d3fd292d63a.case.json A395 `=MAKEARRAY(SQRT(BYCOL(Data!C7:C10,_xleta.MAX)),3,LAMBDA(w,y,((Data!B10^Data!B7)-100)))`: A395 Excel #VALUE!, ours #SPILL!; cells (1): A395
- gen-m7/18b80d3fd292d63a.case.json A963 `=MAKEARRAY((1.005-MIN(Data!C5,(-1),Data!D10:D10)),2,LAMBDA(x,acc,ROUNDDOWN((acc*acc),-6)))`: A963 Excel #VALUE!, ours 0.0; cells (4): A963:B963, A964:B964
- gen-m7/36809e9faa59553f.case.json A1285 `=MAKEARRAY(2,ACOT((Data!C2^Data!C4)),_xleta.MIN)`: A1285 Excel #VALUE!, ours 1.0; cells (2): A1285, A1286
- gen-m7/7307132bb8820f17.case.json A1 `=MAKEARRAY(1,Data!E$12,_xleta.COUNT)`: A1 Excel #VALUE!, ours 2.0; cells (2): A1:B1
- gen-m7/7307132bb8820f17.case.json A1137 `=MAKEARRAY((Data!A3*Data!D6),3,LAMBDA(w,acc,Data!$E$9))`: A1137 Excel #VALUE!, ours "Hello World"; cells (6): A1137:C1137, A1138:C1138
- gen-m7/90de8755a2e565e6.case.json A409 `=MAKEARRAY(1,(LOG10(2^53+1)-(1.005*Data!E5)),_xleta.SUM)`: A409 Excel #VALUE!, ours 2.0; cells (13): A409:M409
- gen-m7/ac67253061bba16a.case.json A1222 `=MAKEARRAY((ATAN2((-2.5),0.2)*(2.5+42)),4,LAMBDA(v,w,UNIQUE(w,LAMBDA(acc,z,v)(w,Data!C10))))`: A1222 Excel #VALUE!, ours #SPILL!; cells (1): A1222
- gen-m7/b24ce74f08d3f91a.case.json A382 `=MAKEARRAY(2,Data!$D12,LAMBDA(v,x,v))`: A382 Excel #VALUE!, ours 1.0; cells (6): A382:C382, A383:C383
- gen-m7/c36ff5f42a1a4d47.case.json A285 `=MAKEARRAY(SUM(Data!B4:B10),Data!E5,LAMBDA(v,acc,LEFT(acc,(Data!A7*Data!A$5))))`: A285 Excel #VALUE!, ours #SPILL!; cells (1): A285
- gen-m7/c36ff5f42a1a4d47.case.json A853 `=MAKEARRAY(((42-0.2)+-3.14159),4,LAMBDA(w,acc,DECIMAL(XLOOKUP((-2.5),Data!D2:D7,Data!A9:D9,w),29)))`: A853 Excel #VALUE!, ours #SPILL!; cells (1): A853
- gen-m7/c36ff5f42a1a4d47.case.json A1705 `=MAKEARRAY(WRAPCOLS(Data!D2:D5,3),4,LAMBDA(z,x,"1.5"))`: A1705 Excel #VALUE!, ours "1.5"; cells (1): A1705

## Acceptance criteria
- [ ] Visible parity: the 38 gen-m7 cells listed under Context have Excel's value, type, error and text: these 38 known-failure lines disappear from eval/baseline/gen-m7.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] The docs/excel-semantics entry cited above states the rule and its "Ours:" line names this task's change.
- [ ] Unit tests cover the rule on a small hand-written input (one error or edge case and one normal case that must keep passing).
- [ ] `make check` green; reviewers approve.

## Out of scope
- SEQUENCE's and RANDARRAY's fractional sizes (they truncate in Excel, arrays.md).

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (a wrong type, error, text or value), so this task is in the campaign. Its target is that every cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0); the visible-parity criterion under Acceptance criteria states it.
