---
id: T-3422
title: "Model::range_clear_contents walks only the cells that exist, not every cell of the area"
milestone: M12
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Clearing the contents of a large area (a whole sheet, 1,048,576 x 16,384 cells) takes time proportional to the cells
that exist in it, not to the area, so a crafted external diff or a whole-sheet clear cannot stall the engine.

## Context
Found in T-2896. `Model::range_clear_contents` (crates/rustsheet-engine/src/model.rs) loops
`for row in range.row..range.row + range.height { for column in ... { ws.get_cell_structure(row, column) } }` over every
cell of the area, about 17 billion calls for a whole sheet. Since T-2896 the area has to lie within the grid (no overflow),
but a peer's `RangeClearContents` diff can still name the whole sheet (directive 5: bounded work on untrusted input).
`Model::range_clear_all` and `Model::can_clear_range` already walk only the existing cells (`sheet_data.rows()`,
`columns_in_row`); follow them.

## Acceptance criteria
- [ ] A test clears the contents of the whole sheet (an area of `LAST_ROW` x `LAST_COLUMN`) holding a few cells and a
      dynamic spill, and finishes in well under a second, with the same result as today's walk.
- [ ] The existing clear and undo tests pass unchanged.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
