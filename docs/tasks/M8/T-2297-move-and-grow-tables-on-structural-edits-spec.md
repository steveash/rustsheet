---
id: T-2297
title: "Move and grow tables on structural edits (SPEC-0016 step 1b)"
milestone: M8
lane: engine
status: done
priority: P2
size: M
tier: opus
spec: docs/specs/SPEC-0016-structured-references-in-the-parsed-formula.md
depends_on: [T-2195]
---

## Goal
A row or column edit wholly before a table moves the table, and a row insert or delete strictly inside its
data rows grows or shrinks it, so the structured references to it keep their text through the edit.

## Context
Split from T-2195 (its split hint, part b). T-2195 keeps every structured reference as written but treats
every table as SPEC-0016's *fallback*: an edit that changes the area a reference resolves to rewrites it as
the displaced plain range (`StructuredReferenceKind` arm of `stringify`, parser/stringify.rs), and tables
never move. This task adds the *move* and *grow/shrink* classes of SPEC-0016 "Structural edits": classify
the tables on the edited sheet before the walk by the spec's edge-case table, pass the fallback table names
to `to_string_displaced` (a reference to a moved or grown table prints its text), update the tables'
`reference` after the walk, then `Model::reset_tables`. The interim rule is a hand assumption
(docs/excel-semantics/references.md "Structured references when the table changes"); T-2197 probed row inserts: the move, grow-inside and none
rows hold, but an insert at `r == d1` or `r == t` grows the table in Excel (T-2321 does those rows; keep
them fallback here).
Tests: `src/test/test_structured_references.rs` (`check_edit` pins today's values with plain-range twins).

Probe cells this task owns (gap triage 2026-10-25): FORMULATEXT after an edit wholly before the table or a row edit strictly
inside its data rows keeps the structured text in Excel, ours prints the lowered range:
- probes/table-edits/table-delete-column-before F!A5 `=SUM(Sales[Qty])` (ours `=SUM(S!$B$4:$B$7)`), F!A9
  `=COUNTA(Sales[Region])`, F!A12 `=COLUMN(Sales[Qty])`
- probes/table-edits/table-delete-row-inside-data F!A5, F!A9, F!A12 (ours `=SUM(S!$C$4:$C$6)` ...)
- probes/table-edits/table-insert-column-left-edge F!A5, F!A9, F!A12 (ours `=SUM(S!$D$4:$D$7)` ...)
The table-insert-row-* cases are T-2321's, and table-insert-column-inside is T-2339's.

## Acceptance criteria
- [x] Each row of SPEC-0016's edge-case table and each column case has a test: move and grow/shrink keep
  the structured text, fallback prints today's lowered text; values equal today's in every case.
- [x] One test has the formula on another sheet; defined names and CF formulas follow the same rule.
- [x] `make eval` shows no regression; `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`table_edits.rs` classifies the tables of the edited sheet before the formula walk of a row or column
insert or delete (`Model::displace_formulas`, now the one call the six structural edits make): a table
moves with an edit wholly before it, grows with rows inserted strictly inside its data, shrinks with a
delete strictly inside its data (not at its first or last data row: the undo, an insert there, would not
grow it back; T-2338), and is left alone by an edit after it. The walk (`to_string_displaced_keeping`, cells
on every sheet and conditional formats) keeps the text of every structured reference to those tables;
their `reference` is updated after the walk and `reset_tables` reparses, so defined names follow too. Every
other case stays the fallback; a `#This Row` reference keeps the fallback rule, and a headerless table's first row is a fallback
(T-2845 probes both; on another sheet an off-data `#This Row` cell now reads the moved table's row).
Eval: probes known failures 3018 -> 2995 (23 fixed, 0 regressions): every FORMULATEXT cell this task owned
plus the insert-row move/grow cases (above-header, before, inside-data, above-last-data).
