---
id: T-0477
title: "Criteria ranges of a linked workbook's cache are #VALUE! without building their array"
milestone: M11
lane: perf
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
A criteria or sum range of the IF and IFS functions that is a range of a linked workbook's cached
sheet gives its `#VALUE!` without the engine building the range's array first.

## Context
Found by T-0430 profiling corpus-bench 52450 (release, 338 s of evaluation). Its 36,600 COUNTIFS
read whole columns of an external workbook's cached sheet (`'[1]XA Data Pull'!$J:$J`). Each such
argument was evaluated by reference (`evaluate_node_with_reference` -> `external_area_value`), which
builds the 1,048,576-row array of cached cells with one hash lookup per cell. `ifs_range`
(functions/statistical/if_ifs.rs) then turns any non-range into `#VALUE!` "Expected a range", so
the array was thrown away. The value-index plan of T-0465 does not apply here: no criterion is
ever tested.

## Acceptance criteria
- [x] The IF and IFS range arguments (criteria ranges, sum ranges, the walk-cost estimate) skip
      building the array of a cached external area of more than one cell within the array bound,
      and give the same `#VALUE!`. An uncached area, a single cell and an area past the bound keep
      today's path.
- [x] Tests: the functions over cached, uncached, single-cell, whole-column and too-large external
      ranges give today's results (also checked once by hand with the shortcut disabled), and the
      predicate is true
      only for the cached multi-cell case.
- [x] corpus-bench 52450 release timings before and after in the Outcome; its `!Timeout` line
      leaves the baseline via `make eval-baseline`.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- `Model::is_external_array` (external_links.rs) says, without reading any cell, whether a node is
  a range of a linked workbook that would be built as an array (cached cells, or `#BLOCKED!`).
  `ifs_range_arg` (if_ifs.rs) gives such an argument the `#VALUE!` "Expected a range" that
  `ifs_range` gave it after building it (one `not_a_range` for both). It is used by every IF and
  IFS range evaluation: criteria ranges, sum ranges, and the walk-cost estimate AGGREGATE also
  reads (1 for a non-range either way).
- corpus-bench 52450 (release): evaluation 338 s -> 48 ms (load 3.0 s, import). Its `!Timeout` line
  left eval/baseline/corpus-bench.txt (`make eval-baseline`: fixed 1, 0 regressions in every suite).
  All 37,641 of its cells match Excel's cached values.
- Tests (test_external_links.rs): COUNTIF/COUNTIFS/SUMIF/SUMIFS/AVERAGEIFS/MAXIFS over cached,
  uncached, single-cell, whole-column and too-large external ranges keep today's results (the same
  assertions passed once with the shortcut disabled, by hand); the predicate agrees with
  what evaluating each range builds (cached, uncached, single cell, too large, local).
- T-0465 (value indexes) still applies to calipers pv26_002, not to 52450.
