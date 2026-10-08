---
id: T-1176
title: "Keep the inner parentheses of a parenthesized union cut in FORMULATEXT"
milestone: M1
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
FORMULATEXT of `=SUM(((A1,B1) B1))` prints the written text with both parentheses, as Excel does.

## Context
probes/union-cut-precedence F6 (oracle excel-graph, T-1140): Excel prints `=SUM(((A1,B1) B1))`; the engine prints `=SUM((A1,B1) B1)`, dropping the union's own parentheses when the union is the left operand of a cut and the sole argument. Compare the `written` handling on `OpUnionKind` (T-1024) and `Parser::parse_parenthesized`.

## Acceptance criteria
- [ ] probes/union-cut-precedence F6 passes and `make eval-baseline` records the fix.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`OpIntersectKind` gained `written` (parentheses levels around a cut whose leftmost operand is a parenthesized union); the parser sets it and `stringify` prints it. The cut now also prints its written parentheses inside larger expressions (`-((A1,B1) B1)`, `SUM(((A1,B1) B1),1)`), matching the typed text but without an oracle cell of its own (T-1178); other parenthesized cuts are unchanged. The whole-formula, `@` and `:` wrappers do not count those levels twice (unit tests in test_union.rs). Eval: probes known failures 394 -> 393 (F6 fixed), no regressions.
