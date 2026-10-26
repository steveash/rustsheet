---
id: T-2762
title: "Refuse a string literal longer than 4,095 characters, typed and stored, as Excel does"
milestone: M7
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A formula holding a string literal whose value is longer than 4,095 characters is refused, as Excel refuses it: typed,
the user-input parser rejects it; stored, the importer refuses the workbook (like the 8,192-unit formula limit).

## Context
T-2739 (excel-graph 2026-10-26 UTC, syntax.md "Formula length"): a typed literal of 4,095 characters is accepted
(probes/t2739-typed-4095) and 4,096 is refused (Graph PATCH 400), in a function argument too (t2739-typed-len-4095);
a doubled quote counts once (t2739-typed-dq-4095, 4,096 source characters, is accepted; a 4,096-character value is
refused); two literals of 4,000 in one formula are fine (t2739-typed-two-4000). Stored, 4,095 loads
(t2739-stored-4095) and 4,096 makes Excel refuse the workbook (createSession 501). The engine has no such limit
(expressions/lexer `consume_string`). The count is UTF-16 units (2,047 emoji accepted, 2,048 refused:
t2739-typed-emoji-2047), as for the formula limit (T-1397); refuse in the lexer and the xlsx importer.

## Acceptance criteria
- [ ] The engine refuses a literal of 4,096 UTF-16 units typed and stored and accepts 4,095 (2,047 and 2,048 emoji too).
- [ ] syntax.md "Formula length" records the implementation.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Text a function returns (its own 32,767 limit).
