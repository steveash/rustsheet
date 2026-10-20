---
id: T-2100
title: "Probe legacy T and N over CHOOSE with an array index: single-cell arms, column/2-D index, arm counts"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Excel's value and FORMULATEXT are known for the other shapes of a legacy N or T over a CHOOSE with an array-constant index over references, and the engine matches them.

## Context
T-1915 covers only `=N(CHOOSE({1,2},H1:H2,I1:I2))` (probes/subtotal-take-array-selector D1-D7). Unprobed: T, single-cell arms (`CHOOSE({1,2},H1,I1)`), column and 2-D indexes, one or three-plus arms. They keep the old `@` in `chooses_references_by_an_array_index` (static_analysis/legacy.rs). Use the `excel-probe` skill.

## Acceptance criteria
- [ ] A probe case with the shapes above has oracle values, and the engine matches them (baseline shrinks or the shapes are recorded in docs/excel-semantics/references.md).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
