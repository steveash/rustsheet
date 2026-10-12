---
id: T-1388
title: "Probe the 255-character limit for non-elapsed date/time TEXT and cell display"
milestone: M1
lane: functions-date
status: done
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
- [x] A probe with oracle values lands, the assumption is corrected, and number-formats.md is updated (the engine change is T-1393).
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probes `text-date-255-char-limit`, `text-format-length-limit`, `text-format-length-limit-b` (excel-graph). The 255-character result cap is never reached by repeated date tokens: Excel returns `#VALUE!` first for format codes of about 17 placeholder/literal pairs or 83+ unquoted literal characters. Cell display could not be probed (excel-graph rejects long number formats). Documented in number-formats.md; engine change filed as T-1393. No eval cell changes expected (probes have no expectations in the baseline).
