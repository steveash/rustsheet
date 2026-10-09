---
id: T-1210
title: "A table reference whose table range fails to parse is a parse error, not a panic"
milestone: M1
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A formula with a structured reference to a table whose stored `reference` does not parse (an untrusted xlsx) is a parse error, never a panic (directive 5).

## Context
From T-1110 review: `structured_reference_operand` (crates/rustsheet-engine/src/expressions/parser/operands.rs) calls `parse_range(&table.reference).expect(..)` under `#[allow(clippy::expect_used)]`; the table metadata comes from the workbook. Return a `ParseErrorKind` (or `#REF!`) instead and drop the allow; add a unit test with a malformed table reference.

## Acceptance criteria
- [x] A malformed table `reference` yields a parse error or `#REF!`, not a panic (unit test); the `allow` is gone.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
A table whose stored `reference` does not parse is now `#REF!` in `structured_reference_operand` instead of a panic; the `expect_used` allow is gone. Unit test `malformed_table_reference_is_ref_error_not_a_panic`. No eval delta.
