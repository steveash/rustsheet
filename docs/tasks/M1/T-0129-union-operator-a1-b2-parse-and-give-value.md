---
id: T-0129
title: "Union operator (A1,B2): parse and give #VALUE! in value contexts"
milestone: M1
lane: engine
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-0105]
---

## Goal
A parenthesized reference union `(A1,B2)` parses and evaluates like Excel: `#VALUE!` wherever a
value is needed, and the single cell when an intersection cuts the union down to one cell.

## Context
gen-m1 (T-0109 triage, oracle excel-graph 2026-10-05): all 250 cells of `gen-m1/c940d3cfb0d3b083`
fail. The engine gives `#ERROR!` (parse error) for every union. Forms and Excel's results:
- `union/same-cell` (37): `(Data!E4,Data!E4)` (A1) = `#VALUE!`, even when both areas are the same cell.
- `union/bare` (49): `(Data!B6,Data!E11)` (A4) = `#VALUE!`.
- `union/arith` (50): `(Data!E12,Data!A9)/1` (A2) = `#VALUE!`.
- `union/concat` (43): `(Data!C12,Data!E9)&""` (A14) = `#VALUE!`.
- `union/nested` (39): `((Data!A2,Data!E8),Data!B6)` (A25) = `#VALUE!`.
- `union/cut-by-cell` (32): `(Data!C1:C12,Data!E1:E12) Data!C9` (A5) = 0 (Data!C9 is empty);
  `(Data!D1:D12,Data!A1:A12) Data!D11` (A12) = "a"; `(Data!A1:A12,Data!C1:C12) Data!A7` (A40) =
  `#N/A` (the cell's own error). This form needs the space operator (T-0105).
Semantics: docs/excel-semantics/references.md "Union (comma) reference in a value context is #VALUE!".

Design points (hence opus, M): the lexer and parser must tell a union comma inside parentheses
apart from a function-argument separator (`SUM((A1,B2))` vs `SUM(A1,B2)`), and in comma-decimal
locales the union operator is a different character. A multi-area reference has no `CalcResult`
representation today. Choose one deliberately (a new node and result variant, or a value-context
`#VALUE!` plus an intersection-only path) and keep it reusable for AREAS/INDEX area_num and
SUM((A1,B2)) later.

Code: `expressions/parser/mod.rs` `parse_primary` (the `LeftParenthesis` branch expects `)`);
`expressions/parser/stringify.rs`; `model.rs` `evaluate_node_in_context`, `cast.rs`.

## Acceptance criteria
- [ ] Every F line of `gen-m1/c940d3cfb0d3b083` (250 cells) disappears from eval/baseline/gen-m1.txt,
  with no new F lines in any suite.
- [ ] `(A1,B2)` formulas round-trip through stringify (unit test); `SUM(A1,B2)` still parses as two arguments.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Unions as function arguments (`SUM((A1,B2))`, `AREAS`, `INDEX(…,area_num)`): no gen-m1 evidence;
  file a follow-up with an excel-probe if the representation chosen makes it cheap.
