---
id: T-1215
title: "RANK/RANK.AVG with an error or text number and a non-reference array ref spill the error over the ref's shape (follow-up of T-1208)"
milestone: M1
lane: functions-stat
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
RANK, RANK.EQ and RANK.AVG with an error or text `number` and a non-reference array `ref` (`IF(A1:C1>0,A1:C1)`) return that error (or `#VALUE!` for text) in every element of the ref's shape, as Excel does.

## Context
probes/rank-nonref-errors (T-1208) cells G1,H1,G3,H3,G5,H5 and G15..H19 fail: the engine checks `number` first and returns one cell. See statistics.md "RANK of a non-reference array". Code: functions/statistical/rank_eq_avg.rs, `static_analysis_rank`. RANK.EQ is not probed separately (same implementation).

## Acceptance criteria
- [ ] probes/rank-nonref-errors passes; eval baseline shrinks accordingly.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
