---
id: T-3210
title: "Table auto-expansion on typing beside or below a table: constants, totals row, occupied neighbour, row growth, unqualified [@Col]"
milestone: M2
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The engine grows a table, keeps cells and fills columns exactly as Excel does when a value or formula is typed beside
or just below a table (docs/excel-semantics/references.md "Calculated columns" rule 7a-7f).

## Context
T-3003 probed (probes/t3003-*, excel-graph): a constant beside the table grows it (no fill, header `Column1`; in D1
it becomes the header text); a totals row does not stop growth (D2:D4 filled, totals cell blank; typed in the totals
cell only that cell holds the formula); an occupied neighbour (D1 or D3) stops growth; typing just below the last
data row grows the table by a row; an unqualified `[@Col]*2` is accepted, grows and fills. Code:
crates/rustsheet-engine/src/calculated_columns.rs `Model::table_expansion`, plus the row-growth path.

## Acceptance criteria
- [ ] Each of rules 7a, 7c, 7e, 7f and the occupied-neighbour rule 7d is implemented and the t3003-* probe cases pass
      (eval baseline shrinks).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Overlap with another table (unprobed).
