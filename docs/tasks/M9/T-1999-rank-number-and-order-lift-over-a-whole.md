---
id: T-1999
title: "RANK number and order lift over a whole ref through the SPEC-0011 driver"
milestone: M9
lane: functions-stat
status: done
priority: P1
size: M
tier: opus
depends_on: [T-1982]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
RANK/RANK.EQ/RANK.AVG `number` and `order` lift over an array and a multi-cell range through the
SPEC-0011 driver, with a reference `ref` read once, and a lifted call against a non-reference `ref`
keeps Excel's broadcast `#VALUE!` shape.

## Context
Split from T-1982, which moved FORECAST and T.TEST onto the driver and fixed the order-statistic
precedence. RANK was left out because its `ref` is two kinds at once:
- A reference is data, read once.
- A non-reference array behaves like a lifted argument whose every element is `#VALUE!` (or its
  own error). `RANK({4;"x"},IF(A1:C1>0,A1:C1))` is 2x3 (probes/stat-test-lift P28-P31), and
  statistics.md "RANK of a non-reference array" gives the precedence: an error `number` first,
  then the ref element's error, then `#VALUE!`.

No `ArgKind` expresses that today. `rank_static_result` (static_analysis/reference_parameter.rs)
gives the shape, and entering RANK in the table would drop it through `lifted_static_result`.
Options:
- A new `ArgKind`, such as reference-whole/array-lift, that static analysis broadcasts for a
  non-reference shape.
- Keep RANK's fallback arms by a decision record.
- Have `fn_rank` call `lift_call_with` with an all-`Lift` kinds table when `ref` is not a
  reference. Avoid evaluating `ref` twice if you take this route.

Evidence:
- probes/stat-test-lift P20-P33, P72; P84 and P85 already pass.
- probes/stat-argument-modes H492, H493, H495, H496, H498, H499.
- statistics.md "RANK, FORECAST, PROB, Z.TEST and T.TEST: lifted scalar arguments".

Known behaviour from those probes:
- `number` coerces TRUE and numeric text (P26, P32).
- `RANK({1;3},D1:D3)` with D2 = `#N/A` is `#N/A` (P25).
- `RANK("x",D1:D3)` is `#VALUE!` (P85).
- `order` lifts (P22-P24, P27); `order` `"x"` is `#VALUE!` per element (P33).
- A 1x1 array `number` gives a 1x1 array (P72).

## Acceptance criteria
- [x] probes/stat-test-lift P20-P28, P30-P33, P72 and stat-argument-modes H492, H493, H495, H496,
  H498, H499 leave eval/baseline/probes.txt, with no new F lines (the probes/rank-nonref-* cases
  stay green).
- [x] RANK* are in `Function::arg_kinds` and `lifting_table_functions!` with their fallback arms
  deleted, or a decision record keeps them.
- [x] Element errors carry empty messages.
- [x] `make check` green; reviewers approve.

## Out of scope
- Literal-array and scalar `ref` values that the oracle refuses (decision 0039).

## Outcome
RANK, RANK.EQ and RANK.AVG are in the lifting table as `[Lift, ReferenceByCaller, Lift]` and their
fallback arms are gone. The new `ArgKind::ReferenceByCaller` is read by the function (a reference is
data, read once); a non-reference array comes back as a grid of errors through the new
`Model::lift_call_beside` and broadcasts with `number` and `order`, and static analysis broadcasts its
shape (`reference_by_caller_shape`). A new probe, probes/rank-direct-boolean (18 cells, excel-graph),
showed three things the old scalar path got wrong: a direct boolean `number`/`order` coerces, a text
`order` (also `"1"`) is `#VALUE!`, and `order`'s error comes before the data's; the kernel follows it
(statistics.md). Eval: probes known failures 1844 -> 1824 (fixed 20: stat-test-lift P20-P28, P30-P33,
P72; stat-argument-modes H492, H493, H495, H496, H498, H499), new case 19/19, no regressions.
