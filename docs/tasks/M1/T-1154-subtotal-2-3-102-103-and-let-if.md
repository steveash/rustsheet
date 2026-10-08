---
id: T-1154
title: "SUBTOTAL 2/3/102/103 and LET/IF-routed unions: probe and accept a union (follow-up of T-1071)"
milestone: M1
lane: functions-math
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
SUBTOTAL function numbers 2, 3, 102 and 103 (COUNT, COUNTA) accept a union as SUM-style aggregates do, and any difference for unions routed through LET, IF or CHOOSE is known, with Excel evidence.

## Context
T-1071 made SUBTOTAL's shared value walk (`subtotal_get_values`) take a union via `subtotal_walk_range`. `subtotal_count` and `subtotal_counta` (functions/subtotal.rs) have their own walks with the same hidden-row loop and still give `#VALUE!` for a union; COUNT and COUNTA accept unions (references.md "Multi-area references in functions"). Unprobed: those function numbers, hidden or filtered rows and a nested SUBTOTAL cell inside a union area, a cross-sheet union, an overlap (`(A1:B2,B2)`), and a union through `LET(x,(A1,B2),SUBTOTAL(9,x))`.

## Acceptance criteria
- [x] A probe case (oracle excel-graph) covers the unprobed shapes above and the finding is recorded in docs/excel-semantics/references.md.
- [x] SUBTOTAL 2/3/102/103 of a union match the oracle, reusing `subtotal_walk_range`; eval baseline shrinks accordingly.
- [x] `make check` green; reviewers approve.

- [x] `subtotal_walk_range` converges with `for_each_position_in_areas` (functions/util.rs, the shared position walk AGGREGATE uses for hidden rows and nested SUBTOTAL cells) so there is one range walk.

## Out of scope
- Other functions' union support (their own SPEC-0004 tasks).

## Outcome
Probes `multiarea-subtotal-counts-a..e` (excel-graph) cover COUNT/COUNTA, overlaps, a nested SUBTOTAL cell, error cells, a cross-sheet union and LET/IF/CHOOSE-routed unions; recorded in references.md. `subtotal_walk_areas` (built on `for_each_position_in_areas`) is now the one walk behind every SUBTOTAL function number; COUNT/COUNTA take unions. Eval: a, b, e pass (new cells); c/d (LET/IF/CHOOSE-routed unions, SUM too) are recorded known failures, owned by the engine task T-1064. Hidden rows in a union are unprobed (case format cannot hide rows). The row-hidden status is now read at each cell's own column (it was the area's first column); only differs for a table covering part of an area.
