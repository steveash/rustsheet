---
id: T-1388
title: "Probe the 255-character limit for non-elapsed date/time TEXT and cell display"
milestone: M1
lane: functions-date
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's 255-character limit on date/time TEXT results is probed beyond elapsed tokens and for cell display, and the engine matches.

## Context
T-1375 made `format_number`'s date/time arm return #VALUE! over 255 characters, evidenced only for elapsed tokens (probes/elapsed-huge-g-1900). Unprobed: `TEXT(1,REPT("dddd ",60))`, and what a cell displays for such a result (maybe `####`). Use the excel-probe skill.

## Acceptance criteria
- [ ] A probe with oracle values lands, the engine matches (or the assumption is corrected), and number-formats.md is updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
