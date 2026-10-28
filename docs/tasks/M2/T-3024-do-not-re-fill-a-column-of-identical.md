---
id: T-3024
title: "Do not re-fill a column of identical stored formulas that is not a calculated column (T-2983)"
milestone: M2
lane: engine
status: done
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
Typing a formula into the one blank cell of a table column whose other data cells hold the same stored formula (no `calculatedColumnFormula`) changes only that cell, as Excel does (references.md, calculated columns, rule 8a).

## Context
`Model::calculated_column_targets` (calculated_columns.rs) treats stored identical formulas as a calculated column and re-fills them; probes/t2983-identical-stored-refill fails (C2:C4 become the typed formula). Excel fills only when the column was filled by typing (it saves `calculatedColumnFormula`), so the engine needs some record of that, which is a design question (T-2969 covers import/export of `calculatedColumnFormula`). probes/t2983-formulas-and-blank-enter-blank is a second Excel witness (C2, C4 are one relative formula) and passes only because the eval adapter (crates/rustsheet-eval/src/engine.rs) writes stored and `entered` cells interleaved in row-major order, unlike the oracle, which types after load: fix that ordering too.

## Acceptance criteria
- [x] probes/t2983-identical-stored-refill passes and leaves the probes baseline; rules 1-5 of the t2957 cases still pass; entered cells are typed after all stored cells and t2983-formulas-and-blank-enter-blank still passes.
- [x] `make check` green; reviewers approve.

## Out of scope
- Reading or writing `calculatedColumnFormula` in xlsx (T-3033, after T-2969's probe).

## Outcome
`TableColumn::calculated_column_formula` (Excel's `calculatedColumnFormula`, first data row's text) records a
calculated column. A typed formula that fills or re-fills a column sets it (`fill_calculated_column`). A column whose
other data cells all hold one formula is re-filled only when it is set (`Model::calculated_column_fill`), so identical
stored formulas stay alone (references.md rule 8a). The engine's public `calculated_column_targets` became
`calculated_column`, which returns the table as it was (`CalculatedColumn`). The facade keeps that table for its
rollback (`Undo::TableBefore`, formerly `TableExpanded`). The eval adapter now types `entered` cells in a phase after
every stored cell on every sheet. xlsx import sets the field to `None`, and export does not write it: follow-up T-3033.
Eval delta: probes known failures 2140 -> 2134 (t2983-identical-stored-refill S!C2:C4, S!F2:F4 fixed), 0 regressions.
t2983-formulas-and-blank-enter-blank and -enter-last and the t2957 cases still pass. New tests are in
test_calculated_columns.rs.
