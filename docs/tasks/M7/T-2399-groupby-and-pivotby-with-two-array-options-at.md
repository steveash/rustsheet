---
id: T-2399
title: "GROUPBY and PIVOTBY with two array options at once broadcast them pairwise (field_headers {0,1} with total_depth {0;1} is 2x2)"
milestone: M7
lane: functions-lookup
status: todo
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
- [ ] A209:B210 pass, and the mismatched-shape probe cells pass; arrays.md records the rule.
- [ ] `make check` green; reviewers approve.

## Out of scope
- GROUPBY's `field_relationship` lift (its own path).
