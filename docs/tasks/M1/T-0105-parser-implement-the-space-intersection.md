---
id: T-0105
title: "Parser: implement the space (intersection) operator and reject trailing input"
milestone: M1
lane: engine
status: todo
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
`=A1:B2 B1:B3` must evaluate the intersection like Excel, and garbage after a complete expression must be a parse error instead of being silently dropped.

## Context
docs/research/ironcalc-assessment.md §4: parser never checks end of input (parser/mod.rs ~340).

gen-m1 (T-0109 triage, oracle excel-graph 2026-10-05): **632 failing cells** need the space operator.
The engine stops parsing after the first range and spills it (`#SPILL!`), or gives `#ERROR!`
when a `)` or `&` follows (`(A B)+1`). Every cell of two cases fails, plus three forms of a third:
- `gen-m1/4dc8ed1de05a61be` (all 250 cells: `intersection/*` forms arith, disjoint, triple, corner,
  col-row, cross-sheet).
- `gen-m1/7cb99acdd82402f1` (all 250 cells: `whole-row-col/*`).
- `gen-m1/1ce5b2467b3b19fd`: the 132 cells tagged `range/dollar-range` (42), `range/reversed-corners`
  (37) and `range/range-of-ranges` (53). The range parses already; only the trailing intersection fails.

Excel's results (expected vs engine):
- `Data!A1:C7 Data!C7:E12` (4dc8… A5) = 0.5 (the value of Data!C7); engine `#SPILL!`.
- `Data!C1:D7 Data!C1:C12 Data!A6:E6` (4dc8… A2, triple) = TRUE (Data!C6); engine `#SPILL!`.
- `Data!A3:D4 Data!Z99` (4dc8… A6, disjoint) = `#NULL!`; engine `#SPILL!`.
- `Data!C5:C10 'My Data'!C9:D9` (4dc8… A4, cross-sheet) = `#VALUE!` (not `#NULL!`); engine `#SPILL!`.
- `(Data!C2:C11 Data!C6:E6)+0.1` (4dc8… A3) = 1.1; engine `#ERROR!`.
- `Data!Z:Z Data!290:290` (7cb9… A25, an empty cell) = 0; `(Data!D:D Data!2:2)&""` (A4) = "".
- `Data!C8:B3 Data!B8` (1ce5… A9, reversed corners) = "a"; `Data!A7:A7:Data!C11 Data!A11` (A25) = 1.
- An error in the intersected cell propagates: `Data!D2:E8 Data!E8:E12` (4dc8… A16) = `#DIV/0!`.
Semantics: docs/excel-semantics/references.md "Intersection operator (space)".

Two further groups need this operator and another fix, and are scored by those tasks:
`range/ref-colon-ref` (41 cells, T-0130) and `union/cut-by-cell` (32 cells, T-0135).

Code: `expressions/lexer` (whitespace is currently skipped, so the space must become a token
where it separates two reference operands); `expressions/parser/mod.rs` `parse_range` /
`parse_expr` (end-of-input check); `model.rs` `evaluate_node_in_context`; stringify round-trip.

## Acceptance criteria
- [ ] Every F line of `gen-m1/4dc8ed1de05a61be` and `gen-m1/7cb99acdd82402f1` (500 cells) and the
  132 `range/dollar-range`, `range/reversed-corners` and `range/range-of-ranges` cells of
  `gen-m1/1ce5b2467b3b19fd` disappear from eval/baseline/gen-m1.txt, with no new F lines in any suite.
- [ ] Spec cases in `smoke` for invalid trailing input (the trailing-input half has no gen-m1
  evidence; flag hand expectations for oracle confirmation).
- [ ] Formulas with intersections round-trip through stringify (unit test).
- [ ] `make check` green; reviewers approve.

## Out of scope
Union operator (T-0135). Sheet-qualified second range corner `Data!B8:Data!E12` (T-0130).
