---
id: T-1839
title: "SORTBY lifts its sort_order arguments over a range or an array (alternating by_array/sort_order kinds in the lifting table)"
milestone: M7
lane: functions-lookup
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-1770, T-1833, T-1834]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
SORTBY's sort_order arguments lift over a multi-cell range and an array, one call per element whose value is the top-left of that call's result, while its array and by_array arguments stay whole, as in Excel; the lifting table can describe SORTBY's alternating (by_array, sort_order) pairs as T-1833 specifies.

## Context
Split from T-1271. `ArgKinds` has a single `repeat` kind, and SORTBY's arguments after the first alternate `Whole` by_array and `Lift` sort_order; T-1833 chooses the representation, this task builds it (functions/lift/kinds.rs, and `lifted_static_result`/`get_function_args_signature` where they read it) with SORTBY as its consumer. Each element sorts the whole array, so charge it the array's and by_arrays' cells per element (`lift_call_costed`) as T-1833 specifies. Put the entry in its own `// --- M7 (T-1839) ---` block so T-1835 and T-1836 can land in parallel. Delete SORTBY's `fallback_signature`/`fallback_shape` arms in expressions/parser/static_analysis.rs.

Code: `fn_sortby` in functions/spill_functions.rs.

Owned cells, probes/m7-argument-modes (T-1083, excel-graph, 2026-10-09; arrays.md "Dynamic-array and LAMBDA function argument modes"):
- F26:H26 `SORTBY(A1:A3,B1:B3,{1;-1})`: ROWS 2, COLUMNS 1, `INDEX(..,1,1)` 1 (ours `#N/IMPL!`).
F56:H56 (`SORTBY(A1:A3,B1:B2)` is `#VALUE!`) passes today and must keep passing.

## Acceptance criteria
- [ ] probes/m7-argument-modes F26:H26 pass and F56:H56 still pass; their lines leave eval/baseline/probes.txt (`make eval-baseline`), with no new known failures.
- [ ] SORTBY has a table entry citing those cells; its fallback arms are gone (`no_function_in_the_table_has_a_fallback_arm` passes); a unit test checks the kinds of arguments 1-6 and that the accepted argument counts are those `fn_sortby` accepts today (2, 3, then odd counts from 5; an even count above 3 is the count error).
- [ ] Unit tests: an unlifted SORTBY is unchanged (value and static shape); a lifted sort_order2 in a two-key call lifts; a by_array given as an array literal is not lifted; a lifted call over a large array is `#ERROR!` past the visit cap.
- [ ] `make check` green; reviewers approve.

## Out of scope
- SORT (T-1837); SORT's and SORTBY's argument-error order (T-1351).
