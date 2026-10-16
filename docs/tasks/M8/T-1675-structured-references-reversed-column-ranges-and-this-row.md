---
id: T-1675
title: "Structured references: reversed column ranges and #This Row outside the table's rows"
milestone: M8
lane: engine
status: todo
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
- [ ] Both rules hold (unit tests), with no unsigned wrap or panic for any column order.
- [ ] The gen-m8b cells failing only on these pass; `make eval-baseline`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The `@` shorthand (T-1672).
