---
id: T-1251
title: "FORMULATEXT keeps an @ group that is only the leftmost part of the operand: nested group, dropped (B1) and corner group with no intersection (follow-up of T-1245)"
milestone: M1
lane: engine
status: in-progress
priority: P2
size: S
tier: opus
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

## Handoff (resumed)
Sonnet run read the code (parser/reference_operators.rs `lead_group_text`, `LeadGroup` in parser/mod.rs) and stopped before implementing: two of the three forms are the general "written parentheses are not kept in the tree" problem of T-1244, a design question.
- `=@((A1 B1) C1) D1`: `LeadGroup` is Copy and holds one (levels, operands) pair, and the chain is flattened left-associative, so the inner `(A1 B1)` group is lost. It needs a nested-group record (or the T-1244 `Group` node).
- `=@(A1):(B1) C1`: the parentheses around the right operand of `:` are not kept anywhere in the tree (same cause).
- `=@(A1):B1`: probably simple. `parse_at` (reference_operators.rs ~327) only keeps `lead` when the child is an `OpIntersectKind`, and `lead_group_text` returns `None` when `operands.len() < 2`; allow a single range operand when `lead.of_range` and the child is an `OpRangeKind`.
Next: decide between extending `LeadGroup` and waiting for the T-1244 spec; flip the three asserts in `at_operand_group_leftmost_shapes_match_the_probe` and update references.md (~line 875).
