---
id: T-1291
title: "STDEVA/STDEVPA/VARA/VARPA typed numeric text and MODE typed text and boolean arguments match Excel"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
STDEVA, STDEVPA, VARA and VARPA count a typed numeric text argument as its number, and MODE, MODE.SNGL and MODE.MULT treat typed text and boolean arguments as Excel does.

## Context
probes/stat-argument-modes (excel-graph, 2026-10-09, T-1080; legend in docs/excel-semantics/statistics.md "Statistical function argument modes"): `STDEVA(1,"2",4,TRUE)` is 1.4142135623730951 (as STDEV of 1,2,4,1; ours 2.1213203435596424), STDEVPA 1.224744871391589
(ours 1.5), VARA 2 (ours 4.5), VARPA 1.5 (ours 2.25): Excel counts the typed "2" as 2. `MODE(1,"2",4,TRUE)`,
MODE.SNGL and MODE.MULT are `#N/A` in Excel (ours `#VALUE!`), although STDEV/VAR coerce the same typed TRUE to 1: probe
whether MODE skips typed text and booleans (e.g. `MODE(1,"1",TRUE,1)`) before implementing.

## Acceptance criteria
- [ ] The H24, H27, H30, H69, H72, H75 and H78 cells of probes/stat-argument-modes pass.
- [ ] Any new rule is recorded in docs/excel-semantics/statistics.md with probe evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
