---
id: T-2336
title: "Probe AGGREGATE error-cell k under options 1-3, 5, 7 and an IF-picked fourth argument for functions 1-13"
milestone: M5
lane: functions-stat
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for the two AGGREGATE readings T-2281 implemented beyond its probe: a referenced
error cell as k under options 1, 2, 3, 5 and 7, and a fourth argument IF or CHOOSE picks a
reference for under functions 1-13.

## Context
T-2281 (probes/aggregate-range-k-first-cell) showed that a referenced error cell as k for functions
14-19 is `#VALUE!` under options 0, 4 and 6. `Model::aggregate_k`
(crates/rustsheet-engine/src/functions/aggregate.rs) applies that rule under every option.
`fn_aggregate` also reads its fourth argument through `evaluate_picked_reference_arg`, so
`AGGREGATE(9,0,A1:A2,IF(TRUE,B1))` sums B1 as a reference. No cell probes that yet.
Rule: docs/excel-semantics/statistics.md "AGGREGATE 14-19: k is read before the array".

## Acceptance criteria
- [ ] A probes case (excel-probe skill, excel-graph) covers `AGGREGATE(14,o,A1:A5,D1)` with D1
  `=NA()` for o = 1, 2, 3, 5 and 7, and `AGGREGATE(9,0,A1:A2,IF(TRUE,B1))`,
  `AGGREGATE(9,0,A1:A2,CHOOSE(1,B1))`. Any cell ours misses is a known failure with a follow-up
  task, and statistics.md states the result.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `probes/aggregate-error-k-picked-ref` (8 formula cells) filled from excel-graph; ours matched every
cell, so no code change and no follow-up. statistics.md records the results. Eval delta: 1 new probe
case, all cells pass; recorded in eval/baseline/probes.txt by `make eval-baseline`.
