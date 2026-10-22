---
id: T-0449
title: "Criteria aggregates over large ranges: SUMIFS, COUNTIFS, AVERAGEIFS and friends stop rescanning the criteria ranges per formula"
milestone: M11
lane: perf
status: todo
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
Many SUMIFS, COUNTIFS, AVERAGEIFS, MAXIFS, MINIFS, SUMIF, COUNTIF and AVERAGEIF calls over the same large
criteria ranges recalculate in time roughly proportional to calls + range size, not calls x range size.

## Context
Found by the mog head-to-head (docs/research/mog-head-to-head-2026-10-22.md). Synthetic workbook: 2,000
`SUMIFS($C$1:$C$50000,$A$1:$A$50000,"North",$B$1:$B$50000,">t")` and 2,000 `COUNTIFS` with five
distinct text criteria and 100 numeric thresholds.
- rustsheet: 20.2 s, of which recalculation is 18.5 s.
- mog: 2.4 s, the same values.

Real files: corpus-bench 52450 takes 267 s against mog's 2.5 s (part of T-0430). Calipers
pv_goldens/pv26_002 and pv26_2_002 (1.38M formulas: about 330k COUNTIFS and 230k SUMIFS over
1,400-row ranges, many through OFFSET) take about 11 minutes each in rustsheet and in mog, so
eval/suites/calipers/suite.toml excludes them (T-0446). Each call re-parses
its criteria and re-tests every cell of every criteria range.

Candidate approach: per recalculation, memoise the set of matching rows per (criteria range,
normalised criterion), as a row bitset or sorted row list, and intersect the sets for multi-criteria
calls. Reuse the existing criteria parser and matcher so Excel's semantics stay in one place.

Shared constraints:
- Values must not change. Results are bit-identical to today's, and `make eval` shows no new known
  failure in any suite.
- A cache lives for one recalculation at most. It must not serve a value read before a cell in the
  range was recalculated in the same pass. Either build it only once every formula cell in the range
  has its value for this pass, or key it on a version that every write in the range bumps.
- Directive 5: an index's memory is bounded by the range's cell count, which the existing per-formula
  cell budget already limits.
- T-0400 (benchmark suite) and T-0402 (CI perf gate) are blocked on Phase 2. Measure with the
  existing criterion benches in crates/rustsheet/benches and record before/after release timings in
  the Outcome.

## Acceptance criteria
- [ ] A bench in crates/rustsheet/benches/aggregate.rs reproduces the synthetic workload above at no
      less than 2,000 SUMIFS + 2,000 COUNTIFS over 50,000 rows. Its recalc is at least 5x faster than on
      `main` before this task, and both timings are in the Outcome.
- [ ] corpus-bench 52450's release-build calculation time is in the Outcome; its timeout leaves
      eval/baseline/corpus-bench.txt if it now runs under 60 s.
- [ ] Calipers pv26_002 and pv26_2_002: release calculation times before and after in the Outcome. If
      they now calculate within the 60 s case timeout, their `exclude` entries leave
      eval/suites/calipers/suite.toml and `make eval-baseline` records the result; otherwise file the
      remaining cause as a follow-up.
- [ ] Unit tests cover cache invalidation within a pass, wildcard and comparison criteria,
      text-number coercion in criteria, and errors in criteria ranges.
- [ ] `make eval` shows no new known failure; `make check` green; reviewers approve.

## Out of scope
- Exact-match lookups (T-0448).
- Database functions (DSUM…) and SUMPRODUCT-style masks, unless they share the matcher for free.
