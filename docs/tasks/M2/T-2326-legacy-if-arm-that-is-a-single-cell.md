---
id: T-2326
title: "Legacy IF arm that is a single-cell INDEX(range,row) shows no @ at all (probes/legacy-if-arm-other-references M12)"
milestone: M2
lane: engine
status: done
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
- [x] probes/legacy-if-arm-other-references M12 passes with no regression in the other cells.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
A legacy IF or IFS whose value arms are all `INDEX(range,row)` with a literal non-zero row gets no `@` (`returns_single_cell_index`, static_analysis/if_arm.rs); the INDEX arm stays a reference arm. probes/legacy-if-arm-other-references M12 fixed (1 known failure removed), P12 unchanged.
