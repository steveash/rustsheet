---
id: T-3069
title: "Structured references: this-row FORMULATEXT, stored padded T[ Col ] is #REF!"
milestone: M2
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
FORMULATEXT of a this-row structured reference inside a table, and loading a stored padded bare column, match Excel.

## Context
T-2800 evidence (docs/excel-semantics/syntax.md, "Whitespace beside a missing argument"): inside a table Excel prints
`T[[#This Row], [Col]]` as `[@Col]` and the padded forms as `[ @Col ]`; a stored `SUM(T[ Col ])`, `T[ Col]` or
`T[Col ]` is `#REF!` (FORMULATEXT `=SUM(#REF!)`) and a stored `T[ #Data ]` makes the workbook unloadable (outside
our scope; ignore). Failing cases: probes/t2800-this-row-whitespace (B3:E3, F2, F3) and probes/t2800-stored-ws-0; stored-ws-1 and -2 (one-sided padding) already pass.
`[@ Col]` (F2) must also parse: it is `#ERROR!` for us, 5 in Excel.
Note T-2801/T-2962 chose the stored `T[ Col ]` round-trip for our own export; the export must then not write that form.

## Acceptance criteria
- [x] The listed probe cases pass; baseline updated.
- [x] Our xlsx export no longer writes a padded bare column Excel reads as `#REF!`.
- [x] `make check` green; reviewers approve.

## Out of scope
- The unloadable-workbook case `T[ #Data ]`.

## Outcome
A `#This Row` reference in a cell of its own table now prints in Excel's `@` form (`T[[#This Row], [Col]]` is
`[@Col]`), keeping the padding inside the outer brackets (`[ @Col ]`), and `[@ Col]` parses (the run after `@` is
that padding). A padded bare column `T[ Col ]` is accepted only in our own stored R1C1 text; as A1 text it is a parse
error typed and `#REF!` loaded, as in Excel, and the xlsx export writes `T[ [Col] ]`. Eval: probes known failures
2575 -> 2565 (the eight t2800 cells plus table-edits/table-insert-row-calc-column-inside-data A1:A2), no regressions.
