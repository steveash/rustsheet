---
id: T-2100
title: "Probe legacy T and N over CHOOSE with an array index: single-cell arms, column/2-D index, arm counts"
milestone: M2
lane: research
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Excel's value and FORMULATEXT are known for the other shapes of a legacy N or T over a CHOOSE with an array-constant index over references, and the engine matches them.

## Context
T-1915 covers only `=N(CHOOSE({1,2},H1:H2,I1:I2))` (probes/subtotal-take-array-selector D1-D7). Unprobed: T, single-cell arms (`CHOOSE({1,2},H1,I1)`), column and 2-D indexes, text/boolean/out-of-range index elements, one or three-plus arms. They keep the old `@` in `chooses_references_by_an_array_index` (static_analysis/legacy.rs). Use the `excel-probe` skill.

## Acceptance criteria
- [x] A probe case with the shapes above has oracle values, and the engine matches them (baseline shrinks or the shapes are recorded in docs/excel-semantics/references.md).
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
probes/legacy-n-t-choose-array-index-shapes (excel-graph, 64 cells): every shape probed, N or T over a
CHOOSE with any array-constant index (one element, row, column, 2-D; text, boolean, out-of-range elements)
over one to three cell or range arms, shows no `@` and is `#VALUE!` on the arms' rows and off them.
`chooses_references_by_an_array_index` now covers them all (and drops the automatic `@` on a one-element
index); the new case passes 64/64. Recorded in docs/excel-semantics/references.md.
