---
id: T-1273
title: "Eta-reduced function arguments (_xleta.SUM) in BYROW, BYCOL, MAP, REDUCE, SCAN and MAKEARRAY"
milestone: M7
lane: engine
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A function passed by name as a LAMBDA argument (`BYROW(A1:B3,SUM)`, stored `_xleta.SUM` in xlsx) evaluates like the equivalent LAMBDA in BYROW, BYCOL, MAP, REDUCE, SCAN and MAKEARRAY, getting as many arguments as the LAMBDA would.

## Context
T-1083's probe probes/m7-argument-modes (excel-graph, 2026-10-09; docs/excel-semantics/arrays.md "Dynamic-array and LAMBDA function argument modes"): rows 28 `BYROW(A1:B3,_xleta.SUM)` (3x1, first 11), 48 `MAKEARRAY(2,2,_xleta.SUM)` (2x2, first 2 = SUM(row,col)), 50 `BYCOL(A1:B3,_xleta.MAX)` (1x2, first 3), 51 `SCAN(0,A1:A3,_xleta.MAX)` are `#NAME?` in the engine. xlsx-storage.md "Newer functions need `_xlfn.` in the file, eta arguments `_xleta.`": a bare `SUM` there is `#NAME?`. T-1089 covers exporting `_xleta.`; this task is the parse and evaluation side (an `_xleta.` name becomes a callable function value). Also in the probe: row 44 `ROWS(LAMBDA(x,x))` is 1 in Excel (ours `#VALUE!`) and row 45 a BYROW LAMBDA returning a row is `#CALC!` (ours a 3x1 `#VALUE!`): fix them here if they share the code path, otherwise file them. The rustsheet-gen generator emits `_xleta.` names (T-1083), so gen-m7 scores this too.

More cells (T-1275's gen-m7 suite, excel-graph, 2026-10-11; `./target/release/rustsheet-eval show gen-m7/<case>.case.json`):
`#NAME?` today, Excel values: gen-m7/44ef4f5638e25bcd A7 (`SUM(MAP(..,_xleta.MAX),..)` `#VALUE!`), 59cc50701f4937c9 A4
(`BYROW("",_xleta.PRODUCT)` 0, also needs T-1350), 6db767d720c5b55a A28 (BYROW `_xleta.AVERAGE` inside TEXTBEFORE, `#SPILL!`, also T-1272/T-1361)
and A29 (`MAKEARRAY(3,3,_xleta.AVERAGE)` in REGEXREPLACE, `#SPILL!`, also T-1360), 8ed8a680e9794b77 A19
(`TRUNC(REDUCE(Data!A$3,Data!A2:D8,_xleta.SUM)+DEGREES(-2.5))` 2000000123456790), A23 (`SCAN(..,_xleta.MAX)-0.2` `#SPILL!`), A30
(`IF(SCAN(FILTER(..),..,_xleta.COUNT),..)` `#SPILL!`), 9ff3afcacd3bd0fc A19 (`ERROR.TYPE(MAKEARRAY(1,4,_xleta.SUM))` `#N/A`),
c263b43217a5898b A17 (`..&REDUCE(N(-0.0001),(..="Hello World"),_xleta.AVERAGE)` "3.14159265358979-0.00005") and d38b52bfa1a9f9e6 A39
(`REDUCE((0.3-0.1)*(Data!D$1-Data!C2),Data!D6:D12,_xleta.PRODUCT)` 392.69875).

More cells (T-1289's gen-m9 suite, excel-graph, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
- gen-m9/1466c66c61189a83 Calc!A34 `=BYCOL(AVERAGE(Data!D7:D12),_xleta.MAX)`: Excel -0.04999999999999975, engine `#NAME?`.
- gen-m9/50f10c4062fb2e98 Calc!A32 `=(Data!$B9-SCAN(PERCENTILE.INC(Data!D9:D10,100),(2.5-Data!E3),_xleta.MIN))`: Excel `#NUM!`, engine `#NAME?`.
- gen-m9/7db7ae6cbab32f22 Calc!A3 `=GAMMA.DIST(SCAN("a",(Data!E3<>"TRUE"),_xleta.SUM),..)`: Excel `#VALUE!`, engine `#NAME?`.
- gen-m9/a2e51f48717af2a1 Calc!A3 `=(Data!$E$5&REDUCE((Data!A4^Data!D11),Data!B7,_xleta.MIN))`: Excel "420.999539589003088", engine `#NAME?`.
- gen-m9/b58b6d7cd39d195f Calc!A3 `=(HYPERLINK(..)&LET(acc,Data!$E6,v,SCAN((-0.5),acc,_xleta.AVERAGE),CONCAT(acc,v,acc)))`: Excel "020.752", engine `#NAME?`.
- gen-m9/bc0901ce6193d576 Calc!A8 `=(Data!A12+(BYROW(Data!B8:C10,_xleta.PRODUCT)-..))`: Excel `#SPILL!`, engine `#NAME?`.
- BYROW whose LAMBDA returns a row (row 45's `#CALC!`): gen-m9/4df1b22d3ed63f58 Calc!A35 `=T((BYROW(Data!C6:E11,LAMBDA(x,x))<>COUNTIF(..)))`: Excel `#CALC!`, engine ""; gen-m9/95efb37a9874f82e Calc!A19 `=BYROW(Data!B6:D11,LAMBDA(acc,FISHERINV(RADIANS(acc))))`: Excel `#CALC!`, engine `#SPILL!`.

## Acceptance criteria
- [ ] Probe rows 28, 48, 50 and 51 pass; baseline updated.
- [ ] Rows 44-45 pass or have their own task.
- [ ] gen-m7/44ef4f5638e25bcd A7, 8ed8a680e9794b77 A19, A23, A30, 9ff3afcacd3bd0fc A19, c263b43217a5898b A17 and d38b52bfa1a9f9e6 A39 pass (the other gen-m7 cells listed under Context once their other tasks land). (T-1275's gen-m7 cells listed under Context, `rustsheet-eval show`.)
- [ ] gen-m9/1466c66c61189a83 A34, 50f10c4062fb2e98 A32, 7db7ae6cbab32f22 A3, a2e51f48717af2a1 A3, b58b6d7cd39d195f A3, bc0901ce6193d576 A8, 4df1b22d3ed63f58 A35 and 95efb37a9874f82e A19 pass (the T-1289 gen-m9 cells under Context), with no new failing gen-m9 cell and no new known failures in eval/baseline/.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Superseded by T-1466, which added `_xleta.` names (a LAMBDA whose arity is set by each call, lambda.rs) and BYROW/BYCOL's
`#VALUE!` for anything but a LAMBDA. Probe rows 28, 48, 50 and 51 pass. Of the gen-m7 and gen-m9 cells under Context,
59cc50701f4937c9 A4, 8ed8a680e9794b77 A19, A23, A30, 9ff3afcacd3bd0fc A19, c263b43217a5898b A17, d38b52bfa1a9f9e6 A39 and
the six `_xleta.` gen-m9 cells pass. Left, with their own tasks: rows 44-45 and the BYROW `#CALC!` cells
(gen-m9/4df1b22d3ed63f58 A35, 95efb37a9874f82e A19) and gen-m7/44ef4f5638e25bcd A7 (MAP over single values) in T-1511;
6db767d720c5b55a A28 and A29 wait on T-1272/T-1361 and T-1360.
