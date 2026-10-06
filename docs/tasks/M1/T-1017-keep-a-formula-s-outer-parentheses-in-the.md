---
id: T-1017
title: "Keep a formula's outer parentheses in the AST and stored text (grouped root) for near-zero snapping"
milestone: M1
lane: engine
status: todo
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
`OpUnionKind` already carries a `grouped: bool` for the same reason (printing keeps its parentheses); follow that precedent, for example a `grouped` flag on the root `OpSumKind` / SUM `FunctionKind` or one flag for any root node. Check with the architecture reviewer whether this needs a short spec (the AST change touches about 30 `OpSumKind` sites).
Branch `agent/engine/T-0110-near-zero-snapping` has the snapping itself (`snap_near_zero`, `handle_outermost_sum`, `fn_sum_outermost`, and the root match in `compute_formula`); it needs this flag to skip grouped roots.

## Acceptance criteria
- [ ] `=(A1-1)` and `=A1-1` parse to distinguishable ASTs, and both print back unchanged (tests in parser/tests). Excel-authored workbooks round-trip their outer parentheses.
- [ ] No eval regressions, and `make check` green.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The snapping itself (T-0110).
