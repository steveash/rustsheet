---
id: T-0262
title: "Probe CONCAT/TEXTJOIN limit boundary, UTF-16 and error precedence"
milestone: M2
lane: functions-text
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for the unprobed parts of the CONCAT/TEXTJOIN 32,767-character rule, so text.md can go from "VERIFIED (web only)" with an Open line to fully VERIFIED.

## Context
T-0243 implemented the #CALC! rule from probes/text-result-limit C8, C9. Unprobed: exactly 32,767 characters is fine; the limit counts UTF-16 units (emoji count 2); an error argument after the limit (`CONCAT(REPT("a",32767),"b",NA())`, TEXTJOIN with an error in a later range cell) wins over `#CALC!` (we evaluate every argument, so the error wins). Use the `excel-probe` skill; add cells to probes/text-result-limit and fix the engine if the oracle disagrees.

## Acceptance criteria
- [ ] Probe cells for the three open points have oracle values; text.md Open line resolved; engine matches (baseline shrinks or the entry is a DEVIATION).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
