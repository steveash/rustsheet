---
id: T-1910
title: "Legacy cell: no @ and a spill for a top-level SWITCH with an array result"
milestone: M2
lane: engine
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
A legacy cell `=SWITCH(1,1,{2;3})` shows no `@` and spills 2, 3, as Excel's does.

## Context
probes/legacy-array-at-shapes C20 (Excel `=SWITCH(1,1,{2;3})`, ours `=@SWITCH(...)`) and C11 (Excel 3, ours empty). The nested `=T(SWITCH(...))` already matches. T-1717 `leaves_call_whole` / `legacy_array_call_at` in static_analysis/legacy.rs. Found by T-1847; references.md 'Legacy formulas put @ on array-valued calls'.

Export side: `remove_redundant_legacy_intersection` (static_analysis/redundant.rs) must drop exactly the `@`s the parser adds, so change both.

## Acceptance criteria
- [x] The listed probes/legacy-array-at-shapes cells pass; the baseline shrinks.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
A SWITCH returning only array constants no longer gets the automatic `@` in a legacy cell (`switch_of_array_constants` via `legacy_call_reads_a_value`, `static_analysis/legacy.rs`); the export drops the same `@`s through the shared probe, so round-trip is unchanged. probes/legacy-array-at-shapes C11 and C20 now pass (probes known failures 2137 -> 2135). Range-returning and nested SWITCH keep the `@`; unprobed, filed as T-1944.
