---
id: T-0453
title: "Recompute a data table only when its formula's precedents changed"
milestone: M11
lane: perf
status: in-progress
priority: P3
size: M
tier: opus
depends_on: [T-0441]
---

## Goal
An edit that cannot change a data table's results does not recompute the table. Today every
`evaluate()` recomputes every table: one on-demand evaluation of the table's formula per result
cell, then a second full pass.

## Context
T-0441 (crates/rustsheet-engine/src/data_tables.rs). A workbook with four 9x9 two-variable tables
over a financial model (calipers pv_goldens/pv23_046) re-evaluates the model 324 times on every
evaluation, edits included.

The results depend on three things:
- the table's formula and the cells it reads, transitively;
- the header values;
- the input cells.

Without the dependency graph (T-0401, Phase 2), a conservative test is enough: skip the
recomputation when no cell the last recomputation read has changed. Record the positions read with
`seen` during the substituted evaluations, and compare their values. Directive 5 still bounds the
recorded set.

## Acceptance criteria
- [ ] Release-build timings in the Outcome: an edit to a cell no table reads, in pv23_046, before
      and after, at least 10x faster.
- [ ] Results unchanged: `make eval` shows no new known failure, and `rustsheet-eval check --suite
      calipers` passes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The dependency graph itself (T-0401).
