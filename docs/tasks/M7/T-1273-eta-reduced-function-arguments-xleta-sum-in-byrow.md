---
id: T-1273
title: "Eta-reduced function arguments (_xleta.SUM) in BYROW, BYCOL, MAP, REDUCE, SCAN and MAKEARRAY"
milestone: M7
lane: engine
status: todo
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
(`BYROW("",_xleta.PRODUCT)` 0, also needs T-1350), 6db767d720c5b55a A28 (BYROW `_xleta.AVERAGE` inside TEXTBEFORE, `#SPILL!`, also T-1272/T-1346)
and A29 (`MAKEARRAY(3,3,_xleta.AVERAGE)` in REGEXREPLACE, `#SPILL!`, also T-1360), 8ed8a680e9794b77 A19
(`TRUNC(REDUCE(Data!A$3,Data!A2:D8,_xleta.SUM)+DEGREES(-2.5))` 2000000123456790), A23 (`SCAN(..,_xleta.MAX)-0.2` `#SPILL!`), A30
(`IF(SCAN(FILTER(..),..,_xleta.COUNT),..)` `#SPILL!`), 9ff3afcacd3bd0fc A19 (`ERROR.TYPE(MAKEARRAY(1,4,_xleta.SUM))` `#N/A`),
c263b43217a5898b A17 (`..&REDUCE(N(-0.0001),(..="Hello World"),_xleta.AVERAGE)` "3.14159265358979-0.00005") and d38b52bfa1a9f9e6 A39
(`REDUCE((0.3-0.1)*(Data!D$1-Data!C2),Data!D6:D12,_xleta.PRODUCT)` 392.69875).

## Acceptance criteria
- [ ] Probe rows 28, 48, 50 and 51 pass; baseline updated.
- [ ] Rows 44-45 pass or have their own task.
- [ ] gen-m7/44ef4f5638e25bcd A7, 8ed8a680e9794b77 A19, A23, A30, 9ff3afcacd3bd0fc A19, c263b43217a5898b A17 and d38b52bfa1a9f9e6 A39 pass (the other gen-m7 cells listed under Context once their other tasks land). (T-1275's gen-m7 cells listed under Context, `rustsheet-eval show`.)
- [ ] `make check` green; reviewers approve.

## Out of scope
-
