---
id: T-1911
title: "Legacy @ goes on an array constant inside an operator (N, ABS over {2;3}+0)"
milestone: M2
lane: engine
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
A legacy cell puts the `@` on an array constant inside an operator: `=N(@{2;3}+0)` and `=ABS(@{2;3}+0)` are 2 with no spill, as Excel's are.

## Context
probes/legacy-array-at-shapes I20, J20 (FORMULATEXT) and I11, J11 (Excel empty, ours 3): ours adds no `@` and spills. Found by T-1847; static_analysis/legacy.rs `legacy_array_context`; references.md 'Legacy formulas put @ on array-valued calls'.

Export side: `remove_redundant_legacy_intersection` (static_analysis/redundant.rs) must drop exactly the `@`s the parser adds, so change both.

## Acceptance criteria
- [ ] The listed probes/legacy-array-at-shapes cells pass; the baseline shrinks.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`add_array_at`/`remove_array_at` (static_analysis/legacy.rs) now put the automatic `@` on array-constant operands of operators in a legacy scalar argument, and drop them on export. probes/legacy-array-at-shapes I11, I20, J11, J20 pass; probes known failures 1948 -> 1944, no regressions.
