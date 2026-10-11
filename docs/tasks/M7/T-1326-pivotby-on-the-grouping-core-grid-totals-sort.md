---
id: T-1326
title: "PIVOTBY on the grouping core: grid, totals, sort, filter, headers (SPEC-0006 step 5)"
milestone: M7
lane: functions-lookup
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-1322]
spec: docs/specs/SPEC-0006-groupby-pivotby.md
---

## Goal
PIVOTBY computes the probed grid, totals, sort orders, filter and headers on the shared grouping core.

## Context
SPEC-0006 PIVOTBY section; anchors P!A1, P!M1, P!Y1, P!A25, P!M25, P!M73, P!Y73, P!A97, P!M97, P!M121 (P!Y97 pending T-1328). No-record cells are `""`; column subtotals have `""` in the second header row.

## Acceptance criteria
- [ ] Those anchors pass (LAMBDA forms until T-1325) and leave the baseline; probes/m7-argument-modes F35-H35 pass once eta names exist.
- [ ] A PIVOTBY cross-product of two high-cardinality fields is size-checked before allocation (SPEC-0006 Design step 6).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
