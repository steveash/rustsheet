---
id: T-3168
title: "Legacy CHOOSE with an array-constant index: @ goes on the call, not the constant (=@CHOOSE({1,2},INDEX(B1:B6,3),5))"
milestone: M2
lane: engine
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A legacy CHOOSE whose index is an array constant shows the `@` on the call, as Excel does:
`=@CHOOSE({1,2},INDEX(B1:B6,3),5)` and `=ABS(@CHOOSE({1,2},INDEX(B1:B6,3),5))`, not `=CHOOSE(@{1,2},...)`.

## Context
- Known failures probes/t3134-legacy-if-index-arm-shapes I54 and K54 (excel-graph, T-3134). Values (3, 3; SUM 8)
  already pass; only FORMULATEXT differs.
- docs/excel-semantics/references.md, IF/IFS arm section (T-3134 paragraph): an array condition or index keeps
  the `@` on the call (`=@IF({TRUE,FALSE},INDEX(B1:B6,3))` already passes, I50).
- The `@` on the constant comes from the array-constant scalar-argument rule (T-1633, `add_array_at` in
  crates/rustsheet-engine/src/expressions/parser/static_analysis.rs), applied to CHOOSE's index (legacy.rs
  `index == 0` for CHOOSE). Check which other probes pin `CHOOSE(@{..},...)` before changing it.

## Acceptance criteria
- [ ] I54 and K54 pass; every other cell of probes/t3134-legacy-if-index-arm-shapes and the legacy CHOOSE probes keeps
      passing (`make eval-check`: 0 regressions, `make eval-baseline` run).
- [ ] `make check` green; reviewers approve.

## Out of scope
- IF and IFS conditions (already evidenced, T-3134).

## Progress notes (opus run 2026-10-30, stopped at the session budget; resumed by a later opus run)
- Evidence in eval/suites/probes: with range arms Excel keeps both `@`s, `=@CHOOSE(@{1,2},H1:H2,I1:I2)`, also under
  ABS and LEN (legacy-choose-array-index, subtotal-take-array-selector). With scalar arms (a scalar, or a single-cell
  INDEX) it shows only the call's `@`, `=@CHOOSE({1,2},INDEX(B1:B6,3),5)` (t3134 I54, K54). Under N, T, SUM or ROWS no
  `@` appears in either case.
- Mechanism: `add_array_at` (static_analysis/legacy.rs) puts `@` on CHOOSE's index constant, because `is_choose_index` makes it
  an array context. The index is then `@{1,2}`, which is Scalar. So `returns_single_cell_index` (static_analysis/if_arm.rs) makes the
  call Scalar and the call gets no `@`.
- Likely fix: in the `Node::FunctionKind` loop of `add_legacy_intersection` (static_analysis.rs ~line 220), skip the
  index-constant `@` for CHOOSE when every value arm is Scalar or `is_single_cell_index`. Mirror that in
  `remove_array_at`. The call then has shape Array(1, 2), and `legacy_array_call_at` (returns_an_argument) already adds its `@`.
  Verify that the range-arm probes keep `@{1,2}`.
