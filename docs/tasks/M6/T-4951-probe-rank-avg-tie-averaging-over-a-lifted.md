---
id: T-4951
title: "Probe RANK.AVG tie averaging over a lifted OFFSET or INDIRECT ref (T-4801 follow-up)"
milestone: M6
lane: functions-lookup
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's RANK.AVG values with tied values in a lifted OFFSET or INDIRECT element's range are known, and ours matches or a fix task exists.

## Context
T-4801 probed the grids with no ties, so RANK.AVG equalled RANK.EQ everywhere. Probe a tie inside one element's range (excel-probe skill), same-sheet and off-sheet; record in docs/excel-semantics/statistics.md.

## Acceptance criteria
- [ ] A probe case filled on excel-graph; ours matches every cell or each mismatch has a fix task.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
