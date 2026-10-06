---
id: T-1032
title: "Probe GEOMEAN of mixed-magnitude values (1E+100,9E+307; 1E-100,1E-300)"
milestone: M1
lane: functions-stat
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle-confirm GEOMEAN of mixed-magnitude finite positives.

## Context
T-0992 made the engine return the bounded value for these (unit tests in test_geomean.rs), unprobed: numbers.md marks it UNVERIFIED.

## Acceptance criteria
- [ ] Probe cells in probes/aggregate-overflow (or a new probe) for GEOMEAN(1E+100,9E+307), GEOMEAN(1E-100,1E-300), GEOMEAN(1E+154,1E+300) get oracle values; engine matches or a task is filed; numbers.md status updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
