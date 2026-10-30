---
id: T-3218
title: "Table auto-expansion on typing beside a table: a typed constant and a table with a totals row grow it too (rules 7a, 7c)"
milestone: M2
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
The engine grows a table by a column, keeps cells and fills the column exactly as Excel does when a constant is
typed just right of a table, or a formula or constant is typed just right of a table with a totals row
(docs/excel-semantics/references.md "Calculated columns" rules 7a and 7c).

## Context
T-3003 probed (probes/t3003-*, excel-graph): a constant beside the table grows it (no fill, header `Column1`; in D1
it becomes the header text); a totals row does not stop growth (D2:D4 filled, totals cell blank; typed in the totals
cell only that cell holds the formula); an occupied neighbour (D1 or D3) stops growth; typing just below the last
data row grows the table by a row; an unqualified `[@Col]*2` is accepted, grows and fills. Code:
crates/rustsheet-engine/src/calculated_columns.rs `Model::table_expansion` and `Model::expand_table`.

Cases owned (decision 0096; every failing cell of each):
- probes/t3003-const-number-d1 S!H1, S!H4, S!K1
- probes/t3003-const-number-d2 S!H1, S!H4, S!J1, S!K1
- probes/t3003-const-text-d1 S!H1
- probes/t3003-const-text-d2 S!H1, S!H4, S!J1, S!K1
- probes/t3003-totals-data-row S!D3, S!D4, S!F3, S!F4, S!H1, S!H4, S!J1, S!J3, S!J4, S!K1
- probes/t3003-totals-totals-row S!H1, S!H4, S!J1, S!K1

That is 26 visibly-off cells at eval 2026-10-30.

Split at planning 2026-10-30, because the task bundled three separate mechanisms:
- Row growth (rule 7e, its two probe cases) moved to T-3273, which depends on this task.
- The unqualified `[@Col]` beside a table (rule 7f, its probe case) moved to T-3274: a parse-context change, not a
  guard change.

What this task changes:
- the guards at the top of `Model::table_expansion` (formula only, `totals_row_count == 0`);
- `Model::expand_table`'s header text: a constant typed into the header cell becomes the header text, and `5` is the
  *text* `5`;
- the facade's `Txn::log_typed_fill` (crates/rustsheet/src/edit/calculated_column.rs), which returns early for
  non-formula text, so a constant's growth reaches the undo log.

## Acceptance criteria
- [ ] Rules 7a and 7c are implemented. Every cell listed above passes bit-exact, and the F lines leave
      eval/baseline/probes.txt via `make eval-baseline`, with 0 regressions.
- [ ] The already-matching occupied-* (7d) and header-formula-* (7b) cases keep passing, as do the t2957-* cases.
- [ ] Facade test: undoing a typed constant that grew the table restores the table and the header cell.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Overlap with another table (unprobed).
- Row growth below the last data row (T-3273) and an unqualified `[@Col]` typed beside the table (T-3274).
- Whether the totals cell D5 is stored as a totals-row formula (unprobed, rule 7c).
