---
id: T-1017
title: "Keep a formula's outer parentheses in the AST and stored text (grouped root) for near-zero snapping"
milestone: M1
lane: engine
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
The parsed formula records whether its outermost expression was written in parentheses (`=(A1-1)` versus `=A1-1`), and the stored formula text keeps those parentheses, so evaluation can tell the two apart after any round trip.

## Context
T-0110 found that Excel snaps a near-zero result of the outermost `+`/`-`/SUM to 0, but **not** when the whole formula is parenthesized: probes-numeric/30bc5f8fab02d0a2, the `parenthesized-sub` cells (`=(Inputs!A2-1)`), 114 cells. docs/excel-semantics/numbers.md has the details under "Near-zero snapping in addition and subtraction".
The parser drops grouping parentheses (`parse_parenthesized` in `expressions/parser/reference_operators.rs` returns the inner node), and the stored formula is `to_rc_format` of the AST (`Model::shared_formula_index`), so `=(A1-1)` is stored as `=A1-1`.
`OpUnionKind` already carries a `grouped: bool` for the same reason (printing keeps its parentheses); follow that precedent with **one root-level flag** (only the outermost expression matters), not per-node flags on `OpSumKind`, which would touch about 30 sites. That precedent means no spec is needed; if the change turns into per-node flags, split the task first.
Branch `agent/engine/T-0110-near-zero-snapping` has the snapping itself (`snap_near_zero`, `handle_outermost_sum`, `fn_sum_outermost`, and the root match in `compute_formula`); it needs this flag to skip grouped roots.

## Acceptance criteria
- [ ] `=(A1-1)` and `=A1-1` parse to distinguishable ASTs, and both print back unchanged (tests in parser/tests). Excel-authored workbooks round-trip their outer parentheses.
- [ ] The stored formula text keeps `=(A1-1)` (`Model::shared_formula_index`), and an xlsx import/export round trip keeps it.
- [ ] No eval regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The snapping itself (T-0110).

## Outcome
The parser marks a cell formula written wholly in parentheses with a root-only `Node::GroupedRootKind` wrapper (`Parser::parse_cell_formula`; defined names and CF formulas keep `Parser::parse`). A wrapper was chosen over a flag outside the AST because every printer (stored R1C1 text, FORMULATEXT/display, xlsx export, moves, renames) then keeps the parentheses with no extra plumbing, and re-parsing the stored text restores it; every exhaustive walker delegates to the child, and SUBTOTAL's nested-SUBTOTAL check unwraps it. Unions are not wrapped (they keep `written`). `=((A1-1))` keeps one level (engine-wide gap, noted in numbers.md). Tests: parser/tests/test_grouped_root.rs and an xlsx round trip (rustsheet-xlsx/tests/test_grouped_root_formula.rs). Eval: 0 regressions, no change in any suite. T-0110 can now skip snapping for a grouped root.
