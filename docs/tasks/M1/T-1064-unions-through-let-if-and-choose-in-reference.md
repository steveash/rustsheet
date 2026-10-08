---
id: T-1064
title: "Unions through LET, IF and CHOOSE in reference context (SPEC-0004 step 4)"
milestone: M1
lane: engine
status: in-progress
priority: P2
size: M
tier: opus
spec: docs/specs/SPEC-0004-multi-area-references.md
depends_on: [T-1062]
---

## Goal
A union bound by `LET`, or returned by `IF` or `CHOOSE`, stays a multi-area reference where a reference is accepted (`LET(x,(A1,B2),SUM(x))`, `SUM(IF(TRUE,(A1,B2)))`) and is `#VALUE!` as a value.

## Context
SPEC-0004 step 4 ("Intersection operands", "LET, IF, CHOOSE"). Intersection operands are evaluated through `evaluate_reference_arg` and the code branches on the result (two single-sheet ranges keep the allocation-free pair path; `is_multi_area` is deleted; one `MAX_AREA_PAIRS` budget per top-level expression). `fn_let` (functions/logical/let.rs) evaluates bindings in value context, so a union binding is `#VALUE!`. `variable_stack` (shared with LAMBDA) stores a `Binding` (`Value(CalcResult)` or `Areas(Areas)`, `Rc`-shared); every reader except `evaluate_reference_arg` sees `#VALUE!` for `Areas`. Give CHOOSE the `as_reference` flag IF and INDEX already have, and make `evaluate_reference_arg` recurse into the chosen branch. Also move AGGREGATE's array form (`aggregate_array_form`, functions/aggregate.rs) from its `Node::OpUnionKind` syntax match onto `evaluate_reference_arg`, deferred from T-1062: only a LET variable or a name can hide a union from the syntax match, so it matters from this step on; keep its `Array`, number and error arms and its `#VALUE!` for a 3-D reference (`get_sheet_span_reference`). Evidence: SPEC-0004 probe rows 9 and 10 (cases from T-1061).

## Acceptance criteria
- [ ] Probe rows 9 and 10 pass, including `LET(x,(A1,B2),x A1)` and `IF(TRUE,(A1,B2)) A1`.
- [ ] Unit test: `name A1` and `A1:B2 B1:B3` still take the pair path (no area list).
- [ ] Unit test: nested LET shadowing a union binding; a deep IF chain stays within the parser depth cap.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Defined names (T-1065).

## Progress (not merged; review round 2)
- `Binding` (`Value` | `Areas`, reference_operators.rs) replaces `CalcResult` in `variable_stack`, now a `VariableStack` (functions/logical/let.rs) that caps the areas all bindings in scope hold at `MAX_HELD_AREAS` (4 × `MAX_AREAS`; DEVIATION in references.md "Union area cap"; doubling-chain unit test).
- LET values and LAMBDA arguments bind through one helper, `Model::bind_argument`: whatever `is_multi_area` admits is evaluated with `evaluate_reference_arg`, so a union through another variable, IF, CHOOSE, LET or as a LAMBDA argument stays its areas. LET in reference context evaluates its body there (`let_impl`), so `SUM(LET(x,(A1,B2),x))` = 6 and `SUM(LET(x,K1,x))` ignores text in K1.
- `if_impl`, `choose_impl` and `let_impl` return `RefArg`; `fn_if`, `fn_choose`, `fn_let` and the single-sheet path collapse it with `into_single_area`.
- Intersection operands: `is_multi_area` stays as a syntactic gate, widened to a LET variable, IF, CHOOSE and LET; SPEC-0004 "Intersection operands" and "LET, IF, CHOOSE" amended with the reason. Unit test observes the gate for `name A1` and `A1:B2 B1:B3`.
- New probes (excel-graph, 2026-10-08): probes/multiarea-letbind (16 cells, all pass) and probes/3d-ref-choose-let (CHOOSE hands a 3-D span to the aggregates, 113 and 3, as the branch now does; LET-bound spans C3:C4 fail, filed as T-1158). Recorded in references.md.
- AGGREGATE's array form moved into T-1065's acceptance criteria.
- Fixed the LET duplicate-name early return, which left the frame's earlier bindings in scope.
- Not done: one cell budget per formula for `SUM(x,x,...)` (performance minor, approved): each argument is already bounded by `MAX_AREA_CELLS`, the same as a plain range argument repeated.
- Eval: probes fixed 1 (min-max-reference-forms D8) on top of round 1's +5; new coverage 22 cells, 2 known failures (3d-ref-choose-let C3:C4); no regressions.
