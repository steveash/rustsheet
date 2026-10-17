---
id: T-1784
title: "Probe WORKDAY.INTL/NETWORKDAYS.INTL with a lifted weekend and a lifted date; TAKE/DROP of a range in EOMONTH/EDATE; mismatched-shape date lifts"
milestone: M6
lane: functions-date
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's behavior is pinned for the array-literal lifts T-1236 left unprobed: WORKDAY.INTL/NETWORKDAYS.INTL with a lifted `weekend` and a lifted date together, `TAKE`/`DROP` of a range as a date argument, and mismatched-shape or error-element date arrays.

## Context
T-1236 lifts the ToolPak-era date functions over array literals with operator broadcasting (hand-derived, no oracle). A lifted date plus a lifted weekend gives `#N/IMPL!` (`date_and_time/lift.rs`). The regenerated gen-m9 suite no longer holds the EOMONTH/TAKE cells T-1236 cited. Semantics: docs/excel-semantics/dates.md "Date function argument modes".

## Acceptance criteria
- [ ] probes/date-argument-modes (or a new probe) gains oracle cells for each case above, and the engine and dates.md match them.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
