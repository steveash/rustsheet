---
id: T-2292
title: "Probe per-element calls over arrays of references: error elements, criteria arrays beside a grid, VSTACK of several grids, IF/LET pass-through"
milestone: M6
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's values are known for the per-element cases T-2064 implemented without evidence, so the engine can be held to them.

## Context
T-2064 (SPEC-0014 rule 6) lists these as unprobed in docs/excel-semantics/lookup.md "OFFSET, INDIRECT and TRIMRANGE
lift into arrays of references". With A1 = 1, A2 = 100, B1 = 2, probe one formula per workbook (the combined workbooks of
T-1955 failed to open in Excel for the web), with the excel-probe skill:
- `=AREAS(OFFSET(A1,{0;-1},0))`, `=ROW(OFFSET(A1,{0;-1},0))`, `=SUBTOTAL(9,OFFSET(A1,{0;-1},0))` (error elements);
- `=COUNTIF(OFFSET(A1,{0;1},0),{">0",">5"})`, `=SUMIF(OFFSET(A1,{0;1},0),{">0",">50"})`, `=SUBTOTAL({9,1},OFFSET(A1,{0;1},0))`
  (an array beside a grid: 2x1 of top-left values, or a 2x2 broadcast?);
- `=COUNTIF(OFFSET(A1,{0;1},0),IF(TRUE,C5))` with C5 blank (an empty criteria beside a grid);
- `=SUM(VSTACK(OFFSET(A1,{0;1},0),OFFSET(A1,0,{0,1})))` (several grids);
- `=SUBTOTAL(9,IF(TRUE,OFFSET(A1,{0;1},0)))`, `=ROW(LET(x,OFFSET(A1,{0;1},0),x))`, `=AREAS(TAKE(OFFSET(A1,{0;1;2},0),2))`
  (pass-through to a per-element caller).
Code: `Model::call_per_reference_element`, `Model::vstack_reference` (functions/lookup_and_reference).

## Acceptance criteria
- [ ] A probes/ case holds each formula above with excel-graph values.
- [ ] lookup.md records the results; each mismatch with the engine is filed as a task.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
