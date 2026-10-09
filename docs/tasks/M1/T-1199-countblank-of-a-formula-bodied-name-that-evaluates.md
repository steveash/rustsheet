---
id: T-1199
title: "COUNTBLANK of a formula-bodied name that evaluates to an error is #VALUE! (engine 0)"
milestone: M1
lane: functions-stat
status: done
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
- [x] probes/union-defined-name-uses H12 passes, plus the new probe cells; baseline updated.
- [x] `make check` green; reviewers approve.

## Out of scope
- COUNTBLANK of ranges and unions (already verified).

## Outcome
probes/countblank-formula-names (15 cells, excel-graph): a name that is not a reference gives its own error when the body is an error and `#VALUE!` for any other value (`=5`, `="x"`, `=""`, `=TRUE`); reference bodies count blanks. `fn_countblank` now returns those errors for a defined-name argument. Eval delta: probes/union-defined-name-uses H12 fixed, +15 new passing cells, 0 regressions. Literal expression arguments are refused by Excel (recorded, no case). No follow-up.
