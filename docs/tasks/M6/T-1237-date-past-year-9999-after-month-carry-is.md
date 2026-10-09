---
id: T-1237
title: "DATE past year 9999 after month carry is #NUM!; DATEDIF reads numeric text"
milestone: M6
lane: functions-date
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
DATE is `#NUM!` when month or day carry pushes the year past 9999, and DATEDIF reads numeric text dates, as Excel does.

## Context
probes/date-argument-modes (excel-graph, 2026-10-09, T-1090): `INDEX(DATE(2024,A1:A3,1),2)` (month 45031.5) is
`#NUM!` in Excel, the engine returns 1415864 (G3, G4; check DATE's carry against the 9999 limit); `DATEDIF("45000",46000,"d")`
is 1000 in Excel, `#VALUE!` here (F89). Other date functions already read numeric text. Probe the day-carry form
(`DATE(9999,12,32)`) with the excel-probe skill before implementing it.
Semantics: docs/excel-semantics/dates.md "Date function argument modes".

## Acceptance criteria
- [ ] probes/date-argument-modes G3 (once T-1235 lifts DATE, G4) and F89 pass; baseline updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
