---
id: T-0134
title: "Union operator (A1,B2): parse and give #VALUE! in value contexts"
milestone: M1
lane: engine
status: done
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

Scope (opus, M): the lexer and parser must tell a union comma inside parentheses apart from a
function-argument separator (`(A1,B2)` vs `SUM(A1,B2)`). Evaluation stays narrow: a union in a
value context is `#VALUE!`, and an intersection that leaves one cell of one area gives that cell.
No general multi-area `CalcResult` is added here. That representation, needed for
`SUM((A1,B2))`, AREAS and INDEX area_num, is decided by the spec in T-0133 first (CLAUDE.md
directive 4).

Code: `expressions/parser/mod.rs` `parse_primary` (the `LeftParenthesis` branch expects `)`);
`expressions/parser/stringify.rs`; `model.rs` `evaluate_node_in_context`, `cast.rs`.

## Acceptance criteria
- [x] Every F line of `gen-m1/c940d3cfb0d3b083` (250 cells) disappears from eval/baseline/gen-m1.txt,
  with no new F lines in any suite.
- [x] `(A1,B2)` formulas round-trip through stringify (unit test); `SUM(A1,B2)` still parses as two arguments.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Unions as function arguments (`SUM((A1,B2))`, `AREAS`, `INDEX(…,area_num)`) and any general
  multi-area value: spec T-0133.

## Outcome
The union operator is `Node::OpUnionKind`, a binary node like `OpIntersectKind`.
- Parser: the comma is the union only directly inside parentheses. `Parser::parse_parenthesized`
  sets `union_allowed`, and `parse_primary` clears it for every other operand (calls, arrays,
  LAMBDA), so `SUM(A1,B2)` is still two arguments and `SUM((A1,B2))` one union. The tier sits
  between unary operators and the intersection. Operands must be references, as for the
  intersection after T-0148. Each union counts toward the T-0931 depth cap.
- Printing: a union always carries its parentheses (`union_text`), so it stays one argument inside
  a call. A left union operand loses its own, so `((A,B),C)` prints `(A,B,C)`, which is the same tree.
- Evaluation: a union where a value is needed is `#VALUE!`, after any operand error. The
  intersection now works on lists of areas: one area left is that area, several are `#VALUE!`,
  none is `#NULL!`. Every area list is capped at 65,536 areas, both intersection products and
  accumulated unions (also balanced ones), one reference expression compares at most 1,048,576
  pairs, and a union in a value context keeps only its area count (security review; DEVIATION,
  references.md "Union area cap"). Two plain areas still intersect without an area list.
- ISREF, LET variable ids, units, static analysis, the sheet and name renames, bound-variable
  export and move_formula all handle the new node.
- To keep the oversized files from growing, `argument_separator` replaces four copies of the
  locale's separator in stringify.rs, and the `^` arm uses the `sub` closure.

Eval:
- gen-m1: 258 → 8 known failures (all 250 cells of c940d3cfb0d3b083 fixed).
- probes: 107 → 106 (intersection-function-operands H5, `=A1:A3 (A2:B2,A2)` = `#VALUE!`).
- No suite regressed (fixtures, gen-m2, ironcalc-calc, smoke unchanged). poi (fetched locally, not
  gated in CI) has no regressions. It shows 8 cells already fixed on main (DAY/MONTH/YEAR(0) and
  61652.xlsx), and their baseline lines were left alone.
- Duplication 7,881 → 7,825 lines. Size locks: stringify.rs 1244 → 1158, parser/mod.rs
  1254 → 1249, static_analysis.rs 2161 → 2158, model.rs 3603 → 3580. Part of the stringify.rs
  and model.rs drop was already on main (decision 0010).

Engine behaviour chosen without evidence is recorded as UNVERIFIED in references.md ("Union:
parsing and evaluation the engine chose without evidence"): precedence against arithmetic,
operand refusal, error order, cross-sheet unions, ISREF, the comma-decimal separator, and
top-level unions in defined names (Print_Area). Known gaps left to the T-0133 spec: unions as
function arguments (`SUM((A1,B2))`, AREAS, INDEX area_num), and unions returned by IF, CHOOSE or LET.
Probe task for the UNVERIFIED choices: T-0185.

Follow-up: T-0184. The R1C1 lexer does not read a sheet prefix
on the second corner, so `SUM(Data!B8:Data!E12,A1)` fails its R1C1 round trip. This was already
broken on main and is not specific to unions.
