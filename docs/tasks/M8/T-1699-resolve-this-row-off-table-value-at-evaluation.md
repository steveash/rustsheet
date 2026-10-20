---
id: T-1699
title: "Resolve #This Row off-table #VALUE! at evaluation, not parse time"
milestone: M8
lane: engine
status: todo
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
T-1675 returns `Node::ErrorKind(VALUE)` at parse time in `structured_reference_operand` (operands.rs), as `#Totals` without a totals row already returns `#REF!`; the table is not re-resolved on later edits. Excel evidence: gen-m8b this-row e1bf5aa0ffb4ed87.

Re-scoped as SPEC-0016 rollout step 2 (T-1699's first attempt, 2026-10-20). Every structured reference is lowered and stored as a plain range, so the text is gone before evaluation; T-2195 keeps it first. Shared formulas parse at context (1, 1), so `#This Row` lowers to `ThisRowKind` (data-row bounds), which is evaluated against the cell's row. The engine has no table-resize API, so "grow the table" in the test means `remove_table` + `add_table` over a larger range.

## Acceptance criteria
- [ ] `Node::ThisRowKind` and `Model::this_row_area` per SPEC-0016; `#This Row` emits `StructuredReferenceKind`.
- [ ] Unit test: grow the table so a former off-table row becomes a data row and the formula reads it; export keeps the structured reference.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Visible-parity stage
Decision 0083: wrong values here are visible to anyone using defined names or tables (gen-m8b `[#This Row]` off-table cells), so this task is in the campaign. Its acceptance criteria are already the visible-parity stage.
