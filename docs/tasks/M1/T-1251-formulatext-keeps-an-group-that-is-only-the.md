---
id: T-1251
title: "FORMULATEXT keeps an @ group that is the left corner of a whole-operand range, =@(A1):B1 (follow-up of T-1245; E34, E35 moved to T-1244)"
milestone: M1
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The engine prints `=@(A1):B1` as Excel does (as written); `=@((A1 B1) C1) D1` and `=@(A1):(B1) C1` were moved to T-1244 (Outcome).

## Context
probes/at-group-leftmost-shapes E34, E35, E36 (excel-graph, T-1245): Excel prints all three as written; the engine prints `=@(A1 B1 C1) D1`, `=@(A1):B1 C1` and `=@A1:B1`. Pinned in `at_operand_group_leftmost_shapes_match_the_probe`. Code: `lead_group_text`, `LeadGroup` (parser/reference_operators.rs, parser/mod.rs); related T-1244 (inner parentheses).

## Acceptance criteria
- [x] `=@(A1):B1` (E36) prints as written; the test asserts Excel's text; the references.md note is updated.
- [x] `=@((A1 B1) C1) D1` (E34) and `=@(A1):(B1) C1` (E35) moved to T-1244 (see Outcome); their asserts still pin the engine and cite T-1244.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Handoff (resumed)
Sonnet run read the code (parser/reference_operators.rs `lead_group_text`, `LeadGroup` in parser/mod.rs) and stopped before implementing: two of the three forms are the general "written parentheses are not kept in the tree" problem of T-1244, a design question.
- `=@((A1 B1) C1) D1`: `LeadGroup` is Copy and holds one (levels, operands) pair, and the chain is flattened left-associative, so the inner `(A1 B1)` group is lost. It needs a nested-group record (or the T-1244 `Group` node).
- `=@(A1):(B1) C1`: the parentheses around the right operand of `:` are not kept anywhere in the tree (same cause).
- `=@(A1):B1`: probably simple. `parse_at` (reference_operators.rs ~327) only keeps `lead` when the child is an `OpIntersectKind`, and `lead_group_text` returns `None` when `operands.len() < 2`; allow a single range operand when `lead.of_range` and the child is an `OpRangeKind`.
Next: decide between extending `LeadGroup` and waiting for the T-1244 spec; flip the three asserts in `at_operand_group_leftmost_shapes_match_the_probe` and update references.md (~line 875).

## Outcome
Rescoped on resume (opus): E36 shipped; E34 and E35 moved to the T-1244 written-parentheses spec. Both lose a group that is
not the leading one (nested inside it, or around a range's right corner); `LeadGroup` records one group, and a second record
would be one more special case for T-1244 to fold away (directive 4).
- `parse_at` keeps the `@` group as `lead` when it is the left corner of a range that is the whole operand (`@(A1):B1`), and
  `lead_group_text` prints a one-operand `of_range` chain. New probe probes/at-group-range-corner-whole (excel-graph): Excel prints `=@(A1:B1):C1`,
  `=@((A1)):B1:C1`, `=@((A1)):B1`, `=@(A1):B1:C1` and `=@(A1):(B1)` as written; the engine now matches the first four (on main
  it dropped the group), and E34 `=@(A1):(B1)` is a T-1244 known failure.
- T-1244 now lists E34, E35 and this `of_range` case among what it folds in; references.md updated.
Eval delta: probes +1 case (10 cells, 9 pass); known failures 561 -> 561 (fixed at-group-leftmost-shapes E36, new-coverage at-group-range-corner-whole E34 for T-1244), regressions 0.
Follow-ups: none new (T-1244 extended).
