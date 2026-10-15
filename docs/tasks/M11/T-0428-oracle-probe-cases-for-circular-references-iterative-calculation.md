---
id: T-0428
title: "Oracle probe cases for circular references, iterative calculation and TABLE() data tables (cases only, Phase 2 implements)"
milestone: M11
lane: research
status: todo
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
- [ ] ≥ 15 probe cases are oracle-filled and canary-verified, or documented as not evaluable by
  excel-graph with the reason. They cover:
  - a direct self-reference, and 2- and 3-cell cycles, with iteration off and on;
  - iterateCount and iterateDelta limits, and convergent and divergent cycles;
  - one- and two-variable data tables.
- [ ] Findings recorded in docs/excel-semantics/ (a new recalc.md, or the existing file that
  covers calculation).
- [ ] `make check` green (new known failures go in the baseline through `make eval-baseline` as
  new coverage); reviewers approve.

## Out of scope
- Implementing iteration or data tables (Phase 2).
