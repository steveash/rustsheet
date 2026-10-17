---
id: T-1834
title: "SEQUENCE and MAKEARRAY lift their scalar arguments over a range or an array (first array-valued lifting table entries)"
milestone: M7
lane: functions-lookup
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-1770, T-1833]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
SEQUENCE's rows, columns, start and step and MAKEARRAY's rows and cols lift over a multi-cell range and an array, one call per element whose value is the top-left of that call's result, as in Excel; they are the first array-valued entries of the SPEC-0011 table, with the static-analysis and cost rules T-1833 specifies.

## Context
Split from T-1271. This task builds the shared part of T-1833's "array-valued functions" design (the `ArgKinds`/static-analysis change, the element-value rule and its cost bound) with its first two consumers, so T-1835, T-1836, T-1838 and T-1839 only add entries and kernels. Today SEQUENCE over an array is `#N/IMPL!` (`cast_to_number`) and over a range gives one row (implicit intersection); MAKEARRAY over a range is `#VALUE!`.

Code: `fn_sequence` in crates/rustsheet-engine/src/functions/math_and_trigonometry/sequence.rs, `fn_makearray` in functions/logical/bycol_byrow.rs, the table in functions/lift/kinds.rs (`arg_kinds`, `lifting_table_functions!`), `lift_call`/`lift_call_costed` in functions/lift/call.rs. Delete the SEQUENCE and MAKEARRAY arms of `fallback_signature` and `fallback_shape` in expressions/parser/static_analysis.rs (the table-wins test).

Owned cells, probes/m7-argument-modes (T-1083, excel-graph, 2026-10-09; docs/excel-semantics/arrays.md "Dynamic-array and LAMBDA function argument modes"):
- F1 `=ROWS(SEQUENCE(A1:A3))` 3 (ours 1).
- F2:H2 `SEQUENCE({1;2;3})`: ROWS 3, COLUMNS 1, `INDEX(..,1,1)` 1 (ours `#N/IMPL!`).
- F3:H3 `SEQUENCE(2,2,{1;10})`: ROWS 2, COLUMNS 1, INDEX 1 (ours `#N/IMPL!`).
- F23:H23 `MAKEARRAY(A1:A2,2,LAMBDA(r,c,r+c))`: ROWS 2, COLUMNS 1, INDEX 2 (ours `#VALUE!`).
- K9 `=ROWS(INDEX(SEQUENCE({1;2;3}),3,1))` 1; K10 `=ROWS(INDEX(MAKEARRAY(A1:A2,2,LAMBDA(r,c,r+c)),2,1))` 1.
G1 and H1 pass today and must keep passing; so must F53:H53 (`SEQUENCE(0)` is `#CALC!`) and F48:H48 (`MAKEARRAY(2,2,SUM)`).

Parameters not yet probed lift only where T-1833's per-parameter list marks them `Lift` (it decides which need a probe first).

## Acceptance criteria
- [ ] The probes/m7-argument-modes cells F1, F2:H2, F3:H3, F23:H23, K9 and K10 pass; their lines leave eval/baseline/probes.txt (`make eval-baseline`), and no other cell regresses.
- [ ] SEQUENCE and MAKEARRAY have table entries citing those cells; their `fallback_signature`/`fallback_shape` arms are gone and `no_function_in_the_table_has_a_fallback_arm` passes.
- [ ] Unit tests: an unlifted `SEQUENCE(3)` and `MAKEARRAY(2,2,..)` keep their static shape `Unknown` and their values; the T-1833 cost-rule test formulas (for example a lifted `SEQUENCE` whose elements would each be 10^6 cells, and a lifted MAKEARRAY whose elements each call the LAMBDA 10^6 times) are `#ERROR!` or finish within the stated bound; one element error stays in its position (`SEQUENCE({1;0})` element 2 is `#CALC!`, or what T-1833 records).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other M7 functions (T-1835, T-1836, T-1837, T-1838, T-1839).
- Lifting LAMBDA-taking helpers other than MAKEARRAY (BYROW, MAP, SCAN: T-1511, T-1273).
