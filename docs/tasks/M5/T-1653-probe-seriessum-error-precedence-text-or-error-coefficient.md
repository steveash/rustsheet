---
id: T-1653
title: "Probe SERIESSUM error precedence: text or error coefficient on a power <= 0 term with x = 0"
milestone: M5
lane: functions-math
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
SERIESSUM with x = 0 gives Excel's error when a term has a power <= 0 and a text or error coefficient: #NUM!, #VALUE! or the coefficient's error.

## Context
T-1359 made x = 0 `#NUM!` only for a power <= 0, but `fn_seriessum` reads each coefficient before that check, so `SERIESSUM(0,0,1,{#N/A})` gives #N/A and `SERIESSUM(0,1,-1,{"a",1})` gives #VALUE!; neither is probed. Probe those (and an error before or after the bad-power term) with the excel-probe skill, then match; see docs/excel-semantics/numbers.md "SERIESSUM with x = 0".

## Acceptance criteria
- [ ] Probe cells with oracle values; the engine matches them; numbers.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe probes/seriessum-x-zero-error-precedence (27 cells, excel-graph): the engine already matched all of them (coefficients are
read in order, the first problem wins; range coefficients differ for error cells). No engine change; numbers.md records the rule.
Eval delta: probes +27 passing cells, no known-failure change.
