---
id: T-1986
title: "Match the cell format of typed date text whose clock is rejected (13:00 PM gets m/d/yyyy)"
milestone: M2
lane: functions-date
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Typed text that stays text because its clock is rejected carries Excel's cell format, matched by the engine.

## Context
probes/date-text-typed-clock-shapes N14: `1/2/2020 13:00 PM` stays text but the cell format is `m/d/yyyy` (glued `10:00AM` stays General); see docs/research/typed-clock-shapes-2026-10-19.md and dates.md. Found by T-1979. Check whether the format is a fill artifact before implementing.

## Acceptance criteria
- [ ] Format of rejected-clock typed text matches the ENTERED log (or the finding is recorded as an artifact).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
