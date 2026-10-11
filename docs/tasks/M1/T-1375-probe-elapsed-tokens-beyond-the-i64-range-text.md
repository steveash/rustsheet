---
id: T-1375
title: "Probe elapsed tokens beyond the i64 range (TEXT(1E15,'[s]'), TEXT(1E300,'[s]'))"
milestone: M1
lane: functions-date
status: done
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

## Outcome
Probes elapsed-huge-{a,b0,b1,b2,c,d,e,f,g,h,i}-1900 (oracle-filled, excel-graph) record the behavior in number-formats.md. `elapsed_text` now prints 15 significant digits then zeros from 1e15, treats overflowed counts as 0, and a date/time result over 255 characters is #VALUE!. Eval: all new cells pass.
