---
id: T-2798
title: "TEXT: AM/PM before an elapsed token uses the 12-hour clock for it"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`TEXT(1.5,"AM/PM [h]")` prints "PM 12" like Excel: when an AM/PM or A/P marker comes first, the elapsed hour uses the 12-hour clock.

## Context
probes/text-elapsed-two-tokens.case.json S!A13 (excel-graph "PM 12", engine "PM 36"); dates.md "Negative serials under an explicit negative section". Discovered in T-1366. Only the marker-first order is valid; the marker after an elapsed token is #VALUE! (done).

## Acceptance criteria
- [ ] S!A13 passes and leaves eval/baseline/probes.txt; probe other elapsed tokens (`AM/PM [mm]`, `[s]`) first.
- [ ] `make check` green; reviewers approve.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
