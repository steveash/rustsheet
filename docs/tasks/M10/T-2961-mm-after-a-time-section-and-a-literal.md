---
id: T-2961
title: "mm after a time section and a literal reads as minutes instead of month in h:mm|yyyy-mm-dd"
milestone: M10
lane: functions-text
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
In `h:mm|yyyy-mm-dd` the second `mm` prints the month, as in Excel.

## Context
`TEXT(0.25,"[$-80409]h:mm|yyyy-mm-dd")` is "6:00|5660-05-01" in Excel, ours "6:00|5660-00-01": the `mm` after
`yyyy-` reads as minutes, apparently because an `h` came earlier in the section (probes/t2779-hebrew-saka-more E7,
T-2779). Check whether the plain Gregorian `TEXT(0.25,"h:mm|yyyy-mm-dd")` has the same miss (probe it with
neighbours: `h|yyyy-mm`, `h:mm yyyy mm`, `hh mm|dd mm`) and fix the minutes-or-month rule in `formatter/parser.rs`
or `lexer.rs`, recording the rule in docs/excel-semantics/number-formats.md.

## Acceptance criteria
- [ ] probes/t2779-hebrew-saka-more E7 and the new probe cells pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
A minute already read makes later `m`/`mm` months (`minute_read`, formatter/parser.rs); new probe mm-after-time-and-date-token passes 22/22; t2779-hebrew-saka-more E7 fixed.
