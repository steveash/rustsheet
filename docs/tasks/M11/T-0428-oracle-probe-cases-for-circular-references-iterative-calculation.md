---
id: T-0428
title: "Oracle probe cases for circular references, iterative calculation and TABLE() data tables (cases only, Phase 2 implements)"
milestone: M11
lane: research
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Phase 1 has oracle cases for circular references, iterative calculation and TABLE() data
tables, so Phase 2's dependency-graph work (M3) starts with its acceptance tests already
written (decision 0027: "its oracle cases are still written in Phase 1").

## Context
Owner request (2026-10-15): finish Phase 1's test suite before the Phase 2 re-architecture
(decision 0027). Building test infrastructure is allowed during Phase 1, and this is one of the
suites the milestone exit criteria name that does not exist yet.
- Iterative calculation is a workbook setting (`<calcPr iterate="1" iterateCount=…
  iterateDelta=…>`). spec2xlsx may need a field for it.
- Data tables are stored as `<f t="dataTable" r1=… r2=… dt2D=…>`. Check whether excel-graph
  recalculates them.
- Without iteration, a circular reference gives 0 and a warning in desktop Excel. Record what
  excel-graph returns and saves.
- Cases the engine cannot pass are expected. They go in with their known failures, linked to M3
  tasks (T-0401's spec), not engine fixes now.

## Acceptance criteria
- [x] ≥ 15 probe cases are oracle-filled and canary-verified, or documented as not evaluable by
  excel-graph with the reason. They cover:
  - a direct self-reference, and 2- and 3-cell cycles, with iteration off and on;
  - iterateCount and iterateDelta limits, and convergent and divergent cycles;
  - one- and two-variable data tables.
- [x] Findings recorded in docs/excel-semantics/ (a new recalc.md, or the existing file that
  covers calculation).
- [x] `make check` green (new known failures go in the baseline through `make eval-baseline` as
  new coverage); reviewers approve.

## Out of scope
- Implementing iteration or data tables (Phase 2).

## Outcome
Six oracle-filled probe cases with 58 cells (excel-graph, 2026-10-16, canary-checked by `fill`).
The "≥ 15 probe cases" criterion is met as cells, one question each, grouped by workbook setting,
because the calculation settings are per workbook:
- probes/circular-iteration-off (M3, 13 cells);
- probes/circular-iteration-on (M3, 12 cells: iterateCount 100, iterateDelta 0.001);
- probes/circular-iteration-limits (M3, 7 cells: iterateCount 5, iterateDelta 0.5);
- probes/circular-iteration-delta and -delta-small (M3, 3 cells each: only convergent cycles,
  iterateDelta 0.5 and 0.001);
- probes/data-tables (M11, 20 cells: one-variable column and row tables, a two-variable table, a
  chained table, and reads of the results).
spec2xlsx now takes a top-level `"calculation"` (written on `<calcPr>`) and a per-sheet
`"data_tables"` (written as `<f t="dataTable">`). `CaseSpec` / `SheetSpec` carry both, and the
engine adapter ignores them until Phase 2.
Findings are in the new docs/excel-semantics/recalc.md:
- the oracle calculates each workbook 3 times, and iteration resumes from the previous values;
- without iteration, cycle cells and their readers are 0;
- sweeps run in place, in sheet order;
- iterateDelta stops the whole workbook, not a single cycle (the exact stopping test is
  UNVERIFIED);
- data tables are recalculated, and FORMULATEXT shows `{=TABLE(,A1)}`.
Follow-up T-0429 lets these cells be scored against an engine that calculates once.
Eval delta: probes 6 new cases, +50 new-coverage known failures, 8 cells pass, 0 regressions.
