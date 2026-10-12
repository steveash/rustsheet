---
id: T-1342
title: "Fold union and cut written counts into Group nodes (SPEC-0007 step 3)"
milestone: M1
lane: engine
status: done
priority: P3
size: M
tier: opus
depends_on: [T-1340]
spec: docs/specs/SPEC-0007-written-parentheses.md
---

## Goal
`OpUnionKind.written` and `OpIntersectKind.written` are replaced by `Group` nodes, with the same text and values.

## Context
SPEC-0007 Rollout step 3. `bare` on OpUnionKind stays. `union_text`'s stripping of a left union's inner parentheses becomes plain Group printing. The cut of a written union (T-1176) is `Group{1, Intersect(Group{1, Union}, ..)}`.

## Acceptance criteria
- [x] No `written` field on OpUnionKind/OpIntersectKind; test_union and the T-1176/T-1024 tests unchanged.
- [x] 0 eval regressions.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`OpUnionKind.written` and `OpIntersectKind.written` are gone. `parse_parenthesized` gives a union, and a cut whose leftmost operand is a parenthesized union, its own `Group` (`keeps_own_group`, `regroup`) wherever it stands, at the root and as an `@` operand too, so plain `parse` and `parse_cell_formula` print the same text. `((A1,B1) B1)` is `Group{1, Intersect(Group{1, Union}, B1)}`. `union_text` prints `left,right` only; `parse_cell_formula`, `intersection_operand`, `tight_operand`, `at_operand` and `callee_operand` lose their `written` special cases. As a side effect, the cut/paste text (`to_string_moved`) now keeps the parentheses around a cut of a union (it used to drop them). test_union, the T-1176/T-1024 print tests and the depth tests keep every text and value assertion; only internal shape matches peel the group. Eval: 0 regressions, 0 fixed, known failures unchanged in every suite.
