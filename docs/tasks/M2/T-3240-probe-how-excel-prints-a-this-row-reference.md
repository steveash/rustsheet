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
Since T-3241 the printing cell decides the form (`StructuredReference::text_at`, using the table's parse-time
`TableExtent`; crates/rustsheet-engine/src/expressions/parser/structured_reference.rs): `T[[#This Row],[Col]]` and
`T[@Col]` print `[@Col]` on a data row of T and the long form `T[[#This Row],[Col]]` anywhere else, header and
totals rows included; an unqualified `[@Col]` prints `[@Col]` anywhere in T's range, header and totals rows included,
and `T[[#This Row],[Col]]` outside it (`[Col]` likewise prints `T[Col]` outside). The only evidence is probes/t2800-this-row-whitespace (data-row cells of
a one-row table; docs/excel-semantics/syntax.md "Whitespace beside a missing argument"). Excel likely prints
`T[@Col]` outside the table. Write probe cases (excel-probe skill): `=T[[#This Row],[Col]]` and `=T[@Col]` typed
and stored (a) outside T on a data row, (b) outside T on a non-data row, (c) in T's header row, (d) in T's totals
row, each with FORMULATEXT beside it; also `=[@Col]` copied (or filled) from a data row to outside T, and a
`=[Col]`-style unqualified column reference copied out. Record the findings in syntax.md; file an engine task if ours differs.

Also probe the padded shapes T-3069 left unprobed (PR #2415 review), typed and stored: qualified `=T[@ Col]` and
`=T[ @Col ]`, one-sided `=[@Col ]` and `=[@ Col]` outside the data row, a defined name whose body is `T[ Col ]`,
and a one-sided nested `=T[ [Col]]` (ours prints `T[ Col]`, which an edit's re-parse no longer reads as the column).

## Acceptance criteria
- [ ] Probe cases with excel-graph oracle values committed; syntax.md records each shape.
- [ ] An engine task is filed for every shape where ours differs (or none differs).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing the printing change.
