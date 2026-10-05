---
id: T-0236
title: "AVERAGE family: probe value-arg coercion (array booleans, empty args) and align for_each_value"
milestone: M2
lane: functions-stat
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
AVERAGE, GEOMEAN, MEDIAN, STDEV/VAR and the other `for_each_value` users coerce their arguments
exactly as Excel does: in particular booleans and text inside an array literal, and empty arguments
(`AVERAGE(5,)`), match the oracle.

## Context
T-0217 probe `probes/min-max-value-args` shows MIN/MAX ignore booleans and text in arrays
(`=MIN({TRUE,5})` → 5) and count an empty argument as 0 (`=MIN(5,)` → 0). `for_each_value`
(`crates/rustsheet-engine/src/functions/statistical/count_and_average.rs`) instead counts array
booleans as 1/0 and skips empty arguments. Whether AVERAGE & co. differ from MIN/MAX here is
unprobed: write a probe (excel-probe skill) for `AVERAGE({TRUE,5})`, `AVERAGE(5,)`, `MEDIAN({TRUE,1,3})`,
`GEOMEAN({TRUE,4})`, `STDEV({TRUE,1,3})`, then align `for_each_value` (one iterator; reuse
`fold_min_max`'s coercion via `cast_to_number` if it fits). See docs/excel-semantics/coercion.md.

## Acceptance criteria
- [ ] Probe case with oracle values committed; docs/excel-semantics/coercion.md entry updated.
- [ ] `for_each_value` users match the probe; eval shows no new F lines.
- [ ] `make check` green; reviewers approve.

## Out of scope
- MIN/MAX (T-0217).

## Outcome
Probe `probes/average-value-args` (32 cells, excel-graph oracle). AVERAGE/MEDIAN/GEOMEAN/STDEV/VAR (and COUNT) ignore booleans inside arrays and count an empty argument as 0 (`COUNT(5,)` = 2), exactly like MIN/MAX. `for_each_value`, `fn_count`, `fn_stdev_s/p`, `fn_var_s/p` aligned. Eval delta: new case 32/32 pass, 0 regressions; coercion.md entry VERIFIED.
