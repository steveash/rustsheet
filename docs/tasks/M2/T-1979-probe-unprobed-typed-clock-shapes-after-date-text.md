---
id: T-1979
title: "Probe unprobed typed clock shapes after date text (10:00:30 PM, am/pm case, 13:00 PM, 10:00AM, 10 AM, hours past 23, fractions)"
milestone: M2
lane: functions-date
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for typed date+clock text shapes T-1952 left unprobed, and the engine matched to it.

## Context
T-1952 found that typed `1/2/2020 10:00:30` and `10:00 AM` become numbers. `ends_with_typed_clock` (formatter/parse_formatted.rs) also accepts `10:00:30 PM` and lowercase am/pm unprobed, and rejects `10:00AM`, `10 AM` after a date, hours past 23 and fractional seconds. See dates.md typed-input UNVERIFIED notes.

## Acceptance criteria
- [ ] A probe case with excel-graph values covers each shape (with the ENTERED format log in docs/research/); dates.md UNVERIFIED notes resolved; implementation matches.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
