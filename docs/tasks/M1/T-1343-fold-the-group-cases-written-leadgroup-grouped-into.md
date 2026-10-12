---
id: T-1343
title: "Fold the @ group cases (written, LeadGroup, grouped) into Group nodes; export SINGLE((x)) (SPEC-0007 step 4)"
milestone: M1
lane: engine
status: done
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
- [x] The remaining at-operand-nested-groups cells, at-group-leftmost-shapes E34/E35 and at-group-range-corner-whole E34 pass and leave the baseline.
- [x] The T-1167 test `at_operand_adds_no_level_for_a_nested_inner_group` (E40, E41, E43, E44) and the F50, F51 asserts of `at_operand_group_leftmost_shapes_match_the_probe` assert Excel's text.
- [x] Export of `=@(Z1+(Z2))` is `_xlfn.SINGLE((Z1+(Z2)))`; import → print round-trips it.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- `ImplicitIntersection` loses `written`, `lead` and `grouped`; `LeadGroup` and the parser's `at_paren`/`at_group`
  state are gone. Every written group around, inside or at the start of an `@` operand, and around a whole `@`, is now a
  `Node::Group` where it was written (`parse_parenthesized` always regroups). `at_text` prints `@` + the child; only a
  nested `@` with no written group gets one level. Net -160 lines.
- Shape-matching sites that look at an `@`'s child now peel groups: `is_at_if`, the `@INDEX` arm of
  `eval_ref_or_index`, and the `#` error arm in model.rs (`ANCHORARRAY((@A1:B2))`).
- Export: `=@(Z1+(Z2))` → `_xlfn.SINGLE((Z1+(Z2)))`; a new test checks the whole SPEC-0007 storage table,
  export → import → print.
- T-1167 test (E40, E41, E43, E44) and the F50/F51 asserts now assert Excel's text; `A1 (@A1 B1)` keeps its group.
- Eval: probes known failures 2890 → 2885 (at-operand-nested-groups F11, F13, F14; at-group-leftmost-shapes E34;
  at-power-text I46, the `=(@-2)^2` group). 0 regressions. at-group-leftmost-shapes E35 and at-group-range-corner-whole
  E34 already passed on main.
- Not probed (UNVERIFIED, xlsx-storage.md): Excel's stored form of a redundant `@` with a written group
  (`=@(A1:A3)` at the root); the engine exports `_xlfn.SINGLE((A1:A3))`, which round-trips.
