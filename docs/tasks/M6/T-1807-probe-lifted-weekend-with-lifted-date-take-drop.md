---
id: T-1807
title: "Probe lifted weekend with lifted date, TAKE/DROP of a range and mismatched shapes in the date functions"
milestone: M6
lane: functions-date
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's behavior is pinned for the date-function lifts left unprobed: WORKDAY.INTL/NETWORKDAYS.INTL with a lifted `weekend` and a lifted date together, `TAKE`/`DROP` of a range as a date argument (`#VALUE!` as a reference), mismatched-shape date arrays, and a boolean `months` in an array.

## Context
SPEC-0011 lifts scalar arguments through one driver. These cases have no oracle cells; the regenerated gen-m9 suite no longer holds the EOMONTH/TAKE cells T-1236 cited. Semantics: docs/excel-semantics/dates.md "Date function argument modes".

## Acceptance criteria
- [ ] probes/date-argument-modes (or a new probe) gains oracle cells for each case above, and dates.md records them.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Added probes/date-lift-open (35 formulas x ROWS/COLUMNS/INDEX cells) with excel-graph oracle values and a dates.md entry
covering each case in the goal. 105 of 175 cells pass; the 70 failures are all `#N/IMPL!` from the WORKDAY/NETWORKDAYS lift
already tracked by T-1236 (no new task needed). No regressions.
