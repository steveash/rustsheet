---
id: T-1293
title: "Probe the FORECAST.ETS family on excel-graph: when is it a number, not #N/A"
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Evidence for when FORECAST.ETS, FORECAST.ETS.CONFINT, FORECAST.ETS.SEASONALITY and FORECAST.ETS.STAT return a number on excel-graph, recorded in docs/excel-semantics/statistics.md.

## Context
probes/stat-argument-modes (excel-graph, 2026-10-09, T-1080; legend in docs/excel-semantics/statistics.md "Statistical function argument modes"): all four are `#N/A` on excel-graph for an 8-point series on the timeline 1..8 (as ranges and as array literals),
yet lift a scalar argument (ROWS 2). Is ETS unavailable in Excel for the web, or does it need more points or a seasonality
argument? Probe longer series (e.g. 24 and 36 points, seasonality 0/1/4) before anyone implements the family (the engine
is `#N/IMPL!`). If excel-graph never computes it, mark the catalog entries `oracle: none` per decision 0039 / M14.

## Acceptance criteria
- [ ] A probe case with excel-graph values answers the question; the finding is in statistics.md.
- [ ] `make check` green; reviewers approve.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
