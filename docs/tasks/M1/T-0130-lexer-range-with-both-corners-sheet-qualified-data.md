---
id: T-0130
title: "Lexer: range with both corners sheet-qualified (Data!B8:Data!E12)"
milestone: M1
lane: engine
status: todo
priority: P1
size: S
tier: sonnet
depends_on: [T-0105]
---

## Goal
`Data!B8:Data!E12` (the second corner repeats the same sheet prefix) parses as the range
`Data!B8:E12`, as in Excel.

## Context
gen-m1 (T-0109 triage, oracle excel-graph 2026-10-05): the 41 cells tagged `range/ref-colon-ref`
in `gen-m1/1ce5b2467b3b19fd` give `#ERROR!` in the engine (a parse error):
A10 A13 A15 A16 A18 A19 A24 A26 A29 A30 A36 A38 A48 A54 A56 A68 A70 A72 A83 A98 A103 A107 A117
A121 A126 A127 A137 A151 A155 A160 A167 A180 A183 A189 A198 A201 A229 A233 A239 A242 A247.
- `Data!B8:Data!E12 Data!B12` (A10): Excel 1e-15 (Data!B12), engine `#ERROR!`.
- `Data!C2:Data!E4 Data!C4` (A16): Excel 100, engine `#ERROR!`.
- `Data!C3:Data!D12 Data!C12` (A121): Excel `#DIV/0!` (Data!C12's error), engine `#ERROR!`.
Semantics: docs/excel-semantics/references.md "Range corners may both carry the sheet prefix".

Root cause: `expressions/lexer/ranges.rs` `consume_range_a1` sees `B8:`, then calls
`consume_reference_a1` on `Data!E12`. That fails, so the lexer returns the Illegal token "Expecting
reference in range". The range-of-ranges form `Data!A7:A7:Data!C11` already parses, because the
lexer emits the range `Data!A7:A7`, then `Colon`, then a sheet reference, and
`parser::parse_range` / `Model::get_range` join single-cell ranges. Its 53 cells fail only on the
intersection (T-0105). The fix: when the text after `:` is sheet-qualified, end the token at the
first corner so that the existing `OpRangeKind` path builds the range. Every one of the 41 cells also
contains a trailing intersection, so they can only pass once T-0105 has landed.

## Acceptance criteria
- [ ] The 41 `range/ref-colon-ref` cells listed above disappear from eval/baseline/gen-m1.txt,
  with no new F lines in any suite.
- [ ] Unit test: `=SUM(Data!B8:Data!E12)` evaluates the same as `=SUM(Data!B8:E12)`, and the formula
  round-trips through stringify.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Corners on different sheets (`Data!A1:'My Data'!B2`): no gen-m1 evidence (see T-0122 for 3-D refs).
