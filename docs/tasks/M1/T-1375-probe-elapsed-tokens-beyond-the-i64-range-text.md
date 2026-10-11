---
id: T-1375
title: "Probe elapsed tokens beyond the i64 range (TEXT(1E15,'[s]'), TEXT(1E300,'[s]'))"
milestone: M1
lane: functions-date
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's text for elapsed tokens past the i64 range is recorded and the engine matches it.

## Context
T-1362 widened `elapsed_text` (formatter/format.rs) to i64, which saturates above ~9.2e18 seconds (`TEXT(1E15,"[s]")`, `TEXT(1E300,"[s]")`). Excel's output there (digits, #VALUE!, ####) is unprobed. Use the excel-probe skill.

## Acceptance criteria
- [ ] A probe case with oracle values for huge `[s]`/`[m]`/`[h]` serials lands, the engine matches, and number-formats.md is updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
