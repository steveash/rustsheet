---
id: T-1064
title: "Unions through LET, IF and CHOOSE in reference context (SPEC-0004 step 4)"
milestone: M1
lane: engine
status: done
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

## Outcome
- `Binding` (`Value` | `Areas`) replaces `CalcResult` in `variable_stack`; every reader but `Model::variable_reference` (reference context) sees `#VALUE!` for an `Areas` binding. LET binds areas when its value is a union or a multi-area intersection.
- `if_impl` and the new `choose_impl` return `RefArg`; in reference context (`evaluate_reference_arg`) the branch a scalar condition or index picks is itself evaluated in reference context, so a union survives. `fn_if`, `fn_choose` and the single-sheet path collapse it with `into_single_area`.
- Intersection operands: `is_multi_area` now also treats a LET variable, IF and CHOOSE as possibly several areas, and `evaluate_areas` reads them through the evaluator; plain references and defined names keep the allocation-free pair path. Deviation from the plan: `is_multi_area` stays (extended) instead of branching on evaluated results, which would allocate for every name operand; the pair budget is unchanged (a function operand starts its own, as before).
- Not done here: AGGREGATE's array form still matches `Node::OpUnionKind` (it now only misses a union behind a LET variable or IF/CHOOSE). Moved to T-1065, where names add the remaining hidden case.
- Eval: probes +5 (multiarea-let H1:H2, multiarea-ifchoose H1:H3), no regressions.
