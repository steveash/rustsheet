---
id: T-0127
title: "XMATCH whole-column search: virtual empty tail instead of truncation"
milestone: M1
lane: engine
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
XMATCH over a whole-column/row reference gives the same result as a full walk, whatever the sheet's used area, while still evaluating only the used part.

## Context
`fn_xmatch` (functions/lookup_and_reference/xmatch.rs) truncates `A:A` / `1:1` at the used area via `clip_to_used_area`, which drops the empty tail. Because the engine compares an empty cell as 0/""/FALSE, that changes results: e.g. `XMATCH(0,A:A)` with data in A1:A3 is #N/A truncated but would match row 4 on a full walk, and binary modes (search_mode ±2) probe different positions. T-0119 fixed this for MATCH/LOOKUP/HLOOKUP/VLOOKUP with `SearchVector` (lookup_table.rs, virtual run of empty cells); reuse it here (the `binary_search_*` helpers in functions/binary_search.rs take slices and need an accessor form). Excel's own treatment of the blank tail is T-0126.

## Acceptance criteria
- [ ] XMATCH builds its search vector with `range_search_vector`; `results_independent_of_used_area` unit tests (test/lookup_and_reference/mod.rs) cover every match/search mode over `A:A` and `1:1`.
- [ ] Spill-restart test (`test_xmatch_whole_column_sees_a_spill_beyond_the_used_area`) still passes.
- [ ] `make check` green; reviewers approve.

## Out of scope
Changing values to match Excel's blank-tail behaviour (T-0126).
