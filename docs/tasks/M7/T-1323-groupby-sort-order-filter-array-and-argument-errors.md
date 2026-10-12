---
id: T-1323
title: "GROUPBY sort_order, filter_array and argument errors (SPEC-0006 step 2)"
milestone: M7
lane: functions-lookup
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-1322]
spec: docs/specs/SPEC-0006-groupby-pivotby.md
---

## Goal
GROUPBY's sort_order, filter_array and invalid-argument errors match the probe.

## Context
SPEC-0006 GROUPBY bullets sort_order / filter_array; anchors G!A145, G!J145, G!S145, G!AB145, G!A169, G!S217, G!AB217, G!A241, G!A217, G!J241. Sort by value columns pins the total rows and keeps blank keys last.

## Acceptance criteria
- [ ] Those anchors pass (in their LAMBDA form until T-1325 lands) and leave the baseline.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
