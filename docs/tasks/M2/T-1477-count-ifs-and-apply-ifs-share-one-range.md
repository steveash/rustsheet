---
id: T-1477
title: "count_ifs and apply_ifs share one range resolution and criteria walk (split from T-0408)"
milestone: M2
lane: quality
status: in-progress
priority: P1
size: S
tier: sonnet
depends_on: []
---

## Goal
COUNTIF(S) and the SUMIF(S)-shaped functions (SUMIF/SUMIFS, AVERAGEIF/AVERAGEIFS, MINIFS, MAXIFS) resolve their
criteria ranges, check shapes and long criteria, and build their predicates in one place. That place is the
single criteria-matching implementation M2's exit criterion asks for.

## Context
In crates/rustsheet-engine/src/functions/statistical/if_ifs.rs, `Model::count_ifs` (marked `TODO(T-0408)`) and
`Model::apply_ifs` repeat the same block. Both evaluate each criteria range by reference, reject a cross-sheet
range or a non-range with `#VALUE!`, check `same_shape` (T-0209), check `criterion_too_long` only after the ranges
resolve (probes/long-text-args-min-criteria, probes/ifs-long-criterion-precedence), and build one
`build_criteria` predicate per criterion. They differ in two ways. apply_ifs has a sum_range (strict shape, or
SUMIF's resize via `resize_to`, T-0214). count_ifs counts every matching position instead of numeric sum cells.
T-0408 (perf, M3, Phase 2) lists this merge as one of its four criteria. It is split out here because M2 needs it
now. T-0408 keeps the per-formula range caching, the memo and the bench.

## Acceptance criteria
- [ ] One helper resolves the criteria ranges and builds the predicates. count_ifs and apply_ifs both call it, and the `TODO(T-0408)` comment is gone.
- [ ] `make eval`: 0 regressions and 0 fixes in every suite. probes/ifs-array-criteria, probes/ifs-shape-mismatch, probes/sumif-edge-single-cell and poi/70005-countifs still pass.
- [ ] scripts/check_duplication.py does not rise; record any drop with `--update` per decision 0010.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Performance work: resolving ranges once per lifted formula, the criteria memo, the bench (T-0408).
- The D-functions (they already share `build_criteria_with`).
