---
id: T-1968
title: "Probe typed text/boolean literals and error arrays vs tails/type in paired statistical functions (COVAR, CORREL, T.TEST)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Probe with excel-probe what Excel gives for typed text and boolean literals (`COVAR("x",B1:B2)`, `CORREL(TRUE,B1:B2)`) and for an error array argument combined with a bad tails/type (`T.TEST(#DIV/0!,A1:A2,0.1,3)`), then match it.

## Context
T-1401 made a number and a single cell a one-value array; typed text/boolean literals stay `#VALUE!` and T.TEST reads samples before tails/type, both unprobed (statistics.md "Array arguments given a scalar...").

## Acceptance criteria
- [ ] Probe results recorded in statistics.md and the engine matches them; make check green.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
