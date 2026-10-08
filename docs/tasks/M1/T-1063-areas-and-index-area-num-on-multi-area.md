---
id: T-1063
title: "AREAS and INDEX area_num on multi-area references (SPEC-0004 step 3)"
milestone: M1
lane: engine
status: done
priority: P2
size: S
tier: sonnet
spec: docs/specs/SPEC-0004-multi-area-references.md
depends_on: [T-1062]
---

## Goal
`AREAS` counts the areas of a reference and `INDEX`'s `area_num` selects one, as Excel does.

## Context
SPEC-0004 step 3. `fn_areas` (lookup_and_reference/address_areas.rs) always answers 1 and `index_impl` (lookup_and_reference/mod.rs) treats any `area_num` other than 1 as `#REF!`. Use `evaluate_reference_arg` and `RefArg::single_sheet_areas()` from T-1062; a 3-D reference keeps its VERIFIED `#VALUE!`. Read `area_num` as a float and range-check it before any integer conversion (below 1 `#VALUE!`, NaN or past the last area `#REF!`). Evidence: SPEC-0004 probe rows 6, 7, 8 and 20 (cases from T-1061).

## Acceptance criteria
- [ ] Probe rows 6, 7, 8 and 20 pass; 3-D cells of probes/3d-ref-functions stay green (`AREAS(((A1,B2),C3))`, `INDEX((A1:B2,C3:D4),1,1,2)`, `SUM(INDEX(…,0,0,2))`).
- [ ] Unit tests: `area_num` 0, past the last area, and a huge value do not panic.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other functions.

## Outcome
`AREAS` counts the single-sheet areas of `evaluate_reference_arg` (3-D stays `#VALUE!`; a non-reference
argument stays 1); `INDEX` selects `area_num` from the areas, range-checked as a float (below 1
`#VALUE!`, NaN or past the last area `#REF!`). Added `RefArg::single_sheet_areas`. Unit tests in
`test_multi_area_arguments.rs` cover probe rows 6, 7, 8, 20 and a huge `area_num`.
Eval delta: probes known failures 210 -> 204 (fixed 6), no regressions.
