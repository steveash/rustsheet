---
id: T-2322
title: "A loaded formula naming an unknown table or column is #REF! and prints #REF! (T-2197)"
milestone: M8
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
A formula in a loaded file that names a table the file does not have, or a column its table does not have,
evaluates to `#REF!` and FORMULATEXT prints the reference as `#REF!`, as Excel does.

## Context
T-2197 (excel-graph): F!A1 `=SUM(Nope[Qty])` (no table Nope) and `=SUM(Sales[Nope])` (no column Nope) load
without a repair prompt, evaluate to `#REF!`, `ISERROR` TRUE, and `=FORMULATEXT(A1)` is `=SUM(#REF!)`
(probes/table-edits/table-unknown-name-load, table-unknown-column-load). The engine keeps them as parse
errors (`#ERROR!`, FORMULATEXT the original text). docs/excel-semantics/references.md "Structured
references when the table changes". The xlsx import (crates/rustsheet-xlsx) or the parser's unknown-table
path is where the rewrite belongs; the engine API's `remove_table` path is a separate decision (the entry
records the engine's own behavior).

## Acceptance criteria
- [ ] The 4 failing cells of the two cases match Excel; their lines in eval/baseline/probes.txt disappear
  (`make eval-baseline`), no new known-failure lines.
- [ ] A unit test pins both the unknown-table and unknown-column load.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Formulas typed through the API that name an unknown table (an edit-time question, not probed).
