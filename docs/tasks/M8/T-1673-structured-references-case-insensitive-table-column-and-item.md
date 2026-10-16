---
id: T-1673
title: "Structured references: case-insensitive table, column and item names, and multiple item specifiers"
milestone: M8
lane: engine
status: todo
priority: P2
size: M
tier: sonnet
depends_on: []
---

## Goal
Structured references match table names, column names and item specifiers (`#All`, `#Data`,
`#Headers`, `#Totals`, `#This Row`) ignoring case, as Excel does, and accept several item specifiers
(`Sales[[#Headers],[#Data],[Qty]]`, `Sales[[#Data],[#Totals]]`).

## Context
- `consume_table_specifier` in `crates/rustsheet-engine/src/expressions/lexer/structured_references.rs` matches the specifiers case-sensitively and takes only one;
  column and table names are looked up case-sensitively. Each miss is a parse error (`#ERROR!`).
- Evidence: gen-m8b names-case (5b5c0e20e82153bb, 32/400 pass: `SALES[qty]`, `stock[#all]`,
  `Tiny[[#HEADERS],[Cost]]` all compute in Excel) and specials (e804cf2f8af00680:
  `ROWS(Sales[[#Headers],[#Data],[Note]])` = 11, `ROWS(Sales[[#Data],[#Totals]])` = 10).
  docs/excel-semantics/references.md "Structured references: names and item specifiers".

## Acceptance criteria
- [ ] Table, column and specifier names resolve ignoring case; `[[#Headers],[#Data],...]` and
  `[[#Data],[#Totals],...]` combinations parse and give the right area (unit tests).
- [ ] gen-m8b names-case and specials cells failing only on these pass; `make eval-baseline`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The `@` shorthand (T-1672); totals rows from the eval adapter (T-1671).
