---
id: T-1805
title: "A formula that did not parse keeps its text after a structural edit"
milestone: M2
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A formula that does not parse, such as `=Ab1 (1)`, keeps its text exactly (spaces and R1C1-shaped references included) after a structural edit such as `rename_sheet`.

## Context
Found by T-1758: `=Ab1 (1)` prints `=Ab1 (1)` but `=Ab1(1)` after `rename_sheet` (also on main before T-1758). A parse-error formula is stored as its raw A1 text (`ParseErrorKind`, parser/stringify.rs), and the structural edit re-parses that text in R1C1 mode, where it parses as a named call and is re-printed without the space. The same re-parse rewrites any R1C1-shaped raw text, on main too: `=R[1]C[1]+1` in A8 (`#ERROR!`) becomes `=B9+1` (value 1) after `rename_sheet`, `=(R[1]C[1])(1)` becomes `=(B9)(1)` and `=SUM(R[1]C[1])` `=SUM(B9)`; with T-1758's stored call form `R[-7]C[27](1)`, `=R[1]C[1](1)` likewise becomes `=B9(1)`. A stored text that did not parse in A1 should not be re-parsed and re-printed. Regression test: `unparsed_cell_shaped_call_keeps_its_text_after_a_sheet_rename` (test_defined_name_grouped.rs) pins the current `=Ab1(1)`; update it.

## Acceptance criteria
- [ ] `=Ab1 (1)` (and another unparsable formula with spaces) reads back unchanged after `rename_sheet`, `insert_rows` and `insert_columns`, as do `=R[1]C[1]+1`, `=R[1]C[1](1)` and `=SUM(R[1]C[1])` (still `#ERROR!`).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
