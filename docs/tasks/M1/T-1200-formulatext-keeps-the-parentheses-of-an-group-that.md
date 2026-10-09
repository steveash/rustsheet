---
id: T-1200
title: "FORMULATEXT keeps the parentheses of an @ group that is not the whole operand"
milestone: M1
lane: engine
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
FORMULATEXT prints an `@` operand's parenthesized group when the group is only the leftmost part of the operand, as Excel does.

## Context
probes/at-group-not-whole-operand (T-1147, excel-graph): `=@(A1 B1) C1`, `=@((A1 B1)) C1` and `=@(A1):B1 C1` print as written; the engine prints `=@A1 B1 C1`, `=@A1 B1 C1` and `=@A1:B1 C1`. Failing cells E30, E31, E32. T-1134 keeps a group that is exactly the leftmost operand (`ImplicitIntersection::lead`, `lead_group_text`, `parse_intersection` in parser/reference_operators.rs); extend it to a group followed by `:` or an operand that is itself an intersection. See references.md "@ operand".

## Acceptance criteria
- [ ] E30:E32 of probes/at-group-not-whole-operand pass; `make eval-baseline` records them; the unit test `at_operand_keeps_a_group_that_is_only_its_leftmost_operand` is updated to the oracle values.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
