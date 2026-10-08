---
id: T-1054
title: "DVAR, DVARP, DSTDEV and DSTDEVP use the VAR/STDEV one-pass/two-pass switch"
milestone: M1
lane: functions-stat
status: done
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
- [x] A probe case gives DVAR/DVARP/DSTDEV/DSTDEVP oracle values for data that separates the paths, and it passes.
- [x] The statistics.md entry says whether the D-functions follow the switch.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `probes/dvar-one-vs-two-pass` (32 cells, excel-graph) showed Excel's D-functions follow the VAR/STDEV switch; the engine's own one-pass sums differed from the oracle by up to 6E-13 relative on offset data (`DVAR` of 101,102,104). `fn_dvar`, `fn_dvarp`, `fn_dstdev` and `fn_dstdevp` now collect the matching numbers and call `variance_result`/`excel_variance`; the probe passes 32/32 and statistics.md records it. Behavior change beyond the digits: an overflowing Σx² now gives `#NUM!` like VAR.P (previously a non-finite number).
Eval delta: ironcalc-calc known failures 1569 -> 1560 (9 fixed), probes gains the new case, 0 regressions.
The overflow behavior is oracle-backed too: DVAR/DVARP/DSTDEV/DSTDEVP of three 1E+200 values are #NUM! (K10:K13).
