---
id: T-0408
title: "*IF(S) lift: resolve ranges once, merge count_ifs into apply_ifs, memo repeated criteria"
milestone: M3
lane: perf
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-1477]
---

## Goal
The *IF(S) array-criteria lift (T-0207, `Model::lift_criteria` in
crates/rustsheet-engine/src/functions/statistical/if_ifs.rs) evaluates each range argument once per
formula and walks the ranges as little as possible, so `COUNTIF(A2:A10000,A2:A10000)` stays fast.

## Context
T-0207 review (performance, quality): (1) `compute` re-evaluates the value and criteria range
arguments for every distinct criteria element; with OFFSET/INDIRECT that repeats work, and a
volatile range argument could differ between elements while Excel evaluates it once. (2) The memo
only reuses consecutive equal criteria, so the occurrence-count idiom is O(n^2); a hash memo
(f64 bit patterns as keys) or one grouping pass for a single equality criterion would help.
(3) Every element builds a cloned key vector. (4) `count_ifs` and `apply_ifs` duplicate the range
resolution and walk (the old FIXME, now `TODO(T-0408)`): split to T-1477 (quality, M2) on 2026-10-13,
because M2 exit needs one criteria implementation; build on its shared helper. The cell-visit cap
(`MAX_LIFTED_CELL_VISITS`, a DEVIATION) bounds the worst case meanwhile.

## Acceptance criteria
- [ ] Ranges resolve once per formula before the lift; a volatile range argument is evaluated once.
- [ ] A bench case (SUMIFS/COUNTIFS with a 10,000-element criteria range) shows the improvement.
- [ ] probes/ifs-array-criteria and poi/70005-countifs still pass; `make check` green; reviewers approve.

## Out of scope
- Changing results.
