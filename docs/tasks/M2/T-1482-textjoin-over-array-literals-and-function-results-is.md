---
id: T-1482
title: "TEXTJOIN over array literals and function results is #N/IMPL!"
milestone: M2
lane: functions-text
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
TEXTJOIN joins the values of an array literal or a function result (`TEXTJOIN(",",FALSE,{1,2})`, `TEXTJOIN(",",FALSE,SORT(A1:A3))`) as Excel does.

## Context
Found in T-1446: both evaluate to `#N/IMPL!` while `TEXTJOIN(",",FALSE,A1:A3)` works. Code: functions/text/concatenate.rs. Probe the Excel values (excel-probe) before fixing.

## Acceptance criteria
- [ ] Probe cases with oracle values for array-literal and function-result arguments; the engine matches.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
