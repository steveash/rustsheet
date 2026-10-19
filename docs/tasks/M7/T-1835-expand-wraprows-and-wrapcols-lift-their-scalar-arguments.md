---
id: T-1835
title: "EXPAND, WRAPROWS and WRAPCOLS lift their scalar arguments over a range or an array"
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
EXPAND's rows, columns and pad_with and WRAPROWS's and WRAPCOLS's wrap_count and pad_with lift over a multi-cell range and an array, one call per element whose value is the top-left of that call's result, as in Excel; the array argument stays whole.

## Context
Split from T-1271. Mechanical once T-1834 lands: add table entries (the array argument `Whole`, the scalar ones as T-1833's per-parameter list says) and kernels using the element-value and cost rules T-1834 built; do not write a new lift or bound. Put the entries in their own `// --- M7 (T-1835) ---` block in functions/lift/kinds.rs, as the engineering and math blocks are, so T-1836 and T-1839 can land in parallel. Delete the three functions' `fallback_signature`/`fallback_shape` arms in expressions/parser/static_analysis.rs.

Code: `fn_expand` in functions/lookup_and_reference/expand.rs, `fn_wrapcols`/`fn_wraprows` in functions/lookup_and_reference/wrapcols_wraprows.rs. T-1786 (done) already bounds one call's output with `check_array_size`; the kernel must keep that check and the function's argument-error order.

Owned cells:
- probes/m7-argument-modes (T-1083, excel-graph, 2026-10-09; arrays.md "Dynamic-array and LAMBDA function argument modes"):
  - F7:H7 `EXPAND(A1:A2,{3;4})`: ROWS 2, COLUMNS 1, `INDEX(..,1,1)` 1 (ours `#N/IMPL!`).
  - F8:H8 `WRAPROWS(A1:A3,{1;2})`: ROWS 2, COLUMNS 1, INDEX 1 (ours `#N/IMPL!`).
  - G42 `=COLUMNS(WRAPCOLS(A1:A3,2,{0;1}))` 1 (ours 2: pad_with is not lifted). F42 and H42 pass today and must keep passing.
  - K5 `=ROWS(INDEX(EXPAND(A1:A2,{3;4}),1,1))` 1; K6 `=ROWS(INDEX(WRAPROWS(A1:A3,{1;2}),1,1))` 1; K7 `=COLUMNS(INDEX(WRAPROWS(A1:A3,{1;2}),2,1))` 1.
- gen-m7/6db767d720c5b55a Calc!A12 `=WRAPCOLS(Data!D3:D3,UNIQUE(Data!B3:E4,SWITCH(..)))` (wrap_count is UNIQUE's 2-D array): Excel `#SPILL!`, ours `#N/IMPL!` (T-1275, excel-graph, 2026-10-11; `./target/release/rustsheet-eval show gen-m7/6db767d720c5b55a.case.json`).

EXPAND's columns and pad_with and WRAPROWS's pad_with are `Lift` by SPEC-0011's per-parameter list ("Array-valued functions"); T-1960 probes them. `KernelWork::Output { weight: 1 }`.

## Acceptance criteria
- [ ] probes/m7-argument-modes F7:H7, F8:H8, G42 and K5-K7 pass, and F42, H42 still pass.
- [ ] gen-m7/6db767d720c5b55a A12 passes, with no new failing gen-m7 cell.
- [ ] Their lines leave eval/baseline/ (`make eval-baseline`), with no new known failures.
- [ ] The three functions have table entries citing those cells; their fallback arms are gone (`no_function_in_the_table_has_a_fallback_arm` passes).
- [ ] Unit tests: an unlifted call is unchanged (value and static shape); `=ROWS(EXPAND(1,SEQUENCE(1024),1024))` is `#ERROR!` (SPEC-0011 rule 3); an element error stays in its position.
- [ ] `make check` green; reviewers approve.

## Out of scope
- WRAPCOLS of a single value and wrap_count below 1 (T-1348).
- Any change to the lift driver or the element rules (T-1834); if they do not fit, hand the task up (work-loop 5b).
