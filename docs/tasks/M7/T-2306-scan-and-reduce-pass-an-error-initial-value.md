---
id: T-2306
title: "SCAN and REDUCE pass an error initial value or accumulator to the LAMBDA and keep per-element errors (gen-m7)"
milestone: M7
lane: functions-logic
status: done
priority: P2
size: S
tier: sonnet
campaign: visible
depends_on: []
---

## Goal
SCAN and REDUCE hand an error initial_value, an error accumulator and an error element to the LAMBDA like any other value, as Excel does, instead of returning the first error at once; SCAN keeps one result per element, so a step that is an error spills that error in its slot and the scan goes on.

## Context
Found by the T-2225 triage of eval/suites/gen-m7 (38 cases, 10,991 Excel-scored cells, excel-graph). Quirk entry: docs/excel-semantics/arrays.md "SCAN and REDUCE keep errors in the LAMBDA".

Root cause: `fn_scan` (functions/logical/scan.rs) returns `init` when `init.is_error()` and returns the accumulator as soon as one step is an error; `fn_reduce` (functions/logical/map_reduce.rs) does the same. Excel calls the LAMBDA anyway: `REDUCE(DROP(..)=#CALC!,Data!B8,LAMBDA(z,y,(y/Data!B12)<>CHOOSE(y,z,z)))` is `#DIV/0!` (y/0 is evaluated first; 07d27aa0add0247f A1433), `REDUCE(<#N/A>,..,LAMBDA(acc,x,MONTH(1861)))` is 2 (36809e9faa59553f A556), `REDUCE((C7*ATANH(B7))=#NUM!,B9:E10,LAMBDA(w,v,v))` is the last element " x " (2c5ea078f25443a2 A347), `SCAN(COUNTIFS(..)=#VALUE!,A8:B12,COUNT)` is 1, 1, 2, ... (COUNT of the error accumulator is 0; 90de8755a2e565e6 A1223), `SCAN(JIS(..)=#NAME?,C6,COUNT)` is 1 (9359f03972e94026 A569), and `SCAN(-0.5,B2:E8,LAMBDA(x,acc,(3.14159^acc)/(x/x)))` spills 28 cells whose later elements are `#NUM!` while the first is 7.59e20 (ours returns one `#NUM!`; 36809e9faa59553f A679). Engine check: `ROWS(SCAN(0,{1,2;3,4},LAMBDA(a,b,1/0)))` is `#DIV/0!` in ours, where MAP, BYROW and MAKEARRAY already give 2 rows of per-element errors. The 7d67c559ce7536fa A1073 LET/SCAN is `#SPILL!` in Excel because the full 3x2 scan is blocked; ours stops at `#VALUE!`. Several cells also hold another family's inner call (JIS #NAME?, NUMBERVALUE #N/IMPL! in 2c5ea078f25443a2 A1199, GROUPBY in 36809e9faa59553f A1247): the LAMBDA ignores or consumes the error, so they pass once the error reaches it.

Visibly-off cells (gap triage 2026-10-21, decision 0083): 219 cells, every one with a wrong type, error, text or value (none is a last-digit difference).

Owned gen-m7 cells (T-2225 triage, excel-graph, 2026-10-21): 219 known-failure lines in eval/baseline/gen-m7.txt, 39 formulas. One line per formula: case, anchor, the anchor formula, Excel vs ours at the first failing cell, then every failing cell (sheet Calc; a range is consecutive cells of one row).

- gen-m7/07d27aa0add0247f.case.json A1213 `=SCAN((Data!D$4-(Data!B11*1000000000000000.0)),Data!B3:C7,_xleta.AVERAGE)`: B1213 Excel #VALUE!, ours <empty>; cells (9): B1213, A1214:B1214, A1215:B1215, A1216:B1216, A1217:B1217
- gen-m7/07d27aa0add0247f.case.json A1433 `=REDUCE(DROP(((-0.0001)+Data!$C12),3,0),Data!B8,LAMBDA(z,y,((y/Data!B12)<>CHOOSE(y,z,z))))`: A1433 Excel #DIV/0!, ours #CALC!; cells (1): A1433
- gen-m7/07d27aa0add0247f.case.json A1497 `=SCAN(UPPER(Data!D12),Data!A9:B11,LAMBDA(w,z,((Data!E6+z)>=Data!D12)))`: B1497 Excel #VALUE!, ours <empty>; cells (5): B1497, A1498:B1498, A1499:B1499
- gen-m7/18b80d3fd292d63a.case.json A751 `=REDUCE(SUMIFS(Data!C4:E9,Data!A8:A10,">0",Data!A4:D7,">=2",Data!A6:D7,"a*"),Data!E4:E4,LAMBDA(x,acc,COLUMN()))`: A751 Excel 1.0, ours #VALUE!; cells (1): A751
- gen-m7/2c5ea078f25443a2.case.json A347 `=REDUCE((Data!C7*ATANH(Data!B7)),Data!B9:E10,LAMBDA(w,v,v))`: A347 Excel " x ", ours #NUM!; cells (1): A347
- gen-m7/2c5ea078f25443a2.case.json A1149 `=SCAN(((Data!D12*Data!C3)*(1000000000000000.0-Data!D1)),Data!C8:E11,LAMBDA(v,w,SEC(134217728)))`: B1149 Excel #NUM!, ours <empty>; cells (11): B1149:C1149, A1150:C1150, A1151:C1151, A1152:C1152
- gen-m7/2c5ea078f25443a2.case.json A1199 `=REDUCE(NUMBERVALUE(("1.5"&Data!C1),SORT(Data!B11:C11)),Data!B6:D8,LAMBDA(z,x,ISNONTEXT("ü")))`: A1199 Excel FALSE, ours #N/IMPL!; cells (1): A1199
- gen-m7/36809e9faa59553f.case.json A137 `=REGEXEXTRACT(SCAN(TEXT("ü","x~*"),Data!A8:D8,LAMBDA(z,acc,(acc-Data!D3))),((Data!D5&"#N/A")&PROPER("1")))`: A137 Excel #N/A, ours #VALUE!; cells (4): A137:D137
- gen-m7/36809e9faa59553f.case.json A556 `=REDUCE(VLOOKUP(MUNIT(3),Data!E10:E10,(Data!$C8-Data!B12),(Data!$B2>0.2)),Data!C4:E11,LAMBDA(acc,x,MONTH(1861)))`: A556 Excel 2.0, ours #VALUE!; cells (1): A556
- gen-m7/36809e9faa59553f.case.json A679 `=SCAN((-0.5),Data!B2:E8,LAMBDA(x,acc,((3.14159^acc)/(x/x))))`: A679 Excel 7.590654881982496e20, ours #NUM!; cells (28): A679:D679, A680:D680, A681:D681, A682:D682, A683:D683, A684:D684, A685:D685
- gen-m7/36809e9faa59553f.case.json A1247 `=SCAN((_xlfn.GROUPBY(Data!B1:D12,Data!A1:C12,_xleta.AVERAGE,2,-1,Data!B9:C9,(Data!B1:B12>0.1))^(Data!B$10+2)),Data!A5:B12,LAMBDA(z,w,FLOOR.PRECISE(SECOND(z))))`: B1247 Excel #VALUE!, ours <empty>; cells (15): B1247, A1248:B1248, A1249:B1249, A1250:B1250, A1251:B1251, A1252:B1252, A1253:B1253, A1254:B1254
- gen-m7/36809e9faa59553f.case.json A1531 `=SCAN((TOROW(Data!D6,3)+DROP(Data!E10:E10,1)),Data!C4:C12,LAMBDA(w,acc,SEARCH("?b","",w)))`: A1531 Excel #CALC!, ours #VALUE!; cells (9): A1531, A1532, A1533, A1534, A1535, A1536, A1537, A1538, A1539
- gen-m7/36809e9faa59553f.case.json A1692 `=REDUCE(VLOOKUP((1e-15+Data!A8),Data!B4:D6,1,ISTEXT(NA())),Data!B10:B12,LAMBDA(y,w,MONTH(48210)))`: A1692 Excel 12.0, ours #N/A; cells (1): A1692
- gen-m7/611ce3bbfab27cef.case.json A27 `=REDUCE(INT(PRODUCT(Data!D2:D4,Data!B9,Data!D7:D7)),Data!B3:B11,LAMBDA(acc,y,(ROWS(Data!A7:C12)*y)))`: A27 Excel -15.0, ours #VALUE!; cells (1): A27
- gen-m7/611ce3bbfab27cef.case.json A128 `=SCAN(OR(OFFSET(Data!B11:D12,3,0),Data!D7:D8,ISNUMBER(Data!E6)),Data!B6:D6,LAMBDA(x,v,(MMULT(x,x)>=(x-Data!B$12))))`: B128 Excel #VALUE!, ours <empty>; cells (2): B128:C128
- gen-m7/611ce3bbfab27cef.case.json A311 `=REDUCE(SUM(Data!E5:E10,VLOOKUP(Data!$D8,Data!A6:A11,1e-15,Data!D3)),Data!B2:E7,LAMBDA(z,x,Data!E8))`: A311 Excel 123456789.123, ours #N/A; cells (1): A311
- gen-m7/611ce3bbfab27cef.case.json A412 `=SCAN(EDATE(365,21),Data!A5:C11,LAMBDA(w,z,TEXTAFTER((Data!C7&w),z,(z+Data!E$6))))`: B412 Excel #N/A, ours <empty>; cells (20): B412:C412, A413:C413, A414:C414, A415:C415, A416:C416, A417:C417, A418:C418
- gen-m7/611ce3bbfab27cef.case.json A1163 `=REDUCE((TEXTBEFORE(Data!E$1,",",Data!B9,0,0)&Data!D2),"a",LAMBDA(z,acc,Data!B6))`: A1163 Excel 0.1, ours #N/A; cells (1): A1163
- gen-m7/611ce3bbfab27cef.case.json A1447 `=REDUCE((EOMONTH(17605,-18)/(Data!$E$8+Data!C7)),Data!D6:D7,LAMBDA(z,y,Data!B$11))`: A1447 Excel -2.5, ours #VALUE!; cells (1): A1447
- gen-m7/7307132bb8820f17.case.json A360 `=REDUCE(CONCAT(INDEX(Data!B3:B3,5)),SUM((7+Data!C$3),Data!B2:B7,(1000000000000000.0-Data!D5)),LAMBDA(y,acc,Data!C5))`: A360 Excel 2.5, ours #REF!; cells (1): A360
- gen-m7/7307132bb8820f17.case.json A1187 `=SCAN(SUM((3.14159/Data!A$10),Data!B10:B11,SUM(Data!E4,(-0.5))),FACT(17),LAMBDA(y,v,IFERROR(HLOOKUP("1",Data!D4:E7,Data!B2,Data!$E1),(123456789.123>=v))))`: A1187 Excel FALSE, ours #DIV/0!; cells (1): A1187
- gen-m7/7d67c559ce7536fa.case.json A1073 `=LET(x,(-(-2.5)-(Data!D5/1000000000000000.0)),w,((x-x)^EXP(0)),acc,w,SCAN((Data!C1>=acc),Data!D10:E12,LAMBDA(v,z,DATE(2090,v,13))))`: A1073 Excel #SPILL!, ours #VALUE!; cells (1): A1073
- gen-m7/7d67c559ce7536fa.case.json A1496 `=SCAN((SUM(42,Data!A2)/Data!$C9),Data!A4:D7,LAMBDA(x,z,EOMONTH(2958466,-11)))`: B1496 Excel #NUM!, ours <empty>; cells (15): B1496:D1496, A1497:D1497, A1498:D1498, A1499:D1499
- gen-m7/8044dce50616cfe9.case.json A124 `=SCAN(COMBIN(Data!D1,5),Data!A4:D6,_xleta.MIN)`: B124 Excel #NUM!, ours <empty>; cells (11): B124:D124, A125:D125, A126:D126
- gen-m7/8044dce50616cfe9.case.json A1544 `=SCAN(MINVERSE(Data!C1:E5),Data!A11:B12,LAMBDA(acc,y,REGEXTEST(TEXTBEFORE(acc,acc,-2),"(\w)(\w)",1)))`: B1544 Excel #VALUE!, ours <empty>; cells (3): B1544, A1545:B1545
- gen-m7/8044dce50616cfe9.case.json A1630 `=REDUCE((("x~*"&"Hello World")>-Data!E8),(IFS(Data!E9,Data!E9)<Data!A3),LAMBDA(y,acc,Data!B7))`: A1630 Excel 1.0, ours #VALUE!; cells (1): A1630 (passes since T-2280: IFS's text condition gives a 1x1 `#VALUE!` array; no longer this task's)
- gen-m7/90de8755a2e565e6.case.json A49 `=REDUCE(SQRT((Data!E$8+Data!B6)),Data!A1:C6,LAMBDA(y,acc,CHAR(65)))`: A49 Excel "A", ours #VALUE!; cells (1): A49
- gen-m7/90de8755a2e565e6.case.json A1223 `=SCAN(COUNTIFS(Data!B2:C4,"<>",Data!B4:D6,">0"),Data!A8:B12,_xleta.COUNT)`: A1223 Excel 1.0, ours #VALUE!; cells (10): A1223:B1223, A1224:B1224, A1225:B1225, A1226:B1226, A1227:B1227
- gen-m7/9359f03972e94026.case.json A285 `=SCAN(Data!$A$12,Data!B4:E6,LAMBDA(x,y,(SECH(y)-x)))`: A285 Excel 0.7999999950000001, ours #VALUE!; cells (12): A285:D285, A286:D286, A287:D287
- gen-m7/9359f03972e94026.case.json A569 `=SCAN(JIS(("TRUE"&"ü")),Data!C6,_xleta.COUNT)`: A569 Excel 1.0, ours #NAME?; cells (1): A569
- gen-m7/9359f03972e94026.case.json A1307 `=REDUCE(CHOOSEROWS(DATEDIF(40708,45000.75,"a*"),1,-3),Data!$E9,LAMBDA(acc,w,Data!E7))`: A1307 Excel TRUE, ours #VALUE!; cells (1): A1307
- gen-m7/9359f03972e94026.case.json A1358 `=TEXTBEFORE(REDUCE(SUMPRODUCT(Data!B1:D9,Data!C2:D10,0.5),Data!A3:B11,LAMBDA(acc,w,WEEKNUM(367,(-0.5)))),"-",(Data!A3*1000000000000000.0),Data!$E$3,0,SUM(Data!D1,(Data!$E10/2.5)))`: A1358 Excel #NUM!, ours #VALUE!; cells (1): A1358
- gen-m7/ac67253061bba16a.case.json A568 `=REDUCE(DROP(Data!C6:E7,2,PRODUCT(Data!$A1,123456789.123)),Data!C7:E9,LAMBDA(z,y,y))`: A568 Excel 1000000000000000.0, ours #CALC!; cells (1): A568
- gen-m7/ac67253061bba16a.case.json A1136 `=REDUCE(JIS(FILTER(Data!A1:D12,(Data!B1:B12=0.2))),(NUMBERVALUE(Data!C$12)+MOD(Data!C4,Data!$D1)),LAMBDA(acc,w,TRUNC(61,-5)))`: A1136 Excel 0.0, ours #NAME?; cells (1): A1136
- gen-m7/ac67253061bba16a.case.json A1283 `=SCAN(MAX(TEXTBEFORE(Data!E5,Data!B4:B11,0)),Data!C9:D10,LAMBDA(y,x,((y+Data!D5)*y)))`: B1283 Excel #VALUE!, ours <empty>; cells (3): B1283, A1284:B1284
- gen-m7/ac67253061bba16a.case.json A1567 `=SCAN(MID(SORT(Data!A11:D11,2,Data!B11:D12,Data!$D10),5,6),Data!A4:B9,LAMBDA(z,w,(LET(acc,Data!B$5,y,z,Data!B7)^LOG10(0))))`: A1567 Excel #NUM!, ours #VALUE!; cells (12): A1567:B1567, A1568:B1568, A1569:B1569, A1570:B1570, A1571:B1571, A1572:B1572
- gen-m7/b24ce74f08d3f91a.case.json A714 `=SCAN(DATEVALUE("March 5, 2020"),Data!C10:C12,LAMBDA(y,z,(SUM(0.2,z)+z)))`: A715 Excel #VALUE!, ours <empty>; cells (2): A715, A716
- gen-m7/b24ce74f08d3f91a.case.json A1282 `=SCAN(Data!D$6,Data!A2:D8,LAMBDA(w,z,((Data!A$5/Data!A9)/(Data!D3*z))))`: B1282 Excel #DIV/0!, ours <empty>; cells (27): B1282:D1282, A1283:D1283, A1284:D1284, A1285:D1285, A1286:D1286, A1287:D1287, A1288:D1288
- gen-m7/b24ce74f08d3f91a.case.json A1294 `=REDUCE(PHONETIC(Data!A3:A7),Data!C1:E9,LAMBDA(z,y,(" x "<=123456789.123)))`: A1294 Excel FALSE, ours #N/A; cells (1): A1294

## Acceptance criteria
- [ ] Visible parity: the 219 gen-m7 cells listed under Context have Excel's value, type, error and text: these 219 known-failure lines disappear from eval/baseline/gen-m7.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] The docs/excel-semantics entry cited above states the rule and its "Ours:" line names this task's change.
- [ ] Unit tests cover the rule on a small hand-written input (one error or edge case and one normal case that must keep passing).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Passing the initial value as a reference (T-2307); a LAMBDA that returns an array (T-2309).

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (a wrong type, error, text or value), so this task is in the campaign. Its target is that every cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0); the visible-parity criterion under Acceptance criteria states it.

## Outcome
`fn_scan` and `fn_reduce` no longer return an error initial value or accumulator at once: the LAMBDA gets it, and a SCAN error step fills its slot. `lambda_result_value` turns a multi-cell array step into the one `#CALC!` already given for a multi-cell range (needed so the freed TRIMRANGE cell 2c5ea078f25443a2 A1717 does not spill). It also finished T-2309's six formulas (63 cells). Eval: all 219 listed gen-m7 lines fixed (271 lines in all), 0 regressions; gen-m7 known failures 1339 -> 1068.
