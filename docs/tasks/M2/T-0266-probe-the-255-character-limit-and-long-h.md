---
id: T-0266
title: "Probe the 255-character limit and long H:M.f:S fractions inside date-clock text"
milestone: M2
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Excel's behavior for date-clock text (`1/2/2020 10:11:12.5...`) longer than 255 characters, and with an `H:M.f:S` fraction of 300+ digits, is recorded and the engine matches.

## Context
From T-0253 review. `parse_date_clock_serial` calls `parse_clock_text` without the 255-char limit that `parse_time_string`/`parse_time_serial` apply (probes/clock-text-length-limit). The engine parses a 318-char date-clock text and gives #NUM! (inf) for a 400-nines fraction; neither has oracle evidence. Also probe a fraction near 9007199254740 whole seconds (the 2^53 ms formula switch in `parse_clock_text`).

## Acceptance criteria
- [ ] Probe case oracle-filled; dates.md updated.
- [ ] The engine matches every cell, or a follow-up task is filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
