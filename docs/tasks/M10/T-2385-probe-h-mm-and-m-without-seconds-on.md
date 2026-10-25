---
id: T-2385
title: "Probe [h]:mm and [m] without seconds on near-midnight serials (T-1732 follow-up)"
milestone: M10
lane: functions-text
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Excel's behavior is evidenced for `[h]:mm` and `[m]` alone on values within a second of an hour/minute boundary.

## Context
T-1732 rounds an elapsed count to the second only when a seconds token is shown; `[h]`/`[m]` without one keep their ms-snap truncation, unprobed (`DisplayClock` in formatter/elapsed.rs). Use the excel-probe skill.

## Acceptance criteria
- [ ] Probe cases for `[h]`, `[h]:mm`, `[m]` on e.g. (3-0.4/3600)/24; the engine matches or a fix is filed; number-formats.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
