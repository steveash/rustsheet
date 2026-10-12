---
id: T-1445
title: "Probe TIMEVALUE of date-time text with an inexact clock (chrono patterns) in a 1904 workbook"
milestone: M1
lane: functions-date
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
In a 1904 workbook, TIMEVALUE of date-time text matched by the chrono patterns (`"7/7/2026 4:35 PM"`, `"2024-03-15 16:35"`) matches Excel to the last bit.

## Context
From the T-1444 fidelity review. `utils::time_of_day_from_datetime` now takes the fraction from the shifted 1904 sum; the probe only exercises this path in its pre-epoch #VALUE! cell, so the low bits of an inexact clock are unchecked.

## Acceptance criteria
- [x] A probe in a 1904 workbook with `TIMEVALUE("7/7/2026 4:35 PM")` and `TIMEVALUE("2024-03-15 16:35")`, Excel values from the oracle; the engine matches; dates.md updated.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe probes/date-system-1904-timevalue-chrono added with excel-graph values (A1-A4); the engine already matches, so there is no engine change; the baseline records the new case and no known failures. Recorded in dates.md.
