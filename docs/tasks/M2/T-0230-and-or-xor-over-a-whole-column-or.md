---
id: T-0230
title: "AND/OR/XOR over a whole-column or whole-sheet range walks every cell: clip to the used area"
milestone: M2
lane: perf
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
AND/OR/XOR over a whole-column or whole-sheet range stay bounded (clip the walk to the used area) with Excel's results unchanged.

## Context
After T-0215 removed the early return in `logical_nary`, `AND(A:XFD)`-style calls always walk every cell; XOR already did. Untrusted input must not trigger unbounded work (CLAUDE.md directive 5). Cells beyond the used area are empty and ignored, so clipping is safe; mind `record_seen_empty_range` dependency tracking as in `fn_countifs`.

## Acceptance criteria
- [ ] Whole-sheet AND/OR/XOR completes quickly; results unchanged; dependency tracking still correct; `make check` green.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Scope note (from the T-0225 review)
The same unbounded walk exists in every `*_or_array` lift (ABS, TRIM, NOT, ...) because `Model::evaluate_range`
(`model.rs`) materializes every cell of the range, so `=NOT(A:XFD)` tries ~17 billion elements. Fix it in
`evaluate_range` (clip to the used area, keep `record_seen_empty_range` tracking) rather than per function.
