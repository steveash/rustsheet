---
id: T-1699
title: "Resolve #This Row off-table #VALUE! at evaluation, not parse time"
milestone: M8
lane: engine
status: done
priority: P1
size: S
tier: opus
campaign: visible
spec: docs/specs/SPEC-0016-structured-references-in-the-parsed-formula.md
depends_on: [T-2195]
---

## Goal
`#This Row` off the table's data rows is decided when the table is resolved at evaluation, so a formula stays right after the table grows, shrinks or has rows inserted, and serialises back to the structured reference.

## Context
T-1675 returns `Node::ErrorKind(VALUE)` at parse time in `structured_reference_operand` (operands.rs), as `#Totals` without a totals row already returns `#REF!`; the table is not re-resolved on later edits. Excel evidence: gen-m8b this-row e1bf5aa0ffb4ed87; for SUM of `#This Row`, in-table 19fdc25163755713 D11; ISREF of `#This Row` is unprobed (T-2197).

Re-scoped as SPEC-0016 rollout step 2 (T-1699's first attempt, 2026-10-20). Every structured reference is lowered and stored as a plain range, so the text is gone before evaluation; T-2195 keeps it first. Shared formulas parse at context (1, 1), so `#This Row` lowers to `ThisRowKind` (data-row bounds), which is evaluated against the cell's row. The engine has no table-resize API, so "grow the table" in the test means `remove_table` + `add_table` over a larger range.

## Acceptance criteria
- [x] `Node::ThisRowKind` and `Model::this_row_area` per SPEC-0016; `#This Row` emits `StructuredReferenceKind`.
- [x] Every `ReferenceKind`/`RangeKind` match site, static analysis included (shape and `StaticResult`), is re-audited for `ThisRowKind`; tests for `ISREF(Sales[@Qty])` (hand expectation TRUE on a data row) and `SUM(Sales[@[A]:[B]])` (in-table D11) match the `$A$3` forms.
- [x] A `ThisRowKind` printed as a lowered area (structural-edit fallback, `to_excel_string`) is first resolved at the printing cell's row (SPEC-0016 "Structural edits"); tests: export of `Sales[@Qty]` on and off the data rows gives today's `$A$3` / `#VALUE!`.
- [x] The bounds are computed with checked arithmetic (overflow is `#REF!`); evaluation is a plain comparison with no unchecked arithmetic.
- [x] Unit test: grow the table so a former off-table row becomes a data row and the formula reads it, and `get_cell_formula` keeps the structured reference (export stays lowered until T-2196).
- [x] `make check` green; reviewers approve (review pending).

## Out of scope
-

## Visible-parity stage
Decision 0083: wrong values here are visible to anyone using defined names or tables (gen-m8b `[#This Row]` off-table cells), so this task is in the campaign. Its acceptance criteria are already the visible-parity stage.

## Outcome
`#This Row` now emits `Node::StructuredReferenceKind` around a new leaf `Node::ThisRowKind` (columns, data-row bounds from
checked parse-time arithmetic, and a `cell` flag for one column named alone), so its text is stored
(`Sales[[#This Row],[Qty]]`), one shared formula serves a column, and the row is decided at evaluation. `Node::this_row_at(row)`
gives the node the old parse-time lowering gave on that row (cell, range or `#VALUE!`); the reference-kind sites call
`Node::ungrouped_at`/`resolved_at` (ISREF, SHEETS, the aggregate and reference readers, CONCAT, PERCENTILE/MODE/CHISQ/paired,
LINEST/TREND, NPV, IMSUM, VSTACK, lifted options, units), evaluation resolves it in `evaluate_node_in_context` and
`evaluate_node_with_reference`, static analysis reads it as on a data row, and the printers resolve it at the printing cell's row
(export, structural-edit fallback). A structural edit or a cut and paste that moves the formula to another row lowers its
`#This Row` areas at the old row (`Model::cell_formula_text`), so it reads what it read. Tests: test/test_this_row.rs (parity of
30 formulas against the old lowering on and off the data rows, on the table's sheet and another, through 13 structural edits and
cut and paste; ISREF and SUM; growing and shrinking the table; export), parser/tests/test_tables.rs (preserved text, round trip).
SPEC-0016 gets an "as built" note. Eval delta: none (`make eval-baseline`: 0 fixed, 0 regressions).
Follow-up: T-2589 (sheet duplicate/delete lower shared `#This Row` formulas without a row, so off-table cells there differ from
the old values).
