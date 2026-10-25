---
id: T-2684
title: "A fraction token after minutes with no seconds ([h]:mm.0) shows the tenths of the seconds"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`TEXT(x,"[h]:mm.0")` shows what Excel shows: the tenths of the seconds right after the minutes ("2:59.4").

## Context
Found by T-2385: probes/elapsed-no-seconds-boundary A32 `=TEXT((3-0.6/3600)/24,"[h]:mm.0")` is "2:59.4" in Excel
(excel-graph): the `.0` is the fraction of the (hidden) seconds 59.4; ours is #VALUE!. Where the engine rejects it: the
fraction token handling in crates/rustsheet-engine/src/formatter (`SecondFraction`, parser.rs / format.rs). Probe `h:mm.00`,
`mm.0` and `[m].0` too before generalizing (excel-probe skill).

## Acceptance criteria
- [ ] probes/elapsed-no-seconds-boundary A32 passes; the probed variants are recorded in number-formats.md and match.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
