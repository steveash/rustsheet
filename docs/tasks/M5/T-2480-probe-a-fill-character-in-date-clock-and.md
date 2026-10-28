---
id: T-2480
title: "Probe a * fill character in date, clock and fraction format codes"
milestone: M5
lane: functions-text
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence decides whether a `*x` fill adds nothing in date, clock and fraction format codes, and ours matches.

## Context
T-1735 made a fill add nothing in number text and (unprobed) in date/clock text; fraction.rs:142 still prints a space. Probe `TEXT(1,"d*x")`, `TEXT(0.5,"h:mm*-")`, `TEXT(0.5,"# ?/?*x")` (excel-probe), align code, update number-formats.md (Fill characters).

## Acceptance criteria
- [ ] Probe case with oracle values; code matches or failures are owned; number-formats.md status updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `probes/text-fill-char-date-clock-fraction` (12 cells, excel-graph): a `*x` fill adds nothing in date, clock and fraction codes. Date/clock already matched; fraction.rs printed a space, now nothing. Probe passes 12/12 (was 6/12). Blank-fraction fill unprobed (noted in number-formats.md).
