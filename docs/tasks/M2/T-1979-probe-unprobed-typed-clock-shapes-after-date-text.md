---
id: T-1979
title: "Probe unprobed typed clock shapes after date text (10:00:30 PM, am/pm case, 13:00 PM, 10:00AM, 10 AM, hours past 23, fractions)"
milestone: M2
lane: functions-date
status: done
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

## Outcome
Added probes/date-text-typed-clock-shapes (17 shapes, excel-graph values, ENTERED log in docs/research/typed-clock-shapes-2026-10-19.md). Excel turns every probed shape into a number except `13:00 PM` and glued `10:00AM`; formats are `m/d/yyyy h:mm`, `mm:ss.0` (fractions) or General (out-of-range components). `typed_clock_shape` replaces `ends_with_typed_clock` and picks the format; dates.md UNVERIFIED note resolved. Eval: new probe passes in full (values).
