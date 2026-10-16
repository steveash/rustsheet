---
id: T-1667
title: "D-function prefix criteria read lone surrogates exactly (probe first)"
milestone: M2
lane: functions-text
status: done
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
- [x] A probe case records Excel's D-function prefix results for a lone half; the engine matches it.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `probes/d-function-prefix-lone-surrogates` (excel-graph) shows Excel reads the D-function prefix unit by unit, a lone half never U+FFFD. `result_starts_with_units` (functions/criteria.rs) now compares lower-cased UTF-16 units. Documented in criteria.md and text.md. Eval: new probe case, 35/35 cells pass; no regressions.
