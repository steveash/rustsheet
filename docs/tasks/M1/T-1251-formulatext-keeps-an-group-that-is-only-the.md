---
id: T-1251
title: "FORMULATEXT keeps an @ group that is only the leftmost part of the operand: nested group, dropped (B1) and corner group with no intersection (follow-up of T-1245)"
milestone: M1
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The engine prints `=@((A1 B1) C1) D1`, `=@(A1):(B1) C1` and `=@(A1):B1` as Excel does (as written).

## Context
probes/at-group-leftmost-shapes E34, E35, E36 (excel-graph, T-1245): Excel prints all three as written; the engine prints `=@(A1 B1 C1) D1`, `=@(A1):B1 C1` and `=@A1:B1`. Pinned in `at_operand_group_leftmost_shapes_match_the_probe`. Code: `lead_group_text`, `LeadGroup` (parser/reference_operators.rs, parser/mod.rs); related T-1244 (inner parentheses).

## Acceptance criteria
- [ ] The three forms print as written; the test asserts Excel's text; the references.md note is updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
