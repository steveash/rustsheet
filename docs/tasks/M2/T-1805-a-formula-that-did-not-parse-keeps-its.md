---
id: T-1805
title: "A formula that did not parse keeps its spacing after a structural edit"
milestone: M2
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
A formula that does not parse, such as `=Ab1 (1)`, keeps its text exactly (spaces included) after a structural edit such as `rename_sheet`.

## Context
Found by T-1758: `=Ab1 (1)` prints `=Ab1 (1)` but `=Ab1(1)` after `rename_sheet` (also on main before T-1758). A parse-error formula is stored as its raw A1 text (`ParseErrorKind`, parser/stringify.rs), and the structural edit re-parses that text in R1C1 mode, where it parses as a named call and is re-printed without the space. A stored text that did not parse in A1 should not be re-parsed and re-printed. Regression test: `unparsed_cell_shaped_call_keeps_its_text_after_a_sheet_rename` (test_defined_name_grouped.rs) pins the current `=Ab1(1)`; update it.

## Acceptance criteria
- [ ] `=Ab1 (1)` (and another unparsable formula with spaces) reads back unchanged after `rename_sheet`, `insert_rows` and `insert_columns`.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
