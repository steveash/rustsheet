---
id: T-1748
title: "AVEDEV/DEVSQ: an empty argument counts as 0; QUARTILE of a typed boolean"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
AVEDEV and DEVSQ count an empty argument as 0, and QUARTILE/QUARTILE.INC accept a typed boolean as a one-element array, as Excel does.

## Context
Oracle (excel-graph, 2026-10-17): `AVEDEV(1,)` and `DEVSQ(1,)` are 0.5 (empty counted as 0; we give 0), and `QUARTILE(TRUE,1)` / `QUARTILE.INC(TRUE,1)` are 1 (we return an error). Failing cells: probes/stat-order-floor-batch A274, A299, A102, A128; see docs/excel-semantics/statistics.md.

## Acceptance criteria
- [ ] The four cells above pass and leave eval/baseline/probes.txt.
- [ ] `make check` green; reviewers approve.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
