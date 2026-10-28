---
id: T-2479
title: "Probe WRAPCOLS of an empty cell or 2-D array with an error or bad wrap_count, and EXPAND array pad_with past the grid"
milestone: M7
lane: functions-lookup
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence decides the order of a not-a-vector WRAPCOLS/WRAPROWS input against a non-error bad wrap_count, and how an empty-cell vector and an array pad_with past the grid behave.

## Context
T-2365 review: ours returns `#VALUE!` for `WRAPCOLS({1,2;3,4},-1)` and `#N/A` for `WRAPCOLS(Z1,NA())` (VectorError::NotVector in wrapcols_wraprows.rs), unprobed. `EXPAND(1,1048577,1,{0;1})` hung the Graph workbook (use a smaller past-grid size or split the case). Probe with excel-probe, add cells to a new probes case.

## Acceptance criteria
- [ ] Probe case with oracle values for the cells above; arrays.md updated; engine matches or failures owned.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed probes/t2479-wrapcols-notvector-expand-padarray (excel-graph, 2026-10-28). Recorded in arrays.md. Fixed `WRAPCOLS/WRAPROWS(<empty cell>, NA())`: the empty-cell `#VALUE!` precedes an error wrap_count (2-D arrays still follow it). EXPAND: rows past the grid are `#NUM!` before an array `pad_with`'s `#VALUE!` (G15, G16, G18), still before our cell bound.
