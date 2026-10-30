---
id: T-3302
title: "Probe mm after [mm] and h:mm:ss (minute_read gaps)"
milestone: M10
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Know what Excel does with `mm` after `[mm]` and after `h:mm:ss`, and make `minute_read` match.

## Context
T-2961 made any `m`/`mm` after a minute a month (`minute_read`, formatter/parser.rs), but an elapsed `[m]`/`[mm]` does not set it, and `h:mm:ss mm` is unprobed (docs/excel-semantics/number-formats.md "`mm` after a minute is a month"). Probe `[h]:mm mm`, `[mm] mm`, `h:mm:ss mm`, `h:mm:ss yyyy-mm` via a case in eval/suites/probes (see probes/mm-after-time-and-date-token), then fix the rule if needed.

## Acceptance criteria
- [ ] Probe cells with excel-graph values pass; the rule is updated in number-formats.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
