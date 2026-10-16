---
id: T-1691
title: "Moving a 3-D span's end sheet past its other end shrinks the span to the remaining sheets, as Excel rewrites it"
milestone: M8
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
When a sheet move puts one end sheet of a 3-D span on the other side of the other end, the engine
rewrites the span the way Excel does. The formula text and the values match excel-graph.

## Context
Found by the T-1608 sheet-op probes:
- `probes/sheet-ops/sheet-ops-move-span-end-first`: with sheets Calc, Jan, Feb, Mar, moving Mar to
  position 1 rewrites `=SUM(Jan:Mar!A1)` to `=SUM(Jan:Feb!A1)` (30).
- `probes/sheet-ops/sheet-ops-move-span-start-last`: moving Jan to position 3 rewrites it to
  `=SUM(Feb:Mar!A1)` (50).
The span keeps the sheets that are still between the ends, and the moved end is replaced by the
nearest one. The engine keeps `Jan:Mar` and re-resolves it by name (40 in both cases); see
`Model::move_sheet` and `reset_parsed_structures` in `new_empty.rs`.
docs/excel-semantics/references.md, "3-D references when sheets move or are renamed".
Moves that keep both ends in order already match (move-into-span, move-out-of-span).

## Acceptance criteria
- [ ] Both probe cases pass, including the FORMULATEXT cells.
- [ ] A unit test covers a move of the start sheet, of the end sheet, and of a middle sheet past
      an end.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Deleting a span's end sheet (T-0198).
