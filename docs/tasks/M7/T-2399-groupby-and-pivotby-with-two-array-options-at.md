---
id: T-2399
title: "GROUPBY and PIVOTBY with two array options at once broadcast them pairwise (field_headers {0,1} with total_depth {0;1} is 2x2)"
milestone: M7
lane: functions-lookup
status: done
priority: P2
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
GROUPBY and PIVOTBY with two lifted options at once give Excel's broadcast result.

## Context
Found by T-2359's probe: `GROUPBY(Data!A1:A4,Data!B1:B4,SUM,{0,1},{0;1})` is a 2x2 of `a` (probes/groupby-lifted-args
A209:B210, excel-graph, 2026-10-22). Ours lifts over the first array option only, and each element's nested lift takes the
top-left, so the result is 1x2 and A210:B210 are empty (2 lines in eval/baseline/probes.txt). `Model::lift_options`
(functions/lookup_and_reference/lifted_options.rs) needs to lift over every array option together, broadcasting their shapes the
way the other lifted functions do (`functions/lift`). Probe mismatched shapes too (a 1x3 against a 2x1, and a 2x2 against a 3x1)
to see whether the padding is `#N/A`.

## Acceptance criteria
- [x] A209:B210 pass, and the mismatched-shape probe cells pass; arrays.md records the rule.
- [x] `make check` green; reviewers approve.

## Out of scope
- GROUPBY's `field_relationship` lift (its own path).

## Outcome
`Model::lift_options` now lifts over every array option at once, broadcast with the lifting driver's `broadcast_shape` (a
position past a shorter array is `#N/A`), and runs the call once per class tuple. New probe probes/groupby-lifted-args2
(excel-graph, 46 scored cells: mismatched shapes, error order, a range, PIVOTBY pairs, a lifted field_relationship) passes in
full; groupby-lifted-args A210:B210 now pass. Eval: probes known failures 1975 -> 1973, 0 regressions. arrays.md records the
rule. Follow-up: T-2451 (A27 is an undecoded rich error).
