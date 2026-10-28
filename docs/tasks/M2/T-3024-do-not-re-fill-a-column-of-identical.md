---
id: T-3024
title: "Do not re-fill a column of identical stored formulas that is not a calculated column (T-2983)"
milestone: M2
lane: engine
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
Typing a formula into the one blank cell of a table column whose other data cells hold the same stored formula (no `calculatedColumnFormula`) changes only that cell, as Excel does (references.md, calculated columns, rule 8a).

## Context
`Model::calculated_column_targets` (calculated_columns.rs) treats stored identical formulas as a calculated column and re-fills them; probes/t2983-identical-stored-refill fails (C2:C4 become the typed formula). Excel fills only when the column was filled by typing (it saves `calculatedColumnFormula`), so the engine needs some record of that, which is a design question (T-2969 covers import/export of `calculatedColumnFormula`).

## Acceptance criteria
- [ ] probes/t2983-identical-stored-refill passes and leaves the probes baseline; rules 1-5 of the t2957 cases still pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
