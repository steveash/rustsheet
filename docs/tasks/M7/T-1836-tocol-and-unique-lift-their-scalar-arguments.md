---
id: T-1836
title: "TOCOL and UNIQUE lift their scalar arguments over a range or an array"
milestone: M7
lane: functions-lookup
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-1770, T-1833, T-1834]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
TOCOL's ignore and scan_by_column and UNIQUE's by_col and exactly_once lift over a multi-cell range and an array, one call per element whose value is the top-left of that call's result, as in Excel; the array argument stays whole.

## Context
Split from T-1271. Mechanical once T-1834 lands: add table entries (the array argument `Whole`, the scalar ones as T-1833's per-parameter list says) and kernels using the element-value and cost rules T-1834 built; do not write a new lift or bound. Each element reads the whole array, so charge it the array's cells per element (`LiftCost::Cells`, `lift_call_costed`) on top of `KernelWork::Output { weight: 1 }` (SPEC-0011 "Array-valued functions" rule 3). Put the entries in their own `// --- M7 (T-1836) ---` block in functions/lift/kinds.rs so T-1835 and T-1839 can land in parallel. Delete TOCOL's and UNIQUE's `fallback_signature`/`fallback_shape` arms in expressions/parser/static_analysis.rs.

Code: `fn_tocol` in functions/lookup_and_reference/tocol_torow.rs, `fn_unique` in functions/spill_functions.rs. TOROW is not probed; leave it as it is unless T-1833's list enters it.

Owned cells:
- probes/m7-argument-modes (T-1083, excel-graph, 2026-10-09; arrays.md "Dynamic-array and LAMBDA function argument modes"):
  - F9:H9 `TOCOL(A1:B3,{0;1})`: ROWS 2, COLUMNS 1, `INDEX(..,1,1)` 1 (ours `#N/IMPL!`).
  - K8 `=ROWS(INDEX(TOCOL(A1:B3,{0;1}),1,1))` 1.
  - F12:H12 `UNIQUE(D1:D3,{FALSE;TRUE})`: ROWS 2, COLUMNS 1, INDEX "x" (ours `#N/IMPL!`).
  F54:H54 and K14-K16 (TOCOL's text and numeric ignore) pass today and must keep passing.
- gen-m9 (T-1289, excel-graph, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
  - gen-m9/1782efcecd5d4b1b Calc!A19 `=TOCOL(Data!C8:D11,HSTACK(Data!E5:E6),..)` (ignore over a 2x1 array): Excel `#SPILL!`, ours `#N/IMPL!`.
  - gen-m9/4df1b22d3ed63f58 Calc!A36 `=UNIQUE(Data!A6:D10,INDEX(Data!D4:D10,0))` (by_col over a 7-cell range): Excel `#SPILL!`, ours `#VALUE!`.

TOCOL's scan_by_column and UNIQUE's exactly_once are `Lift` by SPEC-0011's per-parameter list; T-1960 probes them.

## Acceptance criteria
- [ ] probes/m7-argument-modes F9:H9, K8 and F12:H12 pass; F54:H54 and K14-K16 still pass.
- [ ] gen-m9/1782efcecd5d4b1b A19 and gen-m9/4df1b22d3ed63f58 A36 pass, with no new failing gen-m9 cell.
- [ ] Their lines leave eval/baseline/ (`make eval-baseline`), with no new known failures.
- [ ] TOCOL and UNIQUE have table entries citing those cells; their fallback arms are gone (`no_function_in_the_table_has_a_fallback_arm` passes).
- [ ] Unit tests: an unlifted call is unchanged (value and static shape); a lifted call over a large array is charged per element and `#ERROR!` past the visit cap; an element error stays in its position.
- [ ] `make check` green; reviewers approve.

## Out of scope
- SORTBY (T-1839) and SORT (T-1837).
- Any change to the lift driver or the element rules (T-1834); if they do not fit, hand the task up (work-loop 5b).
