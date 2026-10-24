---
id: T-2342
title: "Convert a table to a range: its structured references become absolute plain ranges (T-2324)"
milestone: M8
lane: engine
status: todo
priority: P3
size: M
tier: sonnet
depends_on: []
---

## Goal
Converting a table to a range keeps its cells and rewrites every structured reference to it as the
absolute plain range Excel writes.

## Context
T-2324 (excel-graph, Graph `tables('Sales')/convertToRange`), probes/table-edits/table-convert-to-range in
docs/excel-semantics/references.md "Structured references under row deletes, column edits, convert, table and sheet delete": values are unchanged and FORMULATEXT prints `=SUM(S!$C$4:$C$7)`, `=COUNTA(S!$B$4:$B$7)`,
`=COLUMN(S!$C$4:$C$7)` (absolute, sheet-qualified from another sheet; a plain `S!C4:C7` is unchanged).
`ROWS(Sales)` 4, `ROWS(Sales[#All])` 5 and `ROW(Sales[#Headers])` 3 keep their values, so they became the
data, all and header ranges. The engine has no convert-to-range: the adapter refuses `convert_table`
(crates/rustsheet-eval/src/case.rs) and the case is an adapter error. Add an engine/facade operation and let
the adapter call it. Formulas on the table's own sheet and `#This Row` (relative form?) are unprobed: probe
them in the same PR (excel-probe skill).

## Acceptance criteria
- [ ] table-convert-to-range scores and every cell passes; the adapter accepts `convert_table`.
- [ ] references.md updated with the same-sheet probe.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Cut and paste, row and column moves, undo and duplicated sheets (desktop, T-2325).
