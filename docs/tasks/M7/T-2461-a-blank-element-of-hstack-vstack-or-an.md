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
probes/dynamic-array-reshape-and-isomitted S!B24:B25, so the fix belongs at INDEX's array read and in VSTACK/EXPAND's blank copy.

## Acceptance criteria
- [ ] The seven t2413-blank-in-stacked-arrays lines above leave eval/baseline/probes.txt, no new known failures.
- [ ] arrays.md's T-2413 paragraph says "Ours" for each.
- [ ] `make check` green; reviewers approve.

## Out of scope
- A lone blank cell reference given to TOROW or UNIQUE (T-2409).
