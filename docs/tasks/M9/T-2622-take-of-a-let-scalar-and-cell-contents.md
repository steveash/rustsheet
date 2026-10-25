---
id: T-2622
title: "TAKE of a LET scalar, CELL(contents) in COUNT and MODE.SNGL, XMATCH, and a LAMBDA joined as text is #VALUE! (T-2475 probe)"
milestone: M9
lane: functions-lookup
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
TAKE of a LET-bound scalar is that scalar, CELL("contents") reads as Excel does inside COUNT and MODE.SNGL, and XMATCH's match mode cell is read as Excel does.

## Context
Found by T-2475's probe, probes/t2475-stat-subexpressions (excel-graph, 2026-10-24): each owned gen-m9 cell's innermost diverging sub-expression, evaluated on a copy of the case's Data sheet (D01..; the probe's column B names the cell). Errors are shown as `#<ERROR.TYPE>` (2 `#DIV/0!`, 3 `#VALUE!`, 4 `#REF!`, 6 `#NUM!`, 7 `#N/A`). Record each rule in docs/excel-semantics/ with the probe cells as evidence.

Owned gen-m9 cells (5; known failures in eval/baseline/gen-m9.txt, and the probe's cells in eval/baseline/probes.txt):
- gen-m9/1b31092a0d79e5f1.case.json Calc!A143: probe P!A70 `TAKE(LET(v,D05!A$11,acc,v,y,"?b",acc),-1,-1)`: Excel 0.0, ours "64|1x1|0"
- gen-m9/775d053de498b8c7.case.json Calc!A235: probe P!A279 `(TAKE(D20!C8:E12,D20!B12,D20!$A7)-(D20!A$3/D20!A2))`: Excel "#2", ours "64|1x1|#2"
- gen-m9/92d4fa29e5510f1e.case.json Calc!A98: probe P!A326 `MODE.SNGL(CELL("contents",D26!C$5))`: Excel "#3", ours "#7"
- gen-m9/c2684066d8df263e.case.json Calc!A201: probe P!A491 `COUNT(D38!B2:B10,CELL("contents",D38!C8))`: Excel 3.0, ours 2.0
- gen-m9/eccd03df42e40006.case.json Calc!A21: probe P!A549 `XMATCH(COUNTA(D45!B8),{3;1;4;1;5},(D45!$C1+D45!E8))`: Excel 2.0, ours "#7"

Also: a LET-bound uncalled LAMBDA joined as text (`t&"|"&v`) is `#VALUE!` in Excel, ours `#N/IMPL!` (the probe's bare `LAMBDA(..)` rows, e.g. P!A1 of each LAMBDA cell; arrays.md "An uncalled LAMBDA is one value").

The LAMBDA-as-text rule is inferred from the wrapper (`t&"|"&v` with TYPE 128); add a direct probe cell (`LAMBDA(x,x)&""`)
before implementing it.

## Acceptance criteria
- [ ] The owned gen-m9 cells leave eval/baseline/gen-m9.txt, with the probe's matching cells (`make eval-baseline`), and no new known-failure lines; each rule is in docs/excel-semantics/ with its evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Cells owned by the other T-2475 follow-ups (T-2618 to T-2623).
