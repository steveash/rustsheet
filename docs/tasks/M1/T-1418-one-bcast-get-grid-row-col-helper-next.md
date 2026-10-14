---
id: T-1418
title: "One bcast_get(grid,row,col) helper next to bcast_idx for the repeated broadcast lookups"
milestone: M1
lane: quality
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
One `bcast_get` helper next to `bcast_idx` does the broadcast element lookup that several functions repeat.

## Context
The lookup `grid.get(bcast_idx(len,row)).and_then(|r| r.get(bcast_idx(r.len(),col)))` is written in functions/statistical/if_ifs.rs, cast.rs, functions/logical/mod.rs (several places) and functions/information/offline_errors.rs (`image_grid`, T-1370). Add `bcast_get` in arithmetic.rs and use it in each; no behavior change.

## Acceptance criteria
- [ ] Every repeated broadcast lookup uses the one helper; `check_duplication.py` does not grow.
- [ ] `make check` green; reviewers approve.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
