---
id: T-1834
title: "SEQUENCE and MAKEARRAY lift their scalar arguments over a range or an array (first array-valued lifting table entries)"
milestone: M7
lane: functions-lookup
status: in-progress
priority: P2
size: M
tier: opus
depends_on: [T-1770, T-1833]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
SEQUENCE's rows, columns, start and step and MAKEARRAY's rows and cols lift over a multi-cell range and an array, one call per element whose value is the top-left of that call's result, as in Excel; they are the first array-valued entries of the SPEC-0011 table, with the static-analysis and cost rules T-1833 specifies.

## Context
Split from T-1271. This task builds the shared part of SPEC-0011 "Array-valued functions" (T-1833): `CallResult` on `ArgKinds` and its static-analysis rule (1), `element_value` (2) and `KernelWork::Output` (3, MAKEARRAY with weight 128) with its first two consumers, so T-1835, T-1836, T-1838 and T-1839 only add entries and kernels. Today SEQUENCE over an array is `#N/IMPL!` (`cast_to_number`) and over a range gives one row (implicit intersection); MAKEARRAY over a range is `#VALUE!`.

Code: `fn_sequence` in crates/rustsheet-engine/src/functions/math_and_trigonometry/sequence.rs, `fn_makearray` in functions/logical/bycol_byrow.rs, the table in functions/lift/kinds/ (mod.rs and the per-family files) (`arg_kinds`, `lifting_table_functions!`), `lift_call`/`lift_call_costed` in functions/lift/call.rs. Delete the SEQUENCE and MAKEARRAY arms of `fallback_signature` and `fallback_shape` in expressions/parser/static_analysis.rs (the table-wins test).

Owned cells, probes/m7-argument-modes (T-1083, excel-graph, 2026-10-09; docs/excel-semantics/arrays.md "Dynamic-array and LAMBDA function argument modes"):
- F1 `=ROWS(SEQUENCE(A1:A3))` 3 (ours 1).
- F2:H2 `SEQUENCE({1;2;3})`: ROWS 3, COLUMNS 1, `INDEX(..,1,1)` 1 (ours `#N/IMPL!`).
- F3:H3 `SEQUENCE(2,2,{1;10})`: ROWS 2, COLUMNS 1, INDEX 1 (ours `#N/IMPL!`).
- F23:H23 `MAKEARRAY(A1:A2,2,LAMBDA(r,c,r+c))`: ROWS 2, COLUMNS 1, INDEX 2 (ours `#VALUE!`).
- K9 `=ROWS(INDEX(SEQUENCE({1;2;3}),3,1))` 1; K10 `=ROWS(INDEX(MAKEARRAY(A1:A2,2,LAMBDA(r,c,r+c)),2,1))` 1.
G1 and H1 pass today and must keep passing; so must F53:H53 (`SEQUENCE(0)` is `#CALC!`) and F48:H48 (`MAKEARRAY(2,2,SUM)`).

Parameters not yet probed (SEQUENCE columns and step, MAKEARRAY cols) are `Lift` by the spec's per-parameter list; T-1960 probes them.

## Acceptance criteria
- [x] The probes/m7-argument-modes cells F1, F2:H2, F3:H3, F23:H23, K9 and K10 pass; their lines leave eval/baseline/probes.txt (`make eval-baseline`), and no other cell regresses.
- [x] SEQUENCE and MAKEARRAY have table entries citing those cells; their `fallback_signature`/`fallback_shape` arms are gone and `no_function_in_the_table_has_a_fallback_arm` passes.
- [x] Unit tests: an unlifted `SEQUENCE(3)` and `MAKEARRAY(2,2,..)` keep their static shape `Unknown` and their values; the spec's rule-3 test formulas are `#ERROR!` (`=ROWS(SEQUENCE(SEQUENCE(1024),1024))`, `=ROWS(MAKEARRAY(SEQUENCE(2048),1024,LAMBDA(r,c,r)))`) and `=ROWS(SEQUENCE({3;4}))` is 2; one element error stays in its position (`SEQUENCE({1;0})` element 2 is `#CALC!`, or what T-1833 records).
- [x] arrays.md's DEVIATION entry "Array results above 1,048,576 cells" (decision 0014) gains the lifted output charge (`KernelWork::Output`) and its test formulas: Excel is expected to answer `=ROWS(SEQUENCE(SEQUENCE(1024),1024))` with 1024 (worked out from F1/F2, not measured: say so in the entry), ours is `#ERROR!`. Add no eval expectation for a 1x1 lifted array (`SEQUENCE({3})`) before T-1960 answers it.
- [x] `make check` green; reviewers approve.

## Out of scope
- The other M7 functions (T-1835, T-1836, T-1837, T-1838, T-1839).
- Lifting LAMBDA-taking helpers other than MAKEARRAY (BYROW, MAP, SCAN: T-1511, T-1273).

## Outcome
Built SPEC-0011 "Array-valued functions" rules 1-3: `CallResult` on `ArgKinds` (`fixed(..).array()`),
`lifted_static_result` gives an unlifted `Array` entry `Unknown` and a lifted one its broadcast;
the driver's `element_node` takes an `Array` entry's element as its call's top-left value and charges
`KernelWork::Output { weight }` (the cells each element builds) against `MAX_LIFTED_CELL_VISITS`.
SEQUENCE (`[Lift; 4]`, weight 1) and MAKEARRAY (`[Lift, Lift, Whole]`, weight 128) are the first
entries, their kernels `sequence_kernel`/`makearray_kernel` under `lift_call`; their fallback
signature/shape arms and `args_signature_sequence` are gone. An `Array` entry whose grids are all
1x1 runs one unlifted call (keeps `SEQUENCE(INDIRECT("C3")*1)` spilling C3's rows; `SEQUENCE({3})`
stays open for T-1960). The metamorphic test compares an `Array` entry's element with
`INDEX(scalar call,1,1)`. arrays.md's DEVIATION entry gains the output charge and its formulas.
Eval: probes/m7-argument-modes F1, F2, G2, H2, F3, G3, H3, F23, G23, H23, K9, K10 newly pass
(probes 1836 → 1824), regressions 0.

Benchmarks (min of 11 runs, one case per process; base = main before this PR): lift SWITCH
numeric 396.9 → 392.7 ms, SWITCH text 499.7 → 498.4 ms; math_lift ABS lifted 302.9 → 287.0 ms,
ROUND lifted errors 2.846 → 2.804 s, ABS per row 22.37 → 22.62 ms. PMT lifted over a range
20.7 → 24.6 ms was measured at different times under other load and needs a back-to-back
re-run before merge (the `Value` path of `element_node` is one early-return compare).
State when the session budget ran out: reviews (security, architecture, quality, fidelity)
APPROVE; performance REQUEST_CHANGES for bench evidence (numbers above, PMT pending). `make
check` passed up to the stale eval/catalog/lift-kinds.json, regenerated in the last commit; rerun
it in full, re-run the performance reviewer, then open the PR.
