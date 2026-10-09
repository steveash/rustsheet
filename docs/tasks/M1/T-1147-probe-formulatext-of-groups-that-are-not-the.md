---
id: T-1147
title: "Probe FORMULATEXT of @ groups that are not the whole operand (@(A1 B1) C1, @(A1):B1 C1, @(A1) B1 C1)"
milestone: M1
lane: research
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's FORMULATEXT for `@` operands whose parenthesized group is not the whole operand is known, and the engine matches it.

## Context
T-1134 prints a group that is exactly the leftmost operand (`@(A1) B1`, oracle E34:E35). Shapes `=@(A1 B1) C1`, `=@((A1 B1)) C1`, `=@(A1):B1 C1`, `=@(A1) B1 C1`, `=@(A1) B1:C1` are unprobed; the engine prints the first three without the group, and the unit test `at_operand_keeps_a_group_that_is_only_its_leftmost_operand` records that as hand-written. Code: `parse_intersection` (parser/reference_operators.rs), `lead_group_text`.

## Acceptance criteria
- [ ] A probe case with excel-graph values for the five shapes exists; `docs/excel-semantics/references.md` records them; mismatches are filed as engine tasks.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
