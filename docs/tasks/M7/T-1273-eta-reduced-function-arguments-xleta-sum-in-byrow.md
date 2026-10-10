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

## Acceptance criteria
- [ ] Probe rows 28, 48, 50 and 51 pass; baseline updated.
- [ ] Rows 44-45 pass or have their own task.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
