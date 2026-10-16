---
id: T-1674
title: "A bare table name and Table[] evaluate as the table's data area"
milestone: M8
lane: engine
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A bare table name (`=ROWS(Sales)`, `VLOOKUP(2,Sales,1,FALSE)`, `INDEX(Sales,2,1)`) and `Sales[]`
evaluate as the table's data area, as in Excel, instead of `#NAME?`.

## Context
- `consume_structured_reference` (`crates/rustsheet-engine/src/expressions/lexer/structured_references.rs`) turns `Sales[]` into `TokenType::Ident("Sales")`, and a
  bare identifier is looked up only among defined names, so both are `#NAME?`.
- Evidence: gen-m8b lookups (27252667210b1d03: VLOOKUP/INDEX over a bare table, about 125 cells)
  and specials (e804cf2f8af00680 K29 `ROWS(Sales[])` = 10, K116 `ROWS(Sales)` = 10).
- Tables and defined names share one namespace (`Model::add_table`).

## Acceptance criteria
- [ ] A bare table name and `Table[]` are the table's data rows (header and totals rows excluded),
  in every function that takes a reference (unit tests).
- [ ] gen-m8b cells failing only on this pass; `make eval-baseline`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Sheet-qualified table names.
