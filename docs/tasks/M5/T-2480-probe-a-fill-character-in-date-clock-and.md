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
- [x] Probe case with oracle values; code matches or failures are owned; number-formats.md status updated.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed 16 `TEXT` formulas with a `*x` fill in date, clock, fraction, percent and scientific codes (plus LEN): Excel adds nothing in every one. Date, clock, percent and scientific already matched; fraction codes printed a trailing space (`fraction.rs`), now fixed. number-formats.md updated. Probe case 32/32 pass.
