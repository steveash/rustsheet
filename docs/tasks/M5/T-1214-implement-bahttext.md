---
id: T-1214
title: "Implement BAHTTEXT"
milestone: M5
lane: functions-text
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
BAHTTEXT returns Excel's Thai-language baht text for a number.

## Context
From T-1087's probe probes/text-modes-bahttext: `BAHTTEXT(5)` and `BAHTTEXT("5")` are "ห้าบาทถ้วน" in Excel and `#NAME?` in the engine. Probe more values (satang, millions, negatives, zero, large numbers) with the excel-probe skill before implementing.

## Acceptance criteria
- [ ] probes/text-modes-bahttext passes, and a wider probe covers satang, negatives and millions.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
