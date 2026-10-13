---
id: T-1497
title: "Bound multi-area walks by the data they visit, not by repetition: union walks inside MAP/BYROW and many whole-column union arguments"
milestone: M1
lane: engine
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Formulas that Excel computes over unions of whole columns stop being `#VALUE!` in the engine, and
evaluation stays bounded (directive 5): the multi-area cell budget grows with the data a walk
visits, not with how often the formula repeats the walk or how many union arguments it has.

## Context
T-1462 probe probes/union-walk-repeated-in-lambda (excel-graph, 2026-10-13): Excel computes all of
these, the engine gives `#VALUE!` from the per-formula `MAX_AREA_CELLS` budget (2^24 cells,
area_budget.rs, T-1162), which charges each multi-area walk the cells of its areas clipped to the
used area:
- Calc (10,000 used rows): `SUM(BYROW(A1:A1000,LAMBDA(r,SUM((B:B,C:C)))))` = 8000, MAP and
  SUBTOTAL(9,..) and COUNTA forms, `MAP(A1:A10000,...)` = 80000, and the bounded union
  `(B1:B10000,C1:C10000)` too.
- Big (data in row 1,048,576): `SUM(BYROW(A1:A10,LAMBDA(r,SUM((B:B,C:C)))))` = 30, and with no
  LAMBDA repetition `=SUM((B:B,C:C),...)` with nine union arguments = 27 (9 x 2^21 cells).
The single-range forms (`SUM(B:C)`, `SUM(B:B)`) pass. `for_each_position_in_areas`
(functions/util.rs) walks every row of the used area densely, so charging what it visits is
right for that walk; the fix is to make the walk cheaper or charge it differently. Options to
weigh (a spec in docs/specs/ first if the change is architectural):
1. Walk multi-area arguments over populated cells only (the sparse storage), as a single range can,
   and charge the budget by populated cells visited; whole-column unions over sparse data then cost
   what the data costs.
2. Charge each distinct union (same areas) once per formula run, caching its cell count, so a
   LAMBDA repeating one union is charged once (does not fix the nine-argument case).
Keep the existing caps on area counts and pairs (`MAX_AREAS`, `MAX_AREA_PAIRS`).
- [x] The 9 failing cells of probes/union-walk-repeated-in-lambda pass, with no new failures.
- [x] A unit test still shows a bounded result (`#VALUE!` or bounded time) for an adversarial
  formula that repeats a dense multi-area walk (directive 5), with the new bound documented in
  references.md "Union area cap".
- [x] `make check` green; reviewers approve.

## Out of scope
- The `MAX_AREAS` and `MAX_AREA_PAIRS` caps.

## Outcome
Option 1. The areas of a union are walked over their existing cells only (`SheetData::next_row_between`
and `columns_in_row_between`; `Model::for_each_position_in_area`), in the same row-major order, and
the walk pays as it goes from the per-formula budget, renamed `MAX_AREA_WALK` (2^24 steps): a step
per cell handed over, per row found (at least one), per unallocated chunk passed over, and per 16
row slots tested (`SLOTS_PER_WALK_STEP`, measured: ~15 ns per cell step, ~1.4 ns per slot). The
up-front dense count (`check_area_cells`) is gone; SUBTOTAL and AGGREGATE (both forms; the array
form had no budget before) walk unions the same way. Single ranges are walked as before. The
seen-empty record of an area is one rectangle. Worst adversarial shapes stop at `#VALUE!` in
<= 0.31 s release (main's old worst: 1.09 s). Eval: probes known failures 5092 -> 5083 (the 9 cells
of probes/union-walk-repeated-in-lambda), no regressions. references.md "Union area cap" updated.
