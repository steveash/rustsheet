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
