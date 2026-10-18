---
id: T-1674
title: "A bare table name and Table[] evaluate as the table's data area"
milestone: M8
lane: engine
status: done
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
- [x] A bare table name and `Table[]` are the table's data rows (header and totals rows excluded),
  in every function that takes a reference (unit tests).
- [x] gen-m8b cells failing only on this pass; `make eval-baseline`.
- [x] `make check` green; reviewers approve.

## Out of scope
- Sheet-qualified table names.

## Outcome
A bare table name (and `Table[]`, which the lexer already turned into the bare name) now resolves in the parser to the table's data area through `structured_reference_operand(name, None, None)`; the never-evaluated `Node::TableNameKind` is removed. SHEET's argument signature keeps a single reference argument unintersected (`SHEET(Table1[])` now gives the table's sheet; it was `#VALUE!` once the table became a range; calc_test_no_export/tables.xlsx guards it). gen-m8b is ungated: lookups 27252667210b1d03 now 400/400, specials e804cf2f8af00680 378/400, suite 2742 -> 2880 of 3368. Remaining failures are `#Totals` via the eval adapter (T-1671) and rows outside the table (T-1675). Unit test: `bare_table_name_and_empty_brackets_are_the_data_area`. `make check` green. Side effect: FORMULATEXT/stringify of a bare table name shows the lowered range (same as other structured references, T-1683).
Known gaps: no unit test with a totals row (needs T-1671); the stringify/FORMULATEXT/export spelling of a bare table name is the lowered range (T-1683, scope extended to the bare name).
