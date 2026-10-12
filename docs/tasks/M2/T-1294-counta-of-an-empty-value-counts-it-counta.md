---
id: T-1294
title: "COUNTA of an empty value counts it: COUNTA(IFERROR(<blank>,1)) is 1; probe COUNTA with omitted arguments"
milestone: M2
lane: functions-stat
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
COUNTA counts the empty value IFERROR/IFNA give for a blank cell, as Excel does, and the rule for omitted arguments (`COUNTA(,)`, `COUNTA(1,)`) is probed and matched.

## Context
probes/iferror-blank-value D8 (excel-graph, T-0291): `COUNTA(IFERROR(B1,1))` with B1 blank is 1 in Excel (COUNT is 1 too);
the engine gives 0 because `fn_counta` (functions/statistical/count.rs) skips `RefArg::Value(EmptyArg)`. The same variant
stands for an omitted argument, so probe `COUNTA(,)`, `COUNTA(1,)` and `COUNT(,)` first (excel-probe) and change the arm
only as far as the evidence goes. While probing, also cover the empty value's unprobed consumers
(NOT, an IF condition, unary minus, SUM/AVERAGE, LEN, ISLOGICAL, inside LET/arrays) and one omitted-argument XMATCH and SORT
key, since `compare_values` now reads every `EmptyArg` as a blank (T-0291 review). See docs/excel-semantics/logic.md "IFERROR / IFNA of a blank cell is an empty value".

## Acceptance criteria
- [x] probes/iferror-blank-value D8 passes, and the omitted-argument probe cells pass or have follow-ups (T-1394, T-1395).
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`fn_counta` now counts `EmptyArg` (an omitted argument and IFERROR/IFNA's empty value); COUNT already did. New probe probes/counta-empty-value (excel-graph) covers omitted arguments and consumers of the empty value; D8 of iferror-blank-value passes. Eval: fixed 1, new-coverage failures 4 recorded in the baseline (D18 -> T-1395, D22-D24 -> T-1394), regressions 0.
