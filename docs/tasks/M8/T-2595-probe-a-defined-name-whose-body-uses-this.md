---
id: T-2595
title: "Probe a defined name whose body uses #This Row"
milestone: M8
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-1699]
---

## Goal
Oracle evidence for what a defined name whose body is a `#This Row` structured reference
(`=Sales[@Qty]`, `=Sales[@Qty]*2`, `=SUM(Sales[@[Qty]:[Val]])`) gives when a cell uses it, on and off the table's data
rows, on the table's sheet and another; the engine matches it.

## Context
Found by T-1699's review. A name has no cell, so the engine parses its body at row 1
(`Model::parse_defined_names`, new_empty.rs) and resolves its `#This Row` areas there (`Node::lower_this_rows_at`):
with the header on row 1, `=Sales[@Qty]` is always `#VALUE!`, which is what it gave before SPEC-0016 step 2. Hand guess:
Excel resolves it at the calling cell's row, like a relative reference in a name (references.md "Structured references:
this row, totals and ranges", unprobed note). If so, drop the `lower_this_rows_at` call there, and the evaluator's
`Node::ThisRowKind` handling resolves it at the calling cell (it does for cell formulas). Write the probe with the
`excel-probe` skill (the table is set up as gen-m8b's cases do).

## Acceptance criteria
- [ ] A probe case (excel-graph) with the forms above; references.md records the rule.
- [ ] The engine matches it; test_this_row.rs `a_defined_name_resolves_at_row_one` follows the evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
