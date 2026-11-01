---
id: T-3439
title: "Move the reference-operator area types and error helpers out of reference_operators.rs into a submodule (the file is at 999 of its 1,000 lines)"
milestone: M2
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`crates/rustsheet-engine/src/reference_operators.rs` (the evaluator of `:`, the space intersection and the union) has
room under the 1,000-line file limit again (`scripts/check_file_size.py`). Its area and reference-argument types and
its free error helpers move, unchanged, into a submodule. Then the engine fix that follows T-3308's probe (which
built-in functions Excel accepts as an intersection operand, now an inferred refuse-by-default list in this file)
and T-3307's insert and delete probe do not first have to split the file.

## Context
Planner run 2026-10-31 (`make eval` at origin/main 4c8a1a20). The file is at 999 lines (`wc -l`), one under the limit.
It is not in quality/oversized-files.json, so the gate fails any PR that adds a line to it. T-3377 and T-3313 set
the pattern for the same problem in other files. Do not confuse it with
`crates/rustsheet-engine/src/expressions/parser/reference_operators.rs`, the parser side, which stays as it is.

What moves (line numbers at 4c8a1a20), into `reference_operators/areas.rs`. The file becomes
`reference_operators/mod.rs`; plain `git mv` keeps its history:
- `MAX_AREAS`, `TOO_MANY_AREAS` and their doc comment (the directive-5 DEVIATION note, decision 0014; l. 20-29);
- `AreaList`, `Areas`, `Ranges`, `RefArg`, `Picked`, `RefElement`, `RefGrid` with their impls, and
  `not_a_reference_array` (l. 34-321);
- the free error helpers `three_d_value_error`, `three_d_needs_one_sheet`, `bounding_area`, `several_areas`,
  `several_areas_ref`, `too_many_areas`, `intersect_pair`, `empty_intersection` and `names_other_workbook`
  (l. 322-416).
`impl Model` (l. 417-986) and `single_sheet_area` stay in mod.rs. mod.rs re-exports every moved `pub(crate)` item
(`pub(crate) use areas::{...};`), so the about 40 `use crate::reference_operators::...` sites across the engine still
resolve with no edit. A private item that both files need becomes `pub(super)`, never a copy (directive 3).

## Acceptance criteria
- [ ] The items above live in `reference_operators/areas.rs` with bodies and comments unchanged (a reviewer can
      diff them line by line); `reference_operators/mod.rs` declares and re-exports them.
- [ ] `reference_operators/mod.rs` is at most 650 lines.
- [ ] No file outside `crates/rustsheet-engine/src/reference_operators/` changes.
- [ ] `make eval-check` shows no cell changed (0 fixed, 0 regressions).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any change to what the range, intersection or union operators compute, or to the union area cap.
- Splitting `impl Model` across files.
