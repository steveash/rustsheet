---
id: T-3132
title: "XMATCH and XLOOKUP compare numbers as exact doubles in every match_mode and search_mode"
milestone: M6
lane: functions-lookup
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
XMATCH and XLOOKUP order and equate numbers as exact doubles, not at 15 digits, in the exact (0), next-smaller (-1)
and next-larger (1) match modes under every search mode (1, -1, 2, -2), so a table number a few ULP from the key is
never equal to it.

## Context
- Evidence: probes/t3106-lookup-xmatch-near-equal (T-3106, excel-graph 2026-10-29); docs/excel-semantics/lookup.md
  "XMATCH, XLOOKUP and LOOKUP compare numbers as exact doubles". `XMATCH(1,{2;1+2^-52;0.5},0)` is `#N/A`,
  `XMATCH(1,{2;1+2^-52;0.5},-1)` is 3 and `XMATCH(0.3,{2;0.1+0.2;0.5},0)` is `#N/A`; the engine gives 2, 2 and 2.
- Failing cells (known failures in eval/baseline/probes.txt), Sheet1: E-AB of rows 1-20 (XMATCH E-P, XLOOKUP Q-AB)
  where the pair is equal at 15 digits, E-AB of rows 30-32 (the runs 1, 1+2^-52, 1+2*2^-52, 2 and its descending
  form) and A-L of rows 34-36. LOOKUP (A-D of rows 1-32) already passes. AC8 is the near-zero snap of `1-X`, owned
  by T-3100, not this task.
- Code: the linear modes compare through `compare_values` in `linear_scan` and `compare_blank_last`
  (functions/lookup_and_reference/xmatch.rs); the exact mode's index keys numbers on `excel_precision_key`
  (`CollatedKey`, lookup_table.rs; `ExactKey` there already keys exact bits with -0 folded to 0); the binary modes
  go through `binary_search` (functions/binary_search.rs), which uses `impl Ord for CalcResult`
  (`cmp_excel_precision`, calc_result.rs). Do not change that `Ord`: SORT and others use it. Reuse the exact number
  ordering of `compare_searched` (lookup_table.rs, T-3099) instead of writing another one.

## Acceptance criteria
- [ ] Every XMATCH and XLOOKUP cell of probes/t3106-lookup-xmatch-near-equal passes (`make eval-baseline` drops their
      known failures), no regressions in any suite.
- [ ] lookup.md's section says the engine matches.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The wildcard and regex modes with a numeric key (unprobed); the `1-X` snap (T-3100).
