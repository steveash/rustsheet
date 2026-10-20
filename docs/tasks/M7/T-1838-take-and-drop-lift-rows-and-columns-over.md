---
id: T-1838
title: "TAKE and DROP lift rows and columns over a range or an array, each element the whole sub-array"
milestone: M7
lane: functions-lookup
status: done
priority: P2
size: M
tier: opus
depends_on: [T-1770, T-1833, T-1834, T-1955, T-1956]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
TAKE's and DROP's rows and columns lift over a multi-cell range and an array, and each element of the result is that element's whole TAKE or DROP result (a reference to it when the array is a reference), as in Excel, implemented as T-1833's "whole sub-array elements" section of SPEC-0011 specifies.

## Context
Carrier: docs/specs/SPEC-0014-reference-array-results.md `RefGrid` (Rollout step 3). Split from T-1271. The design is SPEC-0011 "Elements that are references" (T-1833): a lifted TAKE or DROP element over a reference is the reference the scalar call returns (A1:B1, A1:B2 for `TAKE(A1:B3,{1;2})`), held in the array-of-references carrier T-1955 specifies and T-1956 builds for OFFSET; over an array value it is the top-left value (rule 2), pending T-1960. It builds on T-1834's array-valued table entries (`element_value` hands a multi-cell reference to the carrier). Do not write a second broadcast (directive 3). Delete TAKE's and DROP's `fallback_signature`/`fallback_shape` arms in expressions/parser/static_analysis.rs.

Code: `fn_take`/`fn_drop` in functions/lookup_and_reference/drop_take.rs; INDEX (functions/lookup_and_reference/index.rs) must return a nested element whole; ROWS/COLUMNS count the outer array.

Owned cells, probes/m7-argument-modes (T-1083, excel-graph, 2026-10-09; arrays.md "Dynamic-array and LAMBDA function argument modes"):
- F4:H4 `TAKE(A1:B3,A1:A2)`: ROWS 2, COLUMNS 1, `INDEX(..,1,1)` 1, spilling 10 into I4 (ours `#VALUE!`; I4 empty).
- F5:H5 `TAKE(A1:B3,{1;2})`: ROWS 2, COLUMNS 1, INDEX 1, spilling 10 into I5 (ours `#N/IMPL!`; I5 empty).
- F6:H6 `DROP(A1:B3,{1;2})`: ROWS 2, COLUMNS 1, INDEX `#SPILL!` (the 2x2 element collides with row 7) (ours `#N/IMPL!`).
- K1 `=ROWS(INDEX(TAKE(A1:B3,{1;2}),1,1))` 1; K2 `=COLUMNS(INDEX(TAKE(A1:B3,{1;2}),1,1))` 2.
- K3 `=ROWS(INDEX(DROP(A1:B3,{1;2}),1,1))` 2; K4 `=COLUMNS(INDEX(DROP(A1:B3,{1;2}),1,1))` 2.

INDEX, ROWS and COLUMNS learn the carrier in T-1956; this task only adds TAKE and DROP as producers. If T-1960 finds that a TAKE over an array value keeps a whole sub-array, split that part off before implementing it.

## Acceptance criteria
- [x] probes/m7-argument-modes F4:H4, I4, F5:H5, I5, F6:H6 and K1-K4 pass; their lines leave eval/baseline/probes.txt (`make eval-baseline`), with no new known failures.
- [x] TAKE and DROP have table entries citing those cells; their fallback arms are gone (`no_function_in_the_table_has_a_fallback_arm` passes).
- [x] Unit tests: unlifted TAKE/DROP unchanged (value and static shape); the reference-element consumers T-1955 lists behave as it specifies; a lifted call whose sub-arrays add up past the array limit is `#ERROR!` before it builds them (directive 5).
- [x] `make check` green; reviewers approve.

## Out of scope
- Reference-valued elements (OFFSET, INDIRECT, TRIMRANGE: T-1810).
- Any consumer behavior T-1833 leaves as an open question: file it as a task instead.

## Outcome
TAKE and DROP join OFFSET, INDIRECT and TRIMRANGE as producers of arrays of references
(`Model::take_drop_reference`, `reference_producer`): when rows or cols is an array or a multi-cell range, a
single-sheet range source lifts through `lift_references` (each element the sub-range its call returns) and any
other source through `lift_values` with the new `TAKE_DROP` table entry (each element its call's top-left value,
SPEC-0011 rule 2, pending T-1960), charged the array's cells per element before any element runs (DEVIATION
added to arrays.md "Array results above 1,048,576 cells"). Unlifted calls are unchanged. TAKE and DROP's
static-analysis fallback arms and signature helpers are deleted; `LiftOptions::for_entry` replaces the inline
options in `lift_call_with`. Eval: probes known failures 1696 -> 1681 (m7-argument-modes F4:H6, I4, I5, K1-K4),
no regressions.
