---
id: T-1134
title: "FORMULATEXT keeps the parentheses of a group that is only part of an @ operand"
milestone: M1
lane: engine
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
FORMULATEXT of `=@(A1) B1` and `=@((A1)) B1` prints the parentheses as written, as Excel does.

## Context
probes/at-paren-operands-levels E34 and E35 (T-1132): Excel prints `=@(A1) B1` and `=@((A1)) B1`; the engine prints `=@A1 B1`. See references.md "`@` operand parentheses". Code: `ImplicitIntersection::written` (parser/mod.rs) and `at_operand` (parser/reference_operators.rs; T-1116).

## Acceptance criteria
- [ ] E34 and E35 of probes/at-paren-operands-levels pass and are recorded with `make eval-baseline`, with no regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
