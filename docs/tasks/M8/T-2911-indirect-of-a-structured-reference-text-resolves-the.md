---
id: T-2911
title: "INDIRECT of a structured reference text (INDIRECT(Sales[Qty])) resolves the table column as Excel does"
milestone: M8
lane: engine
status: done
priority: P1
size: S
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
INDIRECT of a text that is a structured reference (`INDIRECT("Sales[Qty]")`) returns the table area Excel does,
instead of `#REF!`.

## Context
- Decision 0188 (owner's 2026-10-31 rulings, R1(c)): this task owns probe-row or text cells of its milestone's exit,
  which Phase 1 still counts in full, so it is in the semantics stage at P1 (was P3).
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

## Outcome
`indirect_of` falls back to the parser's structured-reference lowering (`Model::parser`) when the text is
neither A1 nor a defined name, so `INDIRECT("Sales[Column2]")` is the table column. Documented in lookup.md.
Three probe cells (F!A4 of both table-insert-*-inside-header cases and F!A5 of the two-column case) were fixed and left the baseline; the header case's F!A5 `#REF!` keeps passing. Only the plain `Table[Column]` shape resolves; other shapes are follow-up T-3386.
