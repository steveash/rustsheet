---
id: T-2326
title: "Legacy IF arm that is a single-cell INDEX(range,row) shows no @ at all (probes/legacy-if-arm-other-references M12)"
milestone: M2
lane: engine
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A legacy `=IF(TRUE,INDEX(B1:B6,3))` shows no `@` anywhere, as in Excel.

## Context
Known failure probes/legacy-if-arm-other-references M12 (T-1969): ours shows `=@IF(TRUE,INDEX(B1:B6,3))`, Excel `=IF(TRUE,INDEX(B1:B6,3))`. Code: `is_range_arm` in static_analysis/if_arm.rs treats INDEX of a range as a range arm; the single-cell shape needs its own rule. Probe more shapes (SUM/ABS over it) first.

## Acceptance criteria
- [ ] probes/legacy-if-arm-other-references M12 passes with no regression in the other cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
