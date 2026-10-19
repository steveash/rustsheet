---
id: T-1833
title: "SPEC-0011 amendment: array-valued M7 functions in the lifting table, and TAKE/DROP elements that are whole sub-arrays"
milestone: M7
lane: functions-lookup
status: in-progress
priority: P2
size: M
tier: opus
depends_on: [T-1770]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
SPEC-0011 says how a function whose unlifted result is already an array (SEQUENCE, MAKEARRAY, EXPAND, WRAPROWS, WRAPCOLS, TOCOL, UNIQUE, SORTBY, TAKE, DROP) enters the lifting table, and how a lifted element that is a whole sub-array (TAKE, DROP) is held, so that T-1834, T-1835, T-1836, T-1839 and T-1838 implement one stated design instead of each inventing one.

## Context
Split from T-1271 (SPEC-0011 rollout step 3). The spec leaves these open: "Lifts whose elements are arrays or references" is a non-goal that "T-1271 and T-1241 settle", and open question 5 is still open. Reading the code (2026-10-17) shows four design gaps that every M7 entry hits:
1. **Static analysis.** Every function in the table today returns a value, and `lifted_static_result` (expressions/parser/static_analysis/lifted.rs) gives an unlifted call the shape `Scalar`. The M7 functions are `StaticResult::Unknown` today (static_analysis.rs `fallback_shape` arms near lines 1040-1069) with hand-written signatures (`args_signature_sequence`, `_expand`, `_wrapcols`, `_sortby`, `_unique`, `_take`, `_tocol`, near lines 513-554). The test `no_function_in_the_table_has_a_fallback_arm` forbids keeping those arms for an entered function, so `ArgKinds` (or the derivation from it) must be able to say "the unlifted result is an array".
2. **The element value.** For EXPAND, WRAPROWS, WRAPCOLS, TOCOL, SEQUENCE, MAKEARRAY, UNIQUE and SORTBY each element is one value, the top-left of that element's own call: probes/m7-argument-modes K5-K10 (ROWS/COLUMNS of `INDEX(<lifted call>,i,1)` are 1), H2, H3, H7, H8, H9, H12, H23, H26, while `EXPAND(A1:A2,3)` alone is 3x1 (docs/excel-semantics/arrays.md "Dynamic-array and LAMBDA function argument modes"). `calc_result_to_array_node` (cast.rs) turns an `Array` into `#VALUE!` today, so a kernel cannot simply return the scalar call's result. `functions/util.rs` `top_left_of_array` exists.
3. **The element cost (directive 5).** An element's own result can be far larger than its inputs: `SEQUENCE({1E6;1E6})`, `EXPAND(A1:A2,{1E6;1E6},1E4)`, `MAKEARRAY({1000;1000},1000,LAMBDA(..))` (a LAMBDA call per cell). Charging `LiftCost::Unit` would let 2^20 elements each build up to the array limit. The spec must pick one rule: kernels that compute only the top-left after the function's own argument checks (same error order as the scalar call), or a charged `KernelWork`/`LiftCost`, with a test formula that must be `#ERROR!` or fast.
4. **SORTBY's arguments alternate** `by_array` (`Whole`) and `sort_order` (`Lift`), with an even count above 3 refused (`fn_sortby`); `ArgKinds` has one `repeat` kind.
5. **Whole sub-array elements (TAKE, DROP).** A lifted TAKE or DROP element is the call's whole result: `INDEX(TAKE(A1:B3,{1;2}),1,1)` is the 1x2 row {1,10} (K1 ROWS 1, K2 COLUMNS 2; H5/I5 spill 1, 10; H4/I4 the same over the range A1:A2), `INDEX(DROP(A1:B3,{1;2}),1,1)` is the 2x2 block {2,20;3,30} (K3, K4 are 2; H6 is `#SPILL!` because it collides). The outer result is 2x1 (F4-F6, G4-G6). `ArrayNode` cannot hold an array. Design it on top of `lift_values`/`lift_grids` and `cast::calc_result_to_array_node`, not as a new copy of the broadcast (directive 3), and say what the consumers of such an array do with a nested element (INDEX returns it; ROWS/COLUMNS count the outer array; what a spill, SUM, `=` comparison or arithmetic see). The last are not probed; list them as open questions and file a probe task (milestone M7, research lane) if the design needs the answer.

Also decide which unprobed scalar parameters may be entered as `Lift` on the spec's rule "Worksheet functions also lift over a multi-cell range ... the M7 scalars", and which need a probe first: probed are SEQUENCE rows and start, TAKE/DROP rows, EXPAND rows, MAKEARRAY rows, WRAPROWS wrap_count, WRAPCOLS pad_with (row 42), TOCOL ignore, UNIQUE by_col, SORTBY sort_order1, and over a range or an array in gen-m7/6db767d720c5b55a A12 (WRAPCOLS wrap_count) and gen-m9/4df1b22d3ed63f58 A36 (UNIQUE by_col over a range). Not probed: SEQUENCE columns and step, MAKEARRAY cols, EXPAND columns and pad_with, WRAPROWS pad_with, WRAPCOLS wrap_count over an array literal, TOCOL scan_by_column, UNIQUE exactly_once, TAKE/DROP columns, MAKEARRAY's LAMBDA (presumably `Whole` or `WholeByCaller`).

This task owns no eval cell. SORT is not in scope: its sort_index and sort_order are `Whole` lists (T-1837), and nothing of SORT lifts in the probes.

## Acceptance criteria
- [ ] docs/specs/SPEC-0011-scalar-argument-lifting.md has a section for array-valued functions that answers gaps 1-4 above (the `ArgKinds`/static-analysis change with its signature and shape rules, the element-value rule citing K5-K10, the cost rule with its test formulas, the alternating-kind representation) and one for whole sub-array elements answering gap 5, each rule citing the probe cells listed in Context.
- [ ] The spec's per-parameter list for the ten functions above gives each parameter a kind, marking which rest on a probe cell and which on the general rule; any parameter that needs a probe first has a filed task (milestone M7, not M1) named in the spec.
- [ ] SPEC-0011's Tasks list, rollout step 3, the non-goal bullet and open question 5 name T-1834, T-1835, T-1836, T-1837, T-1838 and T-1839 instead of T-1271.
- [ ] If the design changes what those tasks must do, their Context and Acceptance criteria are updated in the same PR (sizes stay S/M; split further if not).
- [ ] No code changes; `make check` green; reviewers approve.

## Out of scope
- Any implementation (T-1834 onward).
- OFFSET, INDIRECT and TRIMRANGE reference-valued elements (T-1810).
- SORT's argument-error order (T-1351); WRAPCOLS of a single value and wrap_count below 1 (T-1348).
