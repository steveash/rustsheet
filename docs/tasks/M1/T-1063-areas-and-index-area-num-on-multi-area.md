---
id: T-1063
title: "AREAS and INDEX area_num on multi-area references (SPEC-0004 step 3)"
milestone: M1
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
spec: docs/specs/SPEC-0004-multi-area-references.md
depends_on: [T-1062]
---

## Goal
`AREAS` counts the areas of a reference and `INDEX`'s `area_num` selects one, as Excel does.

## Context
SPEC-0004 step 3. `fn_areas` (lookup_and_reference/address_areas.rs) always answers 1 and `index_impl` (lookup_and_reference/mod.rs) treats any `area_num` other than 1 as `#REF!`. Use `evaluate_reference_arg` and `RefArg::areas()` from T-1062. Evidence: SPEC-0004 probe rows 6, 7, 8 (cases from T-1061).

## Acceptance criteria
- [ ] Probe rows 6, 7 and 8 pass (`AREAS(((A1,B2),C3))`, `INDEX((A1:B2,C3:D4),1,1,2)`, `SUM(INDEX(…,0,0,2))`).
- [ ] Unit tests: `area_num` 0, past the last area, and a huge value do not panic.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other functions.
