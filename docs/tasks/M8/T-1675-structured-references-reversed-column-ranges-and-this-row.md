---
id: T-1675
title: "Structured references: reversed column ranges and #This Row outside the table's rows"
milestone: M8
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`Sales[[#This Row],[Qty]]` read from a row outside the table's data rows is `#VALUE!`, and a
column range written right to left (`Sales[[C]:[A]]`) is the same area as `Sales[[A]:[C]]`.

## Context
- The engine reads the header or totals row (or a blank) for `#This Row` on such a row; Excel
  gives `#VALUE!` (gen-m8b this-row e1bf5aa0ffb4ed87: Data!AE19, AF17, AH22, AA1).
- Reversed column ranges give COLUMNS 0 or 18446744073709551616 (an unsigned wrap): gen-m8b
  column-range 44cecbb1ecd5b2da K143 `COLUMNS(Sales[[2024]:[a.b]])` = 2 in Excel; escaped
  d86ef297e840325f K256. Find the subtraction (expressions/parser/operands.rs, structured
  reference evaluation) and check it cannot overflow or panic on untrusted input.
- docs/excel-semantics/references.md "Structured references: this row, totals and ranges".

## Acceptance criteria
- [x] Both rules hold (unit tests), with no unsigned wrap or panic for any column order.
- [x] The gen-m8b cells failing only on these pass; `make eval-baseline`.
- [x] `make check` green; reviewers approve.

## Out of scope
- The `@` shorthand (T-1672).

## Outcome
`#This Row` off the table's data rows is `#VALUE!` and a column range is normalised to min/max column in `structured_reference_operand` (operands.rs); the stored columns are indexes into the table, so no unsigned wrap remains. Unit tests in `parser/tests/test_tables.rs`. gen-m8b (ungated) cells fixed, none broken (reviewer's with/without comparison): this-row e1bf5aa0ffb4ed87 224 -> 400/400, in-table 19fdc25163755713 146 -> 160/160, column-range 44cecbb1ecd5b2da 370 -> 389 (K143 and other reversed/#Headers/#All/#Data ranges, which had wrapped to 1.8446744073709552e19), escaped d86ef297e840325f 386 -> 388 (K256, K315). gen-m8b is not a gated suite, so no baseline file changed; the remaining cells in those two cases are totals-row cells (T-1671).
