---
id: T-1079
title: "Probe negative serials with mixed elapsed+day tokens and cell display (#####)"
milestone: M1
lane: functions-date
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle-confirm negative serials under date formats that mix an elapsed token with day/month/year tokens, and under a cell number format.

## Context
T-1035 made negative serials #VALUE! in `ParsePart::Date` (formatter/format.rs) except sections containing an elapsed token. `TEXT(-0.5,"[h] dddd")` therefore still takes the old truncate-to-0 path (unverified), and cell display of negatives under a date format (Excel shows `#####`) shares that code and is unprobed.

## Acceptance criteria
- [ ] Probe `TEXT(-0.5,"[h] dddd")`, `TEXT(-1.5,"[h]:mm yyyy")` and a cell with a date number format holding a negative value; engine matches or a fix is made; dates.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
