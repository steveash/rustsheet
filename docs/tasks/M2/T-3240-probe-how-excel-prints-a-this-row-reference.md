---
id: T-3240
title: "Probe how Excel prints a #This Row reference outside its table and in its header or totals row ([@Col] vs T[@Col] vs long form)"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence (excel-graph) for how FORMULATEXT prints a `#This Row` structured reference where T-3069 had none:
in a cell outside its table (same row as a data row, and another row), and in the table's header row and totals row.

## Context
T-3069 prints `T[[#This Row],[Col]]` as `[@Col]` in any cell of its own table, using `Parser::containing_table_name`
(crates/rustsheet-engine/src/expressions/parser/structured_reference.rs), which counts the header and totals rows too,
and keeps the long form outside the table. The only evidence is probes/t2800-this-row-whitespace (data-row cells of
a one-row table; docs/excel-semantics/syntax.md "Whitespace beside a missing argument"). Excel likely prints
`T[@Col]` outside the table. Write probe cases (excel-probe skill): `=T[[#This Row],[Col]]` and `=T[@Col]` typed
and stored (a) outside T on a data row, (b) outside T on a non-data row, (c) in T's header row, (d) in T's totals
row, each with FORMULATEXT beside it. Record the findings in syntax.md; file an engine task if ours differs.

## Acceptance criteria
- [ ] Probe cases with excel-graph oracle values committed; syntax.md records each shape.
- [ ] An engine task is filed for every shape where ours differs (or none differs).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing the printing change.
