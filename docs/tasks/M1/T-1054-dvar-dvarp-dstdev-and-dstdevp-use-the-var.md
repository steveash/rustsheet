---
id: T-1054
title: "DVAR, DVARP, DSTDEV and DSTDEVP use the VAR/STDEV one-pass/two-pass switch"
milestone: M1
lane: functions-stat
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
DVAR, DVARP, DSTDEV and DSTDEVP match Excel to the last digit, including the one-pass/two-pass switch, if Excel uses it for them.

## Context
Found by T-1037. VAR/STDEV keep the one-pass result unless it cancels or is small next to the mean.
In those cases they recompute with two passes (docs/excel-semantics/statistics.md, "VAR/STDEV switch between a one-pass and a
two-pass formula"; `excel_variance` in crates/rustsheet-engine/src/functions/statistical/variance.rs).
The database functions still use their own one-pass sums (`db_numeric_stats` in
crates/rustsheet-engine/src/functions/database.rs) with the `(ss - s*s/n)` form for both divisors.
Nothing shows whether Excel's D-functions share the VAR routine. Probe them with data that
separates the paths, for example a field holding 3,1,4,1,5,9 (one-pass), 1000000003,… (two-pass)
and 0.001,0.002,0.004 (two-pass). If they match, collect the matching values and call
`excel_variance`.

## Acceptance criteria
- [ ] A probe case gives DVAR/DVARP/DSTDEV/DSTDEVP oracle values for data that separates the paths, and it passes.
- [ ] The statistics.md entry says whether the D-functions follow the switch.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
