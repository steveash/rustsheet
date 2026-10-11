---
id: T-1341
title: "Fold GroupedRootKind into a root Group node (SPEC-0007 step 2)"
milestone: M1
lane: engine
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: [T-1340]
spec: docs/specs/SPEC-0007-written-parentheses.md
---

## Goal
`GroupedRootKind` is replaced by a root `Group`, with the same text and values.

## Context
SPEC-0007 Rollout step 2. Pin the root near-zero snap first (`=(0.1+0.2-0.3)` skips it; `evaluate_formula_root` matches `OpSumKind` on the unpeeled root). Remove `parse_cell_formula`'s wrapping and the `grouped_root`/`root_group`/`leading_paren` parser state.

## Acceptance criteria
- [ ] No `GroupedRootKind` left; test_grouped_root's print and value assertions unchanged.
- [ ] 0 eval regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`Node::GroupedRootKind` is gone: `parse_cell_formula` wraps a wholly parenthesized formula in one root `Node::Group` holding its levels, and every walker's `GroupedRootKind` arm is deleted. `evaluate_formula_root` needed no change (a `Group` root is not an `OpSumKind`, so the near-zero snap is skipped as before; pinned by test_near_zero_snapping and probes/near-zero-snap-groups). The parser's `grouped_root` state stays: plain `parse` (defined-name bodies) still strips the root group, which the spec's later parser clean-up does not cover. Eval: 0 changes, 0 regressions.
