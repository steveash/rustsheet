---
id: T-0284
title: "Probe CONCAT/TEXTJOIN range-cell errors past the 32,767 limit"
milestone: M2
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for what CONCAT and TEXTJOIN return when a range cell holds an error and the text is already over 32,767 characters, so the extrapolated cases in text.md are verified or fixed.

## Context
T-0262 found that `TEXTJOIN(",",TRUE,REPT("a",32767),A1:A2)` with an error in `A2` is `#CALC!` (probes/text-result-limit C20) while an error argument wins. Unprobed: `CONCAT(REPT("a",32767),A1:A2)`; `TEXTJOIN(over-limit, A2:A2)` and `TEXTJOIN(over-limit, A2)` (error in the first cell of a range, single-cell reference); `TEXTJOIN(over-limit, A1:A2, NA())`. Our engine returns `#CALC!` for a first-cell range error and for the trailing `NA()`, and the error for a single-cell reference; `fn_concat` is unchanged and returns the error. Code: concatenate.rs. Use the `excel-probe` skill.

## Acceptance criteria
- [ ] Probe cells with oracle values for the four cases; the text.md Unprobed line is resolved; the engine matches (baseline shrinks or a DEVIATION entry).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
