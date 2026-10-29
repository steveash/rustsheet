---
id: T-2970
title: "The legacy @ rewrite looks past typed whitespace (N/T over IF keeps its spaces)"
milestone: M2
lane: engine
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
A formula written with whitespace keeps it through the legacy `@` rewrite (import and export) in every shape the bare
formula takes, so `=N( IF(1,{2;3}) )` saves and reads back with its spaces.

## Context
T-2948 made `add_implicit_intersection` and `remove_redundant_implicit_intersection` decide on the tree with no
whitespace (`Node::rewrite_as_unspaced`): the written tree keeps its spaces only when its rewrite agrees with the bare
one, else the formula loses its whitespace (never a value). Some shape sites in
`expressions/parser/static_analysis/legacy.rs` do not look past `Node::Spaced`, so they disagree and the whitespace is
dropped. Known: `N( IF(1,{2;3}) )` (`leaves_call_whole` via `under_at`, and at least one more site: peeling `Spaced` in
`under_at` alone did not fix it). The test `spaced_formulas_keep_their_whitespace_and_their_at`
(`expressions/parser/tests/test_implicit_intersection_export.rs`) records the current fallback for that case.

## Acceptance criteria
- [ ] `N( IF(1,{2;3}) )`, `T( IF(1,{"a";"b"}) )` and `N( CHOOSE(1,{2;3}) )` keep their whitespace through
      `add_implicit_intersection` and the export, with the `@`s of the bare formula (update the test's expectation).
- [ ] The shape sites peel `Spaced` with the existing `Node::unwrap_spaced`; no new helper.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Removing the `rewrite_as_unspaced` guard: it stays as the safety net for sites not yet covered.
