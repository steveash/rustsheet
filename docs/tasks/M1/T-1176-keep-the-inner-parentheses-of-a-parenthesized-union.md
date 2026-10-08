---
id: T-1176
title: "Keep the inner parentheses of a parenthesized union cut in FORMULATEXT"
milestone: M1
lane: engine
status: in-progress
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
