---
id: T-3667
title: "Probe whether SORT, SORTBY, UNIQUE and the other array functions read a one-cell error reference as a 1x1 array"
milestone: M7
lane: functions-lookup
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
We know, from Excel evidence, whether SORT, SORTBY, UNIQUE, TAKE, DROP, CHOOSEROWS, CHOOSECOLS, EXPAND, WRAPROWS,
WRAPCOLS, VSTACK and HSTACK read a one-cell reference to an error cell in an array argument as a 1x1 array holding the
error (the other arguments checked first), as TOCOL, TOROW and FILTER do; each function that does reuses
`one_cell_error_ref_as_array`.

## Context
- T-3506 found the rule for TOCOL, TOROW and FILTER (gen-m7 arg-pairs, excel-graph): with Data!G9 `#DIV/0!`,
  `TOROW(Data!G9,7)` is `#VALUE!` and `FILTER(Data!G9,Data!B1:B12>-1)` `#VALUE!`, while a literal error still wins
  (`TOROW(#N/A,-1)` is `#N/A`). docs/excel-semantics/arrays.md "A one-cell error reference is a 1x1 array in TOCOL, TOROW
  and FILTER".
- The helper is `one_cell_error_ref_as_array` (and `wrap_one_cell_error_ref`) in
  crates/rustsheet-engine/src/functions/spill_functions.rs. The other array functions still return the referenced
  error before checking their other arguments; whether Excel does is unprobed.
- Probe with the `excel-probe` skill: for each function, a one-cell error reference as the array with an invalid later
  argument (e.g. `SORT(G9,5)`, `UNIQUE(G9,"x")`, `TAKE(G9,0)`, `CHOOSEROWS(G9,2)`), against the same with a literal error.

## Acceptance criteria
- [ ] A probe case with excel-graph values answers the question for each listed function; the result is recorded in
  arrays.md (extending the T-3506 entry or a new one).
- [ ] Each function that follows the rule uses the shared helper, with unit tests; the probe cells pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
- TOCOL, TOROW and FILTER themselves (T-3506), and SORT's argument-check order (T-3507).

## Outcome
Probe `probes/one-cell-error-ref-array-functions` (78 cells, excel-graph): TAKE, DROP, CHOOSEROWS, CHOOSECOLS, VSTACK and
HSTACK read a one-cell error reference as a 1x1 array; SORT, SORTBY, UNIQUE, EXPAND, WRAPROWS and WRAPCOLS return its error.
TAKE/DROP already matched; CHOOSEROWS/CHOOSECOLS/VSTACK/HSTACK now use `one_cell_error_ref_as_array`. Recorded in arrays.md.
Probe case 60/78 -> 78/78 cells.
