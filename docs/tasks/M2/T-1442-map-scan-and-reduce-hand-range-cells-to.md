---
id: T-1442
title: "MAP, SCAN and REDUCE hand range cells to the LAMBDA as references; BYCOL(range,COUNTA) is #VALUE!"
milestone: M2
lane: functions-logic
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: [T-1395]
---

## Goal
MAP, SCAN and REDUCE over a range bind each cell to the LAMBDA as a reference, as BYROW and BYCOL bind rows and columns since T-1395, and `BYCOL(A1:B3,COUNTA)` (an eta-reduced COUNTA) is `#VALUE!` as in Excel.

## Context
probes/lambda-helper-range-slices (excel-graph, 2026-10-12): `MAP(A1:A3,LAMBDA(v,ISREF(v)))` is TRUE per cell (E9:E11),
`SCAN(0,A1:A3,LAMBDA(a,v,a+ISREF(v)))` 1,2,3 (E23:E25), `REDUCE(0,A1:A3,LAMBDA(a,v,a+ISREF(v)))` 3 (E27); the engine
passes values (FALSE, 0). E32 `BYCOL(A1:B3,COUNTA)` is `#VALUE!` in Excel and `#NAME?` here. See
docs/excel-semantics/arrays.md "BYROW, BYCOL, MAP, SCAN and REDUCE hand a range's slices to the LAMBDA as references".
Code: functions/logical/map_reduce.rs (`call_lambda_with_values` already binds a `CalcResult::Range` as a reference;
see `by_slices_source` in bycol_byrow.rs for the BYROW/BYCOL version).

## Acceptance criteria
- [ ] probes/lambda-helper-range-slices E9:E11, E23:E25, E27 and E32 pass; `make eval-baseline` records the fix.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Ranges on several sheets (3-D) and unions.
