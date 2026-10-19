---
id: T-0408
title: "*IF(S) lift: resolve ranges once, merge count_ifs into apply_ifs, memo repeated criteria"
milestone: M3
lane: perf
status: done
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
- [x] Ranges resolve once per formula before the lift; a volatile range argument is evaluated once.
- [x] A bench case (SUMIFS/COUNTIFS with a 10,000-element criteria range) shows the improvement.
- [x] probes/ifs-array-criteria and poi/70005-countifs still pass; `make check` green; reviewers approve.

## Out of scope
- Changing results.

## Outcome
The *IF(S) functions resolve their ranges once per formula, before the lift (`resolve_criteria_ranges`,
`resolve_sum_ranges`, `ifs_range` in statistical/if_ifs.rs); a bad range is still every element's error, and a
volatile range is drawn once (Excel agrees: new probes/ifs-volatile-range-once, criteria.md). The lift driver
gains `Reuse::AnyEqual` (`LiftOptions::REUSE_ANY_EQUAL`, *IF(S) only): a hash pass maps each element to the first
equal one (runs still cost one slot comparison), the visit cap counts distinct elements (DEVIATION text updated),
and repeats clone their first result. `LiftCost::Walk` is gone: the walk is measured from the resolved range, only
when the call lifts. The count_ifs/apply_ifs walk merge stays out (T-1477 shared the resolution and predicates;
the two walks differ in how they treat cells past the used area).
Bench (`cargo bench -p rustsheet --bench lift`, min of 11): `SUM(COUNTIFS(D1:D10000,D1:D10000))` with 100
distinct values 85.5 s -> 0.69 s; the existing cases unchanged within noise (whole-column COUNTIFS 694 -> 668 ms,
array-criterion SUMIFS 581 -> 593 ms).
Eval: 0 regressions, 0 fixes; one new probe case (5 cells, passing).
