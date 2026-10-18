---
id: T-1910
title: "Legacy cell: no @ and a spill for a top-level SWITCH with an array result"
milestone: M2
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
A legacy cell `=SWITCH(1,1,{2;3})` shows no `@` and spills 2, 3, as Excel's does.

## Context
probes/legacy-array-at-shapes C20 (Excel `=SWITCH(1,1,{2;3})`, ours `=@SWITCH(...)`) and C11 (Excel 3, ours empty). The nested `=T(SWITCH(...))` already matches. T-1717 `leaves_call_whole` / `legacy_array_call_at` in static_analysis/legacy.rs. Found by T-1847; references.md 'Legacy formulas put @ on array-valued calls'.

## Acceptance criteria
- [ ] The listed probes/legacy-array-at-shapes cells pass; the baseline shrinks.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
