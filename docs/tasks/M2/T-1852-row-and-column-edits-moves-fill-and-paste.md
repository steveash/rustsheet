---
id: T-1852
title: "Row and column edits, moves, fill and paste keep a legacy N/T marker"
milestone: M2
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-1851]
spec: docs/specs/SPEC-0012-legacy-n-t-marker.md
---

## Goal
A cell whose formula holds a legacy N/T marker keeps it when the engine rewrites its formula
through display text. That happens on row and column inserts and deletes, cell moves, fill and
paste. So a legacy `=N(IF(1,{2;3}))+A5` is still `#VALUE!` after a row is inserted above it.

## Context
SPEC-0012, "Entry points" (last rows) and Excel rule 6, an unprobed assumption (T-1853). These
paths print the formula as localized A1 text and re-enter it through `update_cell_with_formula` /
`set_user_input`, which parse it as a dynamic formula:
- `displace_cells` (actions.rs);
- `move_cell` (actions.rs);
- `Model::move_cell_value_to_area`;
- `Model::extend_to` / `extend_copied_value`;
- UserModel clipboard.rs and autofill.rs.

When the source node holds a marker (`Node::has_legacy_marker`), the text goes through the legacy
entry (`Model::set_cell_with_legacy_formula`, T-1851). The rewrite is idempotent over its own
display text, which T-1713's property test pins. None of these paths is reachable from the
`rustsheet` facade today.

## Acceptance criteria
- [ ] Unit tests: after each of row insert, column delete, move, fill and paste, a marked cell
  whose text the edit rewrites keeps its marker and its `#VALUE!`. An unmarked dynamic twin
  stays dynamic and spills.
- [ ] references.md "Legacy N and T over a reference-returning call" records the assumption as
  unprobed, pointing at T-1853, unless T-1853 has landed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Probing Excel (T-1853).
