---
id: T-3057
title: "Probe a * fill character in a blank fraction"
milestone: M5
lane: functions-text
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence decides what a `*x` fill prints inside a blank fraction, and ours matches.

## Context
T-2480 found a fill adds nothing in fraction codes, but a fill in a blank fraction (zero under `?/?`) is unprobed; fraction.rs `push_spaces` still counts a spacer as one space. Probe `TEXT(0,"?/?*x")`, `TEXT(0,"# ?/?*x")`, `TEXT(0,"?/?*xy")` (excel-probe), align code, update number-formats.md (Fill characters).

## Acceptance criteria
- [ ] Probe case with oracle values; code matches or failures are owned; number-formats.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `probes/text-fill-char-blank-fraction` (10 cells) filled by excel-graph; all 10 pass with the existing code, so `push_spaces` is unchanged (a Text spacer counts as its width, a fill token never reaches it). number-formats.md updated. Eval delta: +10 passing probe cells.
