---
id: T-3153
title: "SORT lifted by_col: re-fill the T-3090 probes after Excel's 2026-10-29 change and probe a text by_col"
milestone: M7
lane: functions-lookup
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Know whether Excel's SORT lifted-by_col rule of 2026-10-29 16:00 UTC still holds, and pin a text by_col ("1", "TRUE") and a
number by_col read from a range cell.

## Context
Excel for the web answered the same t2929/t2947 workbooks differently on 2026-10-27, at 02:04 UTC and at 16:00 UTC on
2026-10-29 (T-3090; arrays.md "TOROW, TOCOL, SORT and TEXTJOIN lifted arguments"). T-3090 refilled the sidecars and refit
`sort_lifted_elements` (functions/spill_functions.rs) to the newest behavior; a staged rollout could still flip it. Whether a
text by_col that reads as a boolean sorts (and so counts toward the rule) or is skipped like "x" is unprobed.

## Acceptance criteria
- [ ] On a later day than 2026-10-29, `rustsheet-oracle fill -b excel-graph --suite probes --force` for probes/t2366-sort-lifted-by-col,
      t2929-sort-lifted-by-col-*, t2947-sort-lifted-by-col-corners and t3090-sort-lifted-by-col-broken-lead; any diff is
      recorded in arrays.md and the code follows it.
- [ ] A probe of `SORT(A1:D4,2,1,{TRUE,"1",TRUE})`, `{TRUE,"TRUE",TRUE}` and a range by_col holding numbers; the result is in
      arrays.md and passes.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
