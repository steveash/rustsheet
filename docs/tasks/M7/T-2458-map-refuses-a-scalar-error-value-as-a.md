---
id: T-2458
title: "MAP refuses a scalar error value as a later array with #VALUE!"
milestone: M7
lane: functions-logic
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: visible
---

## Goal
MAP refuses a scalar error value as its second or later array with `#VALUE!`, as Excel does; references to error cells and array-literal error elements stay allowed.

## Context
Evidence: probes/t1946-map-later-array-index-lambda A7, A13, A19, A40 (fail) and A10, A37 (pass), arrays.md "MAP over single values". Code: `by_slices_source` / `is_scalar_value` (bycol_byrow.rs).

Owned gen-m7 cells (T-1946):
- gen-m7/ac67253061bba16a A890 `=MAP(Data!B1,LEFT(NA(),2),Data!B5:D8,_xleta.PRODUCT)`: Excel #VALUE!, ours #N/A
- gen-m7/b24ce74f08d3f91a A237 (`LOOKUP(..)+..` #N/A as a later array) and A1089 (a `#N/A` SWITCH): Excel #VALUE!, ours #N/A
- gen-m7/21b4eef18424ce94 A1393 (MOD #DIV/0!), 4dac39f89709e2fc A137 (BASE #NUM!): Excel #VALUE!, ours the error
- gen-m7/a470dc9e165bdb20 A3360 (DROP #CALC!), f19581ac44da1653 A2477 (SEQUENCE #CALC!): Excel #VALUE!, ours #CALC!
(7 known-failure lines in eval/baseline/gen-m7.txt; full formulas in docs/tasks/M7/T-1946-*.md.)

## Acceptance criteria
- [ ] The four failing probe cells pass; their lines leave eval/baseline/probes.txt.
- [ ] Visible parity: the 7 gen-m7 cells above pass and leave eval/baseline/gen-m7.txt.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other BY*/REDUCE functions' later arrays (unprobed).

## Visible-parity stage
Decision 0083: every owned cell has the wrong error, so the task is in the campaign.
