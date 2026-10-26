---
id: T-2687
title: "Probe CEILING/FLOOR 16th-digit ties with a non-unit significance (0.5, 3, -2)"
milestone: M5
lane: functions-math
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The CEILING/FLOOR family's cut is shown by an excel-graph probe to apply to `value/significance` (not to the value or the result) when the significance is not 1.

## Context
T-2652's probe (probes/t2652-floor-ceiling-ties) uses significance 1 only, so ratio and value are the same number. numbers.md "FLOOR.PRECISE: a 16th-digit tie..." records that gap. Build the ties from exact arithmetic or stored exact values, e.g. a tie in value/0.5, value/3, value/-2. `snap_ratio` is in functions/math_and_trigonometry/ceiling_floor.rs.

## Acceptance criteria
- [ ] A probe covers non-unit significances (0.5, 3, -2, with a negative value) for the seven family functions, filled from excel-graph.
- [ ] The engine matches, or a task is filed for any difference; numbers.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
