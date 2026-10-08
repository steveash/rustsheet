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

## Progress (not merged; review round 1 open)
- `Binding` (`Value` | `Areas`) replaces `CalcResult` in `variable_stack`; every reader but `Model::variable_reference` (reference context) sees `#VALUE!` for an `Areas` binding. LET binds areas when its value is a union or a multi-area intersection.
- `if_impl` and the new `choose_impl` return `RefArg`; in reference context (`evaluate_reference_arg`) the branch a scalar condition or index picks is itself evaluated in reference context, so a union survives. `fn_if`, `fn_choose` and the single-sheet path collapse it with `into_single_area`.
- Intersection operands: `is_multi_area` now also treats a LET variable, IF and CHOOSE as possibly several areas, and `evaluate_areas` reads them through the evaluator; plain references and defined names keep the allocation-free pair path. Deviation from the plan: `is_multi_area` stays (extended) instead of branching on evaluated results, which would allocate for every name operand; the pair budget is unchanged (a function operand starts its own, as before).
- Not done here: AGGREGATE's array form still matches `Node::OpUnionKind` (it now only misses a union behind a LET variable or IF/CHOOSE). Moved to T-1065, where names add the remaining hidden case.
- Eval: probes +5 (multiarea-let H1:H2, multiarea-ifchoose H1:H3), no regressions.

## Open review findings (round 1, 2026-10-08)
Security and architecture asked for changes; quality approved. Excel fidelity and performance had not reported when the session budget ran out. Still to do before merge:
1. Security (MAJOR): `Binding::Areas` values held in `variable_stack` have no total cap. A chain of doubling LET bindings (`x2,(x1,x1)`, ...) can hold many `MAX_AREAS` lists at once, so memory grows with formula length. Keep a running total of areas held by `Binding::Areas` (reset with `clear_variable_stack`, decremented when a LET frame ends). Past a cap, give `#VALUE!` and add it to the references.md "Union area cap" DEVIATION. Add a doubling-chain regression test, and avoid the `areas.0.to_vec()` copy for a variable operand in `evaluate_areas` where possible.
2. Architecture (MAJOR): SPEC-0004 "Intersection operands" says to branch on `evaluate_reference_arg` results and delete `is_multi_area`. This branch instead extends `is_multi_area` syntactically (LET variable, IF, CHOOSE). Either implement the spec, or amend SPEC-0004 with the reason (a defined-name operand would allocate) and the replacement rule.
3. Architecture (MAJOR): add a test that observes the pair path for `name A1` and `A1:B2 B1:B3`, not only their values (for example by unit-testing `is_multi_area`).
4. Architecture (MAJOR): AGGREGATE's array form is still on its `Node::OpUnionKind` syntax match. Migrate it here, or move it explicitly into T-1065's acceptance criteria. The T-1065 note added on this branch is a start.
5. Minor: LET binds areas only for a syntactic union or intersection, so `LET(y,x,SUM(y))` with `x` bound to a union is `#VALUE!`. Route `NamedVariableKind` values through `variable_reference`. Move `Binding` next to `RefArg` in reference_operators.rs (now `Arc`-backed so `Model` stays `Send`). Re-wrap the long references.md line.
6. Performance (APPROVE, minor): a LET-bound union is reused at no cost per use, so `SUM(x,x,...)` multiplies the per-argument `MAX_AREA_CELLS` walk. Consider one cell budget shared by the formula (like `MAX_AREA_PAIRS`), keeping the caller's pair budget for IF/CHOOSE operands in `evaluate_areas`, and keeping the pair path for a single-range variable.
7. Excel fidelity (REQUEST_CHANGES):
   - (MAJOR) CHOOSE in reference context now hands a 3-D span to the aggregates. Compared with main: `SUM(CHOOSE(1,Data1:Data3!A1:A2))` goes from `#VALUE!` to 113, and `COUNT(CHOOSE(1,Data1:Data3!A1))` from 0 to 3. No probe covers this; the closest evidence is `SUM(IF(TRUE,3-D))` = 113 (probes/3d-ref-functions C22). Probe it (excel-probe), record the result in references.md, and add a unit test.
   - (MAJOR) LET should bind through `evaluate_reference_arg`, as SPEC-0004 says. `LET(x,(A1,B2),y,x,SUM(y))` and `LET(x,IF(TRUE,(A1,B2)),SUM(x))` are `#VALUE!` on this branch, and Excel very likely gives 6. Probe both. Also remove the "engine choice" sentence from the VERIFIED references.md entry, or move it to an UNVERIFIED entry.
   - (MINOR) New unprobed results: `SUM((IF(TRUE,(A1,B2)),C3))` = 13, `IF(FALSE,1,(A1,B2))`, `CHOOSE(1.9,..)` and a nested IF/CHOOSE as union operands. Probe them together with LET shadowing (H7/H8) and `LAMBDA(x,SUM(x))((A1,B2))`.
