---
id: T-1667
title: "D-function prefix criteria read lone surrogates exactly (probe first)"
milestone: M2
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
A plain text D-function criterion (DSUM, DCOUNT, ...; prefix match) holding or meeting a lone surrogate matches as Excel does.

## Context
T-1642 made COUNTIF-style criteria and wildcards read lone units exactly (docs/excel-semantics/text.md, lone surrogates, criteria paragraph), but the D-function prefix path `result_starts_with_string` (functions/criteria.rs) still lower-cases the lossy `&str`, so a lone half reads as U+FFFD. Probe first (excel-probe skill): e.g. DCOUNTA with criterion H (= MID("😀",1,1)) over H, "😀", U+FFFD, H&"a".

## Acceptance criteria
- [ ] A probe case records Excel's D-function prefix results for a lone half; the engine matches it.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
