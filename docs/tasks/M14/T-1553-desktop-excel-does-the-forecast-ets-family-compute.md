---
id: T-1553
title: "Desktop Excel: does the FORECAST.ETS family compute a number?"
milestone: M14
lane: research
status: blocked
priority: P3
size: S
tier: sonnet
depends_on: [T-1074]
---

## Goal
Desktop Excel values for FORECAST.ETS, FORECAST.ETS.CONFINT, FORECAST.ETS.SEASONALITY and FORECAST.ETS.STAT: when each returns a number rather than an error, so the family can be implemented.

## Context
Every valid call of the family is `#N/A` on excel-graph (probes/forecast-ets-longer-series.case.json, statistics.md "FORECAST.ETS family: excel-graph never computes it"), so the web oracle cannot score the algorithm or the argument validation. Needs the `excel-desktop` backend (docs/architecture/oracle.md, decision 0039). Re-run that probe case on desktop, then split implementation tasks.

## Acceptance criteria
- [ ] Desktop values for probes/forecast-ets-longer-series.case.json, plus valid `data_completion` 0/1, `aggregation` 1-7 and the other `statistic_type`s, are recorded (as a `desktop` sidecar or in statistics.md) and the statistics.md entry is updated.
- [ ] `make check` green; reviewers approve.

## Blocked because
Needs the `excel-desktop` backend (Windows with Excel), like T-1016.

## Out of scope
-
