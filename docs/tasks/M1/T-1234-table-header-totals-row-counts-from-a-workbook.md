---
id: T-1234
title: "Table header/totals row counts from a workbook use unchecked i32 arithmetic in structured_reference_operand (follow-up of T-1210)"
milestone: M1
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A table whose `header_row_count` or `totals_row_count` is huge (an untrusted xlsx) cannot overflow or wrap the row arithmetic in `structured_reference_operand`; it gives `#REF!`.

## Context
From the T-1210 security review: `table.totals_row_count as i32`, `header_row_count as i32` and `row_end -= totals_row_count`, `row_start += header_row_count`, `row_end += 1` in crates/rustsheet-engine/src/expressions/parser/operands.rs are unchecked i32 arithmetic on workbook values (panic in debug, wrap in release). Use checked arithmetic and return `Node::ErrorKind(token::Error::REF)`; add a unit test next to `malformed_table_reference_is_ref_error_not_a_panic` in parser/tests/test_tables.rs.

## Acceptance criteria
- [ ] a table with huge header/totals row counts is `#REF!` (unit test), no panic or wrap
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`structured_reference_operand` converts the workbook header/totals row counts with `i32::try_from` and does the row arithmetic with checked ops; anything that does not fit is `#REF!` (unit test `huge_header_or_totals_row_counts_are_ref_error_not_an_overflow`, red before the change). No eval cells change.
