---
id: T-1195
title: "COUNTBLANK of a formula-bodied name that evaluates to an error is #VALUE! (engine 0)"
milestone: M1
lane: functions-stat
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
COUNTBLANK of a defined name whose formula body evaluates to an error gives `#VALUE!`, as Excel does.

## Context
probes/union-defined-name-uses H12 (excel-graph, 2026-10-09, T-1065): `=COUNTBLANK(bad)` with `bad` =
`=1+"a"` is `#VALUE!` in Excel; the engine gives 0 (and so did main before T-1065). `COUNTBLANK(mu)`
with a union body is `#VALUE!` and matches (H13). Probe first whether it is the error or the
non-reference body that decides (`bad2` = `=1/0`, `num` = `=5`, `txt` = `="x"`), then fix the name
path in `fn_countblank` (functions/statistical/count.rs). Record the rule in references.md.

## Acceptance criteria
- [ ] probes/union-defined-name-uses H12 passes, plus the new probe cells; baseline updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- COUNTBLANK of ranges and unions (already verified).
