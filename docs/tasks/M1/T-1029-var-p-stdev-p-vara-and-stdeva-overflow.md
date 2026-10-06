---
id: T-1029
title: "VAR.P, STDEV.P, VARA and STDEVA overflow to #NUM!"
milestone: M1
lane: functions-stat
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
VAR.P, STDEV.P, VARA and STDEVA over values whose deviations overflow give `#NUM!` like Excel, not `NaN`.

## Context
Found by T-1021. probes/aggregate-overflow Calc!A51:C54 and A57:C60: Excel `#NUM!`, engine `NaN` text and a non-error value. VAR/STDEV/VAR.S/STDEV.S already overflow correctly (T-0991, same `CalcResult::finite_number_or_num` check). See docs/excel-semantics/numbers.md (Aggregate overflow).

## Acceptance criteria
- [ ] Calc!A51:C54 and A57:C60 pass and leave eval/baseline/probes.txt (the A cells already pass).
- [ ] `make check` green; reviewers approve.

## Out of scope
- MEDIAN and GEOMEAN (T-0992) and AGGREGATE (T-0993).
