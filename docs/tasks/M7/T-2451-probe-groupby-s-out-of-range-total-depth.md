---
id: T-2451
title: "Probe GROUPBY's out-of-range total_depth under two lifted options through ERROR.TYPE (groupby-lifted-args2 A27 is an undecoded rich error)"
milestone: M7
lane: functions-lookup
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's value for GROUPBY with an out-of-range total_depth element under two lifted options is known and scored.

## Context
Found by T-2399: `GROUPBY(Data!A1:A4,Data!B1:B4,SUM,{0,"x"},{0;5})` spills a 2x2 whose A27 (field_headers 0, total_depth 5)
excel-graph returns as a rich error it cannot decode, so probes/groupby-lifted-args2 leaves it unscored. Read it through
`IFERROR(ERROR.TYPE(INDEX(...,2,1)),0)` in a new probe case, as T-2397 did for groupby-lifted-args A33/A41, and fix any mismatch
in `Model::lift_options` (functions/lookup_and_reference/lifted_options.rs).

## Acceptance criteria
- [x] A new probe cell gives Excel's error type for that element, and ours matches it; arrays.md ("GROUPBY and PIVOTBY lift
  over an array field_headers or total depth") cites it.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
New probe cell probes/groupby-lifted-args2 A61 reads element (2,1) through `IFERROR(ERROR.TYPE(INDEX(..,2,1)),0)`: excel-graph gives 3 (`#VALUE!`), and ours already matches (A27 shows `#VALUE!` in the refilled sidecar too). No engine change. Recorded in arrays.md. Eval delta: none (one new passing cell).
