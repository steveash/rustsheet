---
id: T-1748
title: "AVEDEV/DEVSQ: an empty argument counts as 0; QUARTILE of a typed boolean"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
campaign: visible
depends_on: []
---

## Goal
AVEDEV and DEVSQ count an empty argument as 0, and QUARTILE/QUARTILE.INC accept a typed boolean as a one-element array, as Excel does.

## Context
Oracle (excel-graph, 2026-10-17): `AVEDEV(1,)` and `DEVSQ(1,)` are 0.5 (empty counted as 0; we give 0), and `QUARTILE(TRUE,1)` / `QUARTILE.INC(TRUE,1)` are 1 (we return an error). Failing cells: probes/stat-order-floor-batch A274, A299, A102, A128; see docs/excel-semantics/statistics.md.

Visibly-off cells (gap triage 2026-10-21, decision 0083): 4 cells, 2 numeric past half a unit in the 15th digit and 2 with a wrong type, error or text, blamed on QUARTILE 1, QUARTILE.INC 1, AVEDEV 1, DEVSQ 1. Every one is listed above.

## Acceptance criteria
- [ ] Visible parity: the 4 visibly-off cells listed under Context (2026-10-21) have Excel's value to 15 significant digits, or its type, error and text: the 4 lines in eval/baseline/probes.txt disappear (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] The four cells above pass and leave eval/baseline/probes.txt.
- [ ] `make check` green; reviewers approve.

## Out of scope
- SUMSQ and other functions (probe `SUMSQ(1,)` before changing it).

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (AVEDEV/DEVSQ 0 for an empty argument and QUARTILE errors on a typed TRUE), so this task is in the campaign. Its target is that every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0), or has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it.
