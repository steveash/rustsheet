---
id: T-2064
title: "Per-element callers over an array of references: SUBTOTAL, COUNTIF/SUMIF(S), COUNTBLANK, AREAS, ROW/COLUMN/CELL, ISFORMULA, N/T (SPEC-0014)"
milestone: M6
lane: engine
status: done
priority: P2
size: M
tier: opus
campaign: visible
spec: docs/specs/SPEC-0014-reference-array-results.md
depends_on: [T-1956]
---

## Goal
SUBTOTAL, COUNTIF, SUMIF, SUMIFS, COUNTBLANK, AREAS, ROW, COLUMN, CELL, ISFORMULA, N and T lift over an array of references, one call per whole element, and VSTACK hands the elements on in a reference position, as Excel does.

## Context
Design: docs/specs/SPEC-0014-reference-array-results.md rules 6 and 7, "Consumers" (`Model::lift_over_ref_grid`) and "Bounds" (each element's cells charged before walking). Builds on T-1956's `RefGrid`. Evidence: probes/reference-lift-consumers E57-E78, E97, E109, E113, E165, E169, E185, reference-lift-consumers-b1c E85-E93, -b1d E97, -b2 E125-E133, E177, E193, E197, E225 (excel-graph, 2026-10-19). Today COUNTIF/SUMIF read their range through `evaluate_node_with_reference` (functions/statistical/if_ifs.rs) and SUBTOTAL through `evaluate_picked_reference_arg` (functions/subtotal.rs); both must read `RefArg`.

Visibly-off cells (gap triage 2026-10-21, decision 0083): 23 cells, 0 numeric past half a unit in the 15th digit and 23 with a wrong type, error or text, blamed on OFFSET 22, COUNTBLANK 2, SUMIF 2, ISFORMULA 1, VSTACK 1, INDIRECT 1, SUMIFS 1, AREAS 1.

| case / cell | formula | Excel | ours |
|---|---|---|---|
| probes/reference-lift-consumers-b1c Sheet1!E85 | `=ISFORMULA(OFFSET(A1,{0;1},0))` | FALSE | #VALUE! |
| probes/reference-lift-consumers-b1c Sheet1!E89 | `=ROW(OFFSET(A1,{0;1},0))` | 1 | #VALUE! |
| probes/reference-lift-consumers-b1c Sheet1!E93 | `=COLUMN(OFFSET(A1,0,{0,1}))` | 1 | #VALUE! |
| probes/reference-lift-consumers-b1d Sheet1!E101 | `=SHEET(OFFSET(A1,{0;1},0))` | #N/A | #N/IMPL! |
| probes/reference-lift-consumers-b1d Sheet1!E97 | `=CELL("row",OFFSET(A1,{0;1},0))` | 1 | #VALUE! |
| probes/reference-lift-consumers-b2 Sheet1!E125 | `=COUNTBLANK(OFFSET(A5,0,0,{1;2}))` | 1 | #VALUE! |
| probes/reference-lift-consumers-b2 Sheet1!E129 | `=SUMIF(OFFSET(A1,0,0,{1;2}),">0",OFFSET(B1,0,0,{1;2}))` | 2 | #VALUE! |
| probes/reference-lift-consumers-b2 Sheet1!E133 | `=SUBTOTAL(9,OFFSET(A1,{0;1},0),A1)` | 2 | #VALUE! |
| probes/reference-lift-consumers-b2 Sheet1!E177 | `=SUBTOTAL(109,OFFSET(A1,0,0,{1;2}))` | 1 | #VALUE! |
| probes/reference-lift-consumers-b2 Sheet1!E193 | `=N(OFFSET(A1,0,0,{2;1}))` | 1 | #VALUE! |
| probes/reference-lift-consumers-b2 Sheet1!E197 | `=T(OFFSET(A4,{0;1},0))` | "5" | #VALUE! |
| probes/reference-lift-consumers-b2 Sheet1!E225 | `=SUM(VSTACK(OFFSET(A1,{0;1},0)))` | 101 | #VALUE! |
| probes/reference-lift-consumers Sheet1!E109 | `=COUNTBLANK(OFFSET(A1,{0;1},0))` | 0 | #VALUE! |
| probes/reference-lift-consumers Sheet1!E113 | `=SUMPRODUCT(SUBTOTAL(9,OFFSET(A1,{0;1;2},0)))` | 10101 | #VALUE! |
| probes/reference-lift-consumers Sheet1!E165 | `=COUNTIF(INDIRECT({"A1:A2";"B1:B2"}),">50")` | 1 | #VALUE! |
| probes/reference-lift-consumers Sheet1!E169 | `=SUMIFS(OFFSET(B1,{0;1},0),OFFSET(A1,{0;1},0),">0")` | 2 | #VALUE! |
| probes/reference-lift-consumers Sheet1!E185 | `=SUBTOTAL(9,OFFSET(A1,0,0,{1;2;3}))` | 1 | #VALUE! |
| probes/reference-lift-consumers Sheet1!E57 | `=COUNTIF(OFFSET(A1,{0;1},0),">0")` | 1 | #VALUE! |
| probes/reference-lift-consumers Sheet1!E61 | `=SUMIF(OFFSET(A1,0,0,{1;2}),">0")` | 1 | #VALUE! |
| probes/reference-lift-consumers Sheet1!E65 | `=SUBTOTAL(9,OFFSET(A1,{0;1},0))` | 1 | #VALUE! |
| probes/reference-lift-consumers Sheet1!E73 | `=SUM(SUBTOTAL(9,OFFSET(A1,0,0,{1;2})))` | 102 | #VALUE! |
| probes/reference-lift-consumers Sheet1!E77 | `=AREAS(OFFSET(A1,{0;1},0))` | 1 | #VALUE! |
| probes/reference-lift-consumers Sheet1!E97 | `=N(OFFSET(A1,{0;1},0))` | 1 | #VALUE! |

## Acceptance criteria
- [x] Visible parity: the 23 visibly-off cells listed under Context (2026-10-21) have Excel's value to 15 significant digits, or its type, error and text: the 23 lines in eval/baseline/probes.txt disappear (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] The cells above pass and leave eval/baseline, with no new known failures.
- [x] `SHEET(OFFSET(A1,{0;1},0))` is one `#N/A` (reference-lift-consumers-b1d E101).
- [x] The per-element charge's bound error is recorded as a DEVIATION (decision 0014) under lookup.md.
- [x] A per-element walk past the lift's visit budget is its bound error before any element runs (unit test).
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (`#VALUE!` where Excel lifts a caller over an array of references), so this task is in the campaign. Its target is that every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0), or has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it.

## Outcome
`Function::per_reference_element` lists the per-element callers (SUBTOTAL, COUNTIF, SUMIF, SUMIFS, COUNTBLANK,
AREAS, ROW, COLUMN, CELL, ISFORMULA, N, T) and the arguments that take a reference; `Model::evaluate_function` hands
them to `Model::call_per_reference_element` (lookup_and_reference/ref_lift.rs). It reads a direct OFFSET or INDIRECT
reference argument once and puts back what it gave (`put_back`: an absolute reference, a union, a literal), so a
nest of calls stays linear. Over a grid it reads the other arguments once the same way, checks the shape, charges
every element's walks and the other arguments' reads (DEVIATION in lookup.md), then calls the function per element.
COUNTIF/SUMIF/SUMIFS spill by `criteria_and_grid_static_result`; SHEET of a grid is one `#N/A`; VSTACK of grids in
a reference position stacks them (`Model::vstack_reference`). Eval: probes known failures 1738 -> 1695 (43 cells:
the 23 listed plus their spilled neighbours), no regressions; visible parity 1760 -> 1742. Follow-up: T-2292
probes the cases lookup.md lists as unprobed.

## T-2065 findings
Per-element oracle values (probes/t2065-lift-*, excel-graph): `COUNTIFS(OFFSET(A1,{0;1},0),">0")` 1, 1;
`AVERAGEIF(OFFSET(A1,0,0,{1;2}),">0")` 1, 50.5; `MAXIFS(OFFSET(A1,{0;1},0),OFFSET(A1,{0;1},0),">0")` 1, 100;
`CONCAT(OFFSET(A1,{0;1},0))` `#VALUE!`. See SPEC-0014 and docs/excel-semantics/lookup.md.
