---
id: T-1548
title: "Probe TEXTJOIN with an error, number, boolean or blank delimiter element and the 32,767 limit with cycled delimiters"
milestone: M2
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
TEXTJOIN with an array or range delimiter containing an error, number, boolean or blank element, and the 32,767 limit with cycled delimiters of different sizes, match Excel, backed by an oracle probe.

## Context
T-1510 implemented cycling (`textjoin_delimiters`, `JoinSize::separator` in functions/text/concatenate.rs). Unprobed and recorded UNVERIFIED in text.md: `TEXTJOIN({"x",#N/A},FALSE,1,2)` (the engine returns `#N/A` even though the error delimiter is never used; Excel may give `1x2`), number/boolean/blank delimiter elements, and the limit with delimiters of different sizes.

## Acceptance criteria
- [ ] A probe case covers an error element in used and unused delimiter positions, number/boolean/blank elements, and the limit boundary with cycled delimiters; the engine matches the oracle; the UNVERIFIED notes in text.md are resolved.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
