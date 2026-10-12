---
id: T-1392
title: "Drop the parser's grouped_root/root_group/leading_paren root bookkeeping"
milestone: M1
lane: engine
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The parser no longer keeps `grouped_root`, `root_group` or `leading_paren`; the root `Group` comes from the same rule as every other group.

## Context
T-1341 (SPEC-0007 step 2) replaced `GroupedRootKind` by a root `Group` but kept this state, because plain `parse` (defined-name bodies) strips the root group and `parse_cell_formula` reads `grouped_root`. Decide whether name bodies can keep their root `Group` (defined_names.rs, reference_operators.rs `range_operand`, probes/defined-name-grouped-reference, near-zero-snap-name-bodies) and then remove the state. Code: expressions/parser/reference_operators.rs `parse_cell_formula`/`parse_parenthesized`, mod.rs.

## Acceptance criteria
- [ ] The three fields are gone, text and values unchanged, 0 eval regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Removed `leading_paren`, `grouped_root` and `root_group` and `Parser::parse_cell_formula`. `parse_parenthesized` now always returns its `Group` (nesting when its content is itself a `Group`), so `parse` and the cell-formula callers share one rule. Defined-name bodies keep their root `Group`: no eval case changed (0 regressions, no baseline change). Tests that asserted the stripped shape were updated.
Side effect: `( (A1) )` (spaces between nested parentheses around a whole formula) now prints `((A1))` instead of `(A1)`, as it already did inside an argument; values unchanged, Excel's exact print unprobed.
