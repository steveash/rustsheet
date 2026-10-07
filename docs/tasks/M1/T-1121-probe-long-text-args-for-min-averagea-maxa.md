---
id: T-1121
title: "Probe long text args for MIN, AVERAGEA, MAXA and criteria paths"
milestone: M1
lane: functions-math
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
MIN, AVERAGEA, MAXA/MINA and COUNTIF/SUMIF criteria with text over 255 characters match Excel (T-1107 made SUM, AVERAGE, SUMSQ, PRODUCT and MAX `#N/A`; MIN shares the code path but has no oracle cell).

## Context
Extend probes/sum-long-text-args with these functions, fill the oracle, and fix `for_each_value_a` and the criteria matcher if they differ. Code: `cast_aggregate_arg_to_number` in cast.rs.

## Acceptance criteria
- [ ] New probe cells pass with excel-graph values; docs/excel-semantics/numbers.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `probes/long-text-args-min-criteria` (18 cells, excel-graph): MIN already matched; AVERAGEA, MAXA and MINA direct text over 255 characters are now `#N/A` (`for_each_value_a`, AVERAGEA), and an *IF(S) criterion over 255 characters is `#VALUE!` (`lift_criteria`). All 18 cells pass; semantics in numbers.md.
