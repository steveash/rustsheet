---
id: T-0452
title: "Data tables follow structural edits and refuse writes into their result cells"
milestone: M11
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-0441]
---

## Goal
A what-if data table keeps describing the same cells after rows, columns or sheets around it
change, as in Excel. A write into one of its result cells is refused, as Excel refuses one ("Cannot
change part of a data table").

## Context
T-0441 added data tables as sheet metadata: `Worksheet::data_tables`, a `DataTable` range plus a
row and a column input cell (crates/rustsheet-engine/src/types.rs, data_tables.rs). Nothing yet
updates that metadata:
- Inserting or deleting rows or columns (crates/rustsheet-engine/src/actions.rs) moves cells and
  rewrites formulas, but not a table's range or input cells. A table then computes the wrong cells.
- Deleting a row or column that holds an input cell should make that input `Deleted` (`#REF!`, as
  `del1`/`del2` say in the file).
- `set_user_input` and the facade's `Txn::set` overwrite a result cell, and the next evaluation
  writes the table's value back over it.
- Copying a sheet clones the metadata. Check what Excel does when the copy is made.
Probe Excel first (excel-probe skill) for:
- an insert above, inside and left of a table;
- deleting an input cell's row;
- writing into a result cell.
Record the answers in docs/excel-semantics/recalc.md.

## Acceptance criteria
- [ ] Probe cases cover the edits above and are oracle-filled; recalc.md records the rules.
- [ ] Row and column inserts and deletes move or shrink a table's range and its inputs, or mark the
      inputs `Deleted`, as Excel does. Unit tests cover each.
- [ ] Writing into a result cell is an error through the engine and the facade, and an undo of
      `add_data_table` still works.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Recompute cost (T-0453).
