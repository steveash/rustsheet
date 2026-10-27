---
id: T-2911
title: "INDIRECT of a structured reference text (INDIRECT(Sales[Qty])) resolves the table column as Excel does"
milestone: M8
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
INDIRECT of a text that is a structured reference (`INDIRECT("Sales[Qty]")`) returns the table area Excel does,
instead of `#REF!`.

## Context
Found by T-2339. Excel resolves a structured reference written as INDIRECT text: after a column insert inside
`Sales` (S!B3:C7), `=ROWS(INDIRECT("Sales[Column2]"))` is 4 and `=COUNTA(INDIRECT("Sales[Column1]"))` is 0 (an
empty data column), while a name the table lacks is `#REF!` (probes/table-edits/table-insert-column-inside-header
F!A4, F!A5; table-insert-two-columns-inside-header F!A4, F!A5; excel-graph 2026-10-27). Three of them fail; the
header case's F!A5 (`#REF!`, no `Column2`) passes only because every structured text is `#REF!` today and must keep
passing. The engine's
`indirect_of` (functions/lookup_and_reference/mod.rs) parses only A1/R1C1 references and defined names, so
every structured text is `#REF!` (COUNTA counts it: 1). Reuse the parser's structured-reference resolution
(expressions/parser/structured_reference.rs) rather than a second one (directive 3). Specifiers (`#All`,
`#Headers`, `@`), case and spacing of the text are unprobed: probe what is not covered before generalising.

## Acceptance criteria
- [ ] The three failing cells above pass (and the fourth keeps passing) and leave eval/baseline/probes.txt, with no new known failure.
- [ ] references.md (or lookup.md's INDIRECT entry) records the rule with the probe ids.
- [ ] `make check` green; reviewers approve.

## Out of scope
- `#This Row` (`Sales[@Qty]`) through INDIRECT, unless probed in the same task.
