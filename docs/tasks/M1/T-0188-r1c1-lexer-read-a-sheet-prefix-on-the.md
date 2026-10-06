---
id: T-0188
title: "R1C1 lexer: read a sheet prefix on the second corner (Data!R1C1:Data!R2C2)"
milestone: M1
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A range whose two corners both carry the sheet prefix survives the internal R1C1 form:
`Data!B8:Data!E12` printed as `Data!R[7]C[1]:Data!R[11]C[4]` parses back to the same tree.

## Context
Found in T-0134. Formulas are stored in R1C1 (`stringify::to_rc_format`). The A1 lexer stops a
range token at the first corner when the second carries a prefix (`sheet_prefix_after_colon`,
crates/rustsheet-engine/src/expressions/lexer/ranges.rs, T-0130), so the parser joins the two
references with `OpRangeKind`. `consume_range_r1c1` lacks that check: re-lexing
`SUM(Data!R[7]C[1]:Data!R[11]C[4],R[0]C[0])` or `(Data!R[7]C[1]:Data!R[11]C[4],R[0]C[0])` fails
with "Expecting reference in range". So does any code path that re-parses a stored R1C1 formula
with such a range.
Semantics: docs/excel-semantics/references.md "Range corners may both carry the sheet prefix".

## Acceptance criteria
- [ ] `consume_range_r1c1` ends the token at the first corner when the text after `:` is a sheet
  prefix, as the A1 path does. Unit test: the A1 → R1C1 → tree round trip of
  `SUM(Data!B8:Data!E12,A1)`, `(Data!B8:Data!E12,A1)` and `Data!B8:Data!E12 Data!B12`.
- [ ] The T-0134 parser test `sheet_qualified_corner_pairs_are_union_operands` also checks R1C1.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Column and row corners with a second prefix (not ranges in Excel, T-0139).
