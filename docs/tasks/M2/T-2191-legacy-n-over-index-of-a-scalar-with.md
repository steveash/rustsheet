---
id: T-2191
title: "Legacy N over INDEX of a scalar with a 0 index is #VALUE! and FORMULATEXT shows no @"
milestone: M2
lane: functions-lookup
status: done
priority: P1
size: S
tier: sonnet
depends_on: []
---

## Goal
A legacy (non-array) `=N(INDEX(5,0,0))` and `=N(INDEX(5,0))` are `#VALUE!` and their FORMULATEXT has no `@`, as in Excel.

## Context
probes/index-zero-index-scalar-shape (T-1943, excel-graph) cells E5, E6, F5, F6 fail: the engine returns 5 and shows `=N(@INDEX(5,0,0))`.
The same picks of an array constant (E1-E4, E7-E9) already match. See docs/excel-semantics/lookup.md "INDEX of an array: a whole row stays an array".
Look at where the legacy marker (SPEC-0012) decides a call is reference-returning for a scalar first argument.

## Acceptance criteria
- [ ] The four cells above pass and are removed from eval/baseline; no other cell regresses.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`indexes_a_number_whole` (static_analysis/legacy.rs) treats `INDEX(<number>,0)` and `INDEX(<number>,0,0)` like an array-constant
return under legacy N and T: `#VALUE!` and no `@` in FORMULATEXT. Eval delta: probes/index-zero-index-scalar-shape E5, E6, F5, F6
fixed (4 known-failure lines removed), no regressions.
Follow-up: T-2368 (probe T and the text, boolean and negative scalar forms).
