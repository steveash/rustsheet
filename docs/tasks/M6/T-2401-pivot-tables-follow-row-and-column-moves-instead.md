---
id: T-2401
title: "Pivot tables follow row and column moves instead of being dropped (SPEC-0015)"
milestone: M6
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-2086]
---

## Goal
A row or column move (`move_rows_action`, `move_columns_action`) on a sheet holding pivots moves each pivot's location with its cells when the pivot survives the move whole, instead of dropping every pivot on the sheet.

## Context
T-2086 made inserts and deletes move pivots (`Model::displace_pivots`, crates/rustsheet-engine/src/pivot.rs) but left moves
calling `Model::drop_pivots_on`, a gap recorded in docs/excel-semantics/web.md "GETPIVOTDATA against a pivot table". The move
actions in actions.rs already have the interval arithmetic merged cells use (`interval_survives_move`,
`displace_interval_for_move`): reuse them for the pivot's rows (or columns); a move that would split a pivot drops it, as a
move that splits a merged cell is refused (Excel refuses moves that cut a pivot, SPEC-0015; the engine drops the pivot).

## Acceptance criteria
- [ ] Unit tests: moving lines above/left of a pivot past it shifts it, moving the pivot's whole span moves it, a move that splits it drops it; GETPIVOTDATA keeps answering after a whole move.
- [ ] web.md drops the "row or column move still drops those pivots" gap.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Refusing edits that cut a pivot (Excel's behavior; the engine has no refusal path for pivots).
