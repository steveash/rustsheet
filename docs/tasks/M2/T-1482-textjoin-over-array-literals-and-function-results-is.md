---
id: T-1482
title: "TEXTJOIN over array literals and function results is #N/IMPL!"
milestone: M2
lane: functions-text
status: done
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
- [x] Probe cases with oracle values for array-literal and function-result arguments; the engine matches.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
TEXTJOIN joins array literals and function results row-major (first error wins; empty elements and empty strings skipped only with ignore_empty). An empty string is skipped with ignore_empty everywhere (direct argument and range cell too). Probes `textjoin-array-arguments` (34 cells) and `textjoin-array-limit-empty-strings` (10 cells) pass; new unit test in `fn_textjoin.rs`. The cycling array/range delimiter found on the way is T-1509 (`textjoin-delimiter-array`). Semantics in `docs/excel-semantics/text.md`.
