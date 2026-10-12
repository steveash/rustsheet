---
id: T-1342
title: "Fold union and cut written counts into Group nodes (SPEC-0007 step 3)"
milestone: M1
lane: engine
status: in-progress
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
- [ ] No `written` field on OpUnionKind/OpIntersectKind; test_union and the T-1176/T-1024 tests unchanged.
- [ ] 0 eval regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
