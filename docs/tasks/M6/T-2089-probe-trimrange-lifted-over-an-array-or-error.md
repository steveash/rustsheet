---
id: T-2089
title: "Probe TRIMRANGE lifted over an array or error source"
milestone: M6
lane: functions-lookup
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Excel's results for TRIMRANGE with a lifted trim_rows or trim_cols over sources T-1957 could not evidence are probed and the engine matches them.

## Context
T-1957 lifted TRIMRANGE into an array of references. Unprobed, implemented by analogy (UNVERIFIED in docs/excel-semantics/lookup.md): an array source (`INDEX(TRIMRANGE({1,2},{0;1}),1,1)`; the engine gives `#VALUE!` per element), a one-cell source that is not a written reference (defined name, `CHOOSE(1,A1)`, `INDIRECT("A1")`; the engine recovers a reference only for a written one), and an error source (`TRIMRANGE(NA(),{0;1})`; the engine gives one `#N/A`, static analysis predicts a 2x1). Use the `excel-probe` skill; code in functions/lookup_and_reference/trimrange.rs (`trimrange_reference`).

## Acceptance criteria
- [x] A probe case holds the formulas above with Excel values; lookup.md records them as VERIFIED and the engine matches (baseline shrinks, no regression).
- [x] `make check` green; reviewers approve.

## Out of scope
- Other functions' lifts.

## Outcome
probes/trimrange-lift-sources (44 cells, excel-graph) answers the three questions, all now matched (44/44; probes
known failures 1669 -> 1668 counting the new case's cells, no regressions). An array source does not give `#VALUE!`:
each element is the trimmed array's top-left value (`INDEX(TRIMRANGE({1,2},{0;1}),1,1)` = 1, SUM 2). Every
one-cell reference source, written or not (defined name, CHOOSE, INDIRECT, OFFSET, IF), gives references. An error
source lifts into an array of its error (`ROWS(TRIMRANGE(NA(),{0;1}))` = 2). In passing: SUM over any lifted TRIMRANGE,
written ranges too, was `#VALUE!` because `picked_reference_arg_reading` flattened its grid; it now reads the grid (10).
Code: `Model::lifted_trimrange` (trimrange.rs), one arm in util.rs. lookup.md: new VERIFIED entry "TRIMRANGE lifted
over an array, a computed reference or an error".
