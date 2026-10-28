---
id: T-2596
title: "IMSQRT and the IM functions over a REDUCE that returns TRIMRANGE of a range are #VALUE!, not lifted"
milestone: M9
lane: functions-engineering
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
IMSQRT (and the IM functions) over a REDUCE whose LAMBDA returns TRIMRANGE of a multi-cell range give `#VALUE!` as Excel does, not a lifted array.

## Context
probes/t2579-array-value-edges (excel-graph, 2026-10-24, T-2579): IMSQRT and IMABS do not lift over a multi-cell
reference (`ROWS(IMSQRT(D!B1:B2))` is `#VALUE!` off the range's rows, B44; `IMSQRT(LET(x,D!B1:B2,x))` B48, IMABS B50), but
do over an array (`ROWS(IMSQRT(REDUCE(0,1,LAMBDA(a,b,{2;8}))))` is 2, B46), and a REDUCE that returns a range is that
reference (`IMSQRT(REDUCE(0,1,LAMBDA(a,b,D!B1:B2)))` `#VALUE!`, B40, ours matches). Only the TRIMRANGE form differs:
`ROWS(IMSQRT(REDUCE(0,1,LAMBDA(a,b,TRIMRANGE(D!B1:B2)))))` is `#VALUE!` (B49), ours 2: our REDUCE hands on TRIMRANGE's
value, not its reference. Owned: probes/t2579-array-value-edges B49; gen-m9/df083d85b436b49a Calc!A20
(`IMSQRT(REDUCE(..,LAMBDA(y,z,TRIMRANGE(Data!A4:D10))))`, Excel `#VALUE!`). Code: REDUCE's step result
(functions/logical, `lambda_result_value`) and TRIMRANGE's reference form (`trimrange_reference`).

## Acceptance criteria
- [x] The owned cells pass and leave eval/baseline, with no new known-failure lines.
- [x] arrays.md records the rule (a REDUCE step that is a TRIMRANGE over a range is a reference).
- [x] `make check` green; reviewers approve.

## Out of scope
- Other LAMBDA helpers returning TRIMRANGE (probe if needed).

## Outcome
REDUCE's LAMBDA caller keeps the trimmed reference of a body that is TRIMRANGE over a range
(`LambdaCaller::keeping_trimrange_references`, functions/logical/lambda.rs), as a body that is the
range itself already did, so IMSQRT does not lift over it. Unit test
`reduce_step_trimrange_of_a_range_is_a_reference`; arrays.md "Array-value edges in the gen-m9 sweep"
updated. Eval: 32 known failures fixed (gen-m9 df083d85b436b49a Calc!A20; probes/t2579-array-value-edges
P!B49; probes/t2436/df083d85b436b49a Calc!A61:D67, T61, U61), no regressions.
