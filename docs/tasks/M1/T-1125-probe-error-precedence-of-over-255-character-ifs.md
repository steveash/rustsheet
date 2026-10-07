---
id: T-1125
title: "Probe error precedence of over-255-character IFS criteria"
milestone: M1
lane: functions-math
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
COUNTIFS/SUMIFS with an over-255-character criterion plus another error (error criterion, mismatched ranges) match Excel's precedence.

## Context
T-1121 returns #VALUE! for the long criterion before any other check (`lift_criteria`, if_ifs.rs); Excel's order is unprobed (numbers.md).

## Acceptance criteria
- [ ] Probe cells with excel-graph values; engine matches; numbers.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `probes/ifs-long-criterion-precedence` (7 cells, excel-graph). Only `#REF!` in a criteria range outranks the long criterion; the engine's early check in `lift_criteria` was wrong there, so it moved after range resolution (`build_criteria` sites in if_ifs.rs). All 7 cells and the 25 of T-1121 pass.
