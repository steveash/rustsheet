---
id: T-2461
title: "A blank element of HSTACK, VSTACK or an EXPAND pad is the empty value through INDEX, VSTACK of a blank cell and an EXPAND blank pad (t2413 probe cells)"
milestone: M7
lane: functions-logic
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A blank cell taken into an array value (HSTACK, VSTACK, an EXPAND pad) is Excel's empty value wherever a single element is read:
ISBLANK TRUE, `&` "", COUNT, COUNTA and AVERAGE count it as 0.

## Context
Split from T-2413 (probes/t2413-blank-in-stacked-arrays, excel-graph, 2026-10-23; arrays.md "SCAN and REDUCE keep errors in the
LAMBDA", T-2413 paragraph). T-2413 fixed the LAMBDA element binding only. The remaining known failures:
- INDEX of an array element: P!A2, P!A3, P!A6 (`COUNT(INDEX(HSTACK(D!Z1:Z2,{1;2}),1,1))` is 1, ours 0; AVERAGE(..,1) 0.5, ours 1).
- VSTACK of one-cell blank references makes 0, not the empty value: P!A13 (ISBLANK TRUE, ours FALSE), P!A16 (`&"|"` "|", ours "0|"), P!A20.
- EXPAND's pad from a blank cell makes 0: P!A32 (ISBLANK of the pad elements TRUE, ours FALSE).
Making `array_node_to_calc_result` give `CalcResult::EmptyArg` for every caller was tried and regresses TEXTJOIN over arrays and
external ranges (probes/textjoin-array-arguments A10, A18, A22; fixtures/external-link-cached-ranges Refs!B76), IFS
(probes/t2280-ifs-one-cell-range-condition A61:A65), TOCOL (gen-m7/7d67c559ce7536fa A692..) and
probes/dynamic-array-reshape-and-isomitted S!B24:B25, so the fix belongs at INDEX's array read and in VSTACK/EXPAND's blank copy. T-2413's `ArrayNode::Empty => CalcResult::EmptyArg` arm in `lambda_cells`
(functions/logical/bycol_byrow.rs) goes if the fix moves into a shared read.

T-2436 probe (excel-graph, 2026-10-24; probes/t2436/<case>, each gen-m9 cell's sub-expressions on its Data sheet): `SEARCH(HSTACK(<blank>),"Hello World",8)` is 8: HSTACK of a blank cell is the empty value, read as "" (ours 0, so `#VALUE!`).
Owned gen-m9 cells (T-2436 probe, excel-graph, 2026-10-24): 1 known-failure line in eval/baseline/gen-m9.txt, 1 formula.
Examples:
- gen-m9/2a18b2f5a4dc43ac Calc!A79 `=EXPONDIST(Data!C12,SEARCH(HSTACK(Data!B$5),"Hello World",8),BYCOL(Data!A11:D11,LAMBDA(w,REDUCE(w,w,LAMBDA(x,v,1)))))`: Excel 0.9816843611112658, ours #VALUE!; inner `SEARCH(HSTACK(Data!B$5),"Hello World",8)` Excel 8.0, ours #VALUE!
Cells:
- gen-m9/2a18b2f5a4dc43ac.case.json (1): Calc!A79

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2436): this known-failure line disappears from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] The seven t2413-blank-in-stacked-arrays lines above leave eval/baseline/probes.txt, no new known failures.
- [ ] arrays.md's T-2413 paragraph says "Ours" for each.
- [ ] The probe also reads (cells added to probes/t2413-blank-in-stacked-arrays or a new case): MAP/REDUCE COUNT over blanks
      that SORT, FILTER and TAKE put into an array (T-2413 binds every blank array element as the empty value, unprobed for
      these sources); `MAP(HSTACK(Z1:Z2,{1;2}),LAMBDA(v,ISOMITTED(v)))` and `LAMBDA(x,ISOMITTED(x))(IFERROR(Z1,1))` (ours TRUE;
      a task if Excel is FALSE); the spill of `MAP(HSTACK(Z1:Z2,{1;2}),LAMBDA(v,v))` (ours 0).
- [ ] `make check` green; reviewers approve.

## Out of scope
- A lone blank cell reference given to TOROW or UNIQUE (T-2409).
