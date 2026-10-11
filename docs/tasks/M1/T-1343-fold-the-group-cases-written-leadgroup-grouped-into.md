---
id: T-1343
title: "Fold the @ group cases (written, LeadGroup, grouped) into Group nodes; export SINGLE((x)) (SPEC-0007 step 4)"
milestone: M1
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-1342]
spec: docs/specs/SPEC-0007-written-parentheses.md
---

## Goal
The `@` group cases (`ImplicitIntersection.written`, `lead`/`LeadGroup`, `grouped`) are `Group` nodes, so every written group in an `@` operand prints back, and `@(x)` exports as `_xlfn.SINGLE((x))` like Excel.

## Context
SPEC-0007 Rollout step 4. Removes the parser's `at_paren`/`at_group` state and `LeadGroup`. Evidence: probes/at-operand-nested-groups, probes/at-group-leftmost-shapes E34/E35, probes/at-group-range-corner-whole E34; xlsx-storage.md "Written parentheses are stored as written".

## Acceptance criteria
- [ ] The remaining at-operand-nested-groups cells, at-group-leftmost-shapes E34/E35 and at-group-range-corner-whole E34 pass and leave the baseline.
- [ ] The T-1167 test `at_operand_adds_no_level_for_a_nested_inner_group` (E40, E41, E43, E44) and the F50, F51 asserts of `at_operand_group_leftmost_shapes_match_the_probe` assert Excel's text.
- [ ] Export of `=@(Z1+(Z2))` is `_xlfn.SINGLE((Z1+(Z2)))`; import → print round-trips it.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
