---
id: T-0964
title: "FILTER if_empty array: spill the array and #SPILL! on {1,2,3} cases (ISREF.xlsx D9, J17:L17)"
milestone: M9
lane: functions-lookup
status: done
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

## Outcome
FILTER returns an array `if_empty` whole, so it spills (`fn_filter`, `spill_functions.rs`). `ISREF.xlsx` is not in the repo (T-0965 adds it), so the cells D9 and J17:L17 are covered by the new oracle probe `probes/filter-if-empty-array` (excel-graph, 2026-10-06), which mirrors them: spill across, down, 2x2, `SUM` = 6, and `#SPILL!` when blocked. Probes gains one passing case (12 cells), 0 regressions. Quirk: docs/excel-semantics/arrays.md.
