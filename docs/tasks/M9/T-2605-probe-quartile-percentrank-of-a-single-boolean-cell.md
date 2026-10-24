---
id: T-2605
title: "Probe QUARTILE/PERCENTRANK of a single boolean cell and a bad significance on empty data"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The engine's reading of a single cell holding a boolean (`QUARTILE(A1,1)`, `PERCENTILE.INC(A1,0)`, `PERCENTRANK(A1,1)` with A1 = TRUE) and of a bad significance on empty data (`PERCENTRANK(<empty cell>,2,0)`, ours `#N/A`) has Excel evidence, and the engine matches it.

## Context
Follow-up of T-2066 (excel-fidelity review). A typed boolean is a one-element array (probes-numeric/t1748-typed-boolean-data), and a single text cell is empty data (probes/quartile-single-text-cell, quartile-numeric-text-cell), but a single boolean cell is unprobed and keeps the old reading (`percentile_numbers` in functions/statistical/percentile.rs). PERCENTRANK checks x, then empty data, then significance (`percentrank_of`); whether a bad significance beats the empty-data `#N/A` is unprobed. Semantics: docs/excel-semantics/statistics.md "Array arguments given a scalar, a single cell or an error". Use the excel-probe skill.

## Acceptance criteria
- [ ] A new probe case with oracle values covers those cells (QUARTILE, QUARTILE.EXC, PERCENTILE.INC/.EXC, PERCENTRANK.INC/.EXC of a TRUE and a FALSE cell; PERCENTRANK of an empty and a text cell with significance 0 and 2), and every cell passes or its cause is recorded in statistics.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
