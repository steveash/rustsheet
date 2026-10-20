---
id: T-2174
title: "TAKE and DROP over an array of references keep its elements (SPEC-0014 rule 7)"
milestone: M7
lane: functions-lookup
status: done
priority: P2
size: M
tier: opus
depends_on: [T-1838]
spec: docs/specs/SPEC-0014-reference-array-results.md
---

## Goal
A TAKE or DROP whose array is an array of references (a lifted OFFSET, INDIRECT, TRIMRANGE, TAKE or DROP) keeps
the elements it counts, as an array of references, so SUM and the other reference consumers read them as Excel does.

## Context
SPEC-0014 rule 7 ("Pass-through"): `SUM(TAKE(OFFSET(A1,{0;1;2},0),1))` is 1 and `SUM(DROP(OFFSET(A1,{0;1;2},0),1))`
10100 (probes/reference-lift-consumers E101, -b2 E153, excel-graph, 2026-10-19). Today the array argument is read
through its value view, so DROP's case is `#VALUE!` (b2 E153 is in eval/baseline/probes.txt). T-1838 made TAKE and
DROP producers (`take_drop_reference` in functions/lookup_and_reference/drop_take.rs); this adds the grid source:
read args[0] with `evaluate_reference_arg` where `may_hand_on_grid` says one can come (as `offset_source` does), and
pick the counted elements of the `RefGrid` (a sub-grid, the elements shared or copied within `check_array_size`).
Unlifted counts over a grid only; a lifted count over a grid (`TAKE(OFFSET(..),{1;2})`) is unprobed: file a probe.

## Acceptance criteria
- [x] probes/reference-lift-consumers-b2 E153 passes and leaves eval/baseline/probes.txt; E101 keeps passing.
- [x] Unit tests: TAKE and DROP of a lifted OFFSET grid, by rows and by columns, read by SUM, INDEX and ROWS; an
  error element is kept; `TAKE(grid,0)` is `#CALC!`.
- [x] `make check` green; reviewers approve.

## Out of scope
- Lifted counts over a grid (probe first).
- Per-element consumers (SUBTOTAL, COUNTIF, ...: T-2064).

## Outcome
- `take_drop_reference` (functions/lookup_and_reference/drop_take.rs) reads its array with
  `evaluate_value_or_grid`. In reference mode (SUM's read), it does so only for an OFFSET or INDIRECT source.
- A `RefGrid` with unlifted counts keeps the counted elements as a sub-grid, in `take_drop_of_grid`.
- A lifted count over a grid keeps the value view; T-2193 is the probe for it.
- Unit test: `take_drop_keep_an_array_of_references`. It covers rows, columns, SUM, ROWS, COLUMNS and INDEX, an
  error element, and `TAKE(grid,0)` = `#CALC!`.
- Eval: probes/reference-lift-consumers E101 and -b2 E153 left eval/baseline/probes.txt, with 0 regressions.
  E101 was in the baseline too, contrary to the task text.
