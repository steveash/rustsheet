---
id: T-0964
title: "FILTER if_empty array: spill the array and #SPILL! on {1,2,3} cases (ISREF.xlsx D9, J17:L17)"
milestone: M9
lane: functions-lookup
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
FILTER's `if_empty` argument given as an array constant behaves as in Excel.

## Context
Excel-cached cells in ironcalc-calc/INFORMATION/ISREF.xlsx: Sheet1!D9 `FILTER(L1:L2,M1:M2,{1,2,3})` is `#SPILL!` where we return `#VALUE!`, and J17:L17 hold the spilled 1,2,3 where we return `#VALUE!` in J17. `spill_functions.rs` FILTER only wraps `if_empty` in a 1x1 array. Upstream: d7b09c33 (FILTER if_empty). Found by T-0940.

## Acceptance criteria
- [ ] ISREF.xlsx Sheet1!D9, J17, K17, L17 pass (probe the `#SPILL!` rule first if the blocking is not explained by the sheet).
- [ ] 0 eval regressions, `make check` green.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
