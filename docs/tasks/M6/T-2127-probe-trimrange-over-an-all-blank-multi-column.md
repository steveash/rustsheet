---
id: T-2127
title: "Probe TRIMRANGE over an all-blank multi-column or single-row source with trim_rows 0"
milestone: M6
lane: functions-lookup
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's result for TRIMRANGE over an all-blank multi-column or single-row source with trim_rows 0 and trim_cols 1-3 is known, recorded and matched by the engine.

## Context
T-2120 probed only the single column C20:C21: `TRIMRANGE(C20:C21,0)` and `(…,0,3)` keep the range, `(…,3,0)` is `#REF!`. `trim_of` (functions/lookup_and_reference/trimrange.rs) keeps the full extent of any all-blank source while trim_rows is 0, a generalization no probe backs for C20:E21 or a single row. See lookup.md "TRIMRANGE lifted over an array, a computed reference or an error".

## Acceptance criteria
- [ ] A probe case holds the all-blank multi-column and single-row questions with excel-graph values; lookup.md records them VERIFIED and the engine matches.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
