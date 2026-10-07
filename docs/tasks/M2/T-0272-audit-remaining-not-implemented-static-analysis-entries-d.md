---
id: T-0272
title: "Audit remaining not_implemented static_analysis entries (D*, lookups, IFS/SWITCH, aggregates) for runtime lifting"
milestone: M2
lane: functions-text
status: done
priority: P1
size: S
tier: sonnet
depends_on: []
---

## Goal
No `not_implemented` entry in `static_analysis.rs` belongs to a function that lifts over arrays at runtime.

## Context
T-0271 fixed the *B text functions and audited ~60 other `not_implemented` functions with a throwaway debug test (range arguments in 1-4 positions, looking for the scalar-context `debug_assert!` in `Model::set_cells_with_result`). Not audited: the D* database functions, HLOOKUP/VLOOKUP/LOOKUP/MATCH/XMATCH, IFS/SWITCH, CONCAT, TEXTJOIN, and the aggregates (AVERAGE, COUNT, SUBTOTAL, PRODUCT, ...). Fuzz target `formula_eval` finds these only by chance.

## Acceptance criteria
- [ ] Every remaining `not_implemented` function is evaluated with range/array arguments in each position; any that hit the assert gets the correct static analysis plus a unit test (or a P0 task if out of lane).
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Audited every remaining `not_implemented` function with range arguments in each of 5 positions (throwaway test, not committed). Only IFS/SWITCH (range value argument) hit the scalar-context assert: both now `Unknown` in static analysis, with a unit test and probe `ifs-switch-array-values` (oracle-filled; 12 new known-failure probe cells are the array-condition lifting, T-0276). XNPV with two arguments panics (index out of bounds): P0 T-0275. Eval delta: probes known failures 205 -> 217 (new coverage, no regressions).
