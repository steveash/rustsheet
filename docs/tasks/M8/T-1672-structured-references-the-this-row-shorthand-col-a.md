---
id: T-1672
title: "Structured references: the @ this-row shorthand ([@Col], [@[A]:[B]], [@])"
milestone: M8
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Formulas using Excel's `@` shorthand for `#This Row` parse and evaluate like their `[#This Row]` forms:
`Sales[@Qty]`, `Sales[@[Unit Price]]`, `Sales[@[A]:[B]]` and `Sales[@]`.

## Context
- The lexer (`crates/rustsheet-engine/src/expressions/lexer/structured_references.rs`) knows only `[[#This Row],[Col]]`; every `@` form is a parse error (`#ERROR!`).
  The file form is `[#This Row]` (spec2xlsx `table_refs_to_file_form`), so xlsx import works; typed
  formulas and the eval adapter (which enters the spec text) do not.
- Evidence: gen-m8b in-table (19fdc25163755713, 23/160 pass) and this-row (e1bf5aa0ffb4ed87, 59/400):
  about 460 failing cells, nearly all `#ERROR!`. Semantics: docs/excel-semantics/references.md
  "Structured references: this row, totals and ranges".
- An escaped `'@` inside a column name is a literal `@` (already unescaped by `consume_column_reference`).

## Acceptance criteria
- [x] `[@Col]`, `[@[Col]]`, `[@[A]:[B]]` and `[@]` lex to the same tokens as their `#This Row` forms
  (lexer unit tests); FORMULATEXT showing the `@` form as typed is NOT covered here (the lexer drops the spelling; unverified).
- [x] gen-m8b in-table and this-row cells that only failed on the `@` syntax pass; `make eval-baseline`.
- [x] `make check` green; reviewers approve.

## Out of scope
- Case-insensitive names (T-1673), rows outside the table (T-1675).

## Outcome
Lexer now accepts `[@]`, `[@Col]`, `[@[Col]]` and `[@[A]:[B]]` (`consume_this_row_shorthand`), producing the same tokens as the `#This Row` forms; unit test compares each against its long form. gen-m8b is ungated (no baseline file), so no baseline change: in-table 19fdc25163755713 failing cells 137 -> 5, this-row e1bf5aa0ffb4ed87 341 -> 6 (remaining are rows outside the table / name cases, T-1673/T-1675). `make check` green. FORMULATEXT of the `@` form was not tested or implemented.
