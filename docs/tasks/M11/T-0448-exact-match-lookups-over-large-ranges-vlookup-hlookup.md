---
id: T-0448
title: "Exact-match lookups over large ranges: VLOOKUP, HLOOKUP, MATCH, XMATCH and XLOOKUP stop rescanning the range per formula"
milestone: M11
lane: perf
status: todo
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
A workbook with many exact-match lookups into the same large range recalculates in time roughly
proportional to lookups + range size, not lookups x range size. The functions are VLOOKUP and HLOOKUP
with range_lookup FALSE, MATCH and XMATCH with match type 0, and XLOOKUP's exact modes.

## Context
Found by the mog head-to-head (docs/research/mog-head-to-head-2026-10-22.md). Synthetic workbook: 5,000
each of `VLOOKUP(k,$A$1:$B$20000,2,FALSE)`, `INDEX(B,MATCH(k,A,0))` and `XLOOKUP(k,A,B)` over 20,000
shuffled keys.
- rustsheet: 43.4 s, of which recalculation is 42.8 s.
- mog: 1.2 s end to end, the same values.

Real files: corpus-bench 59932 (VLOOKUP-heavy) takes 241 s against mog's 2.0 s, and is one of T-0430's
five timeouts.

Every call walks the lookup range linearly: `SearchVector::linear_candidates` and the VLOOKUP/MATCH
paths in crates/rustsheet-engine/src/functions/lookup_and_reference/. The sheet's column index
(sheet_data.rs `build_column_index`) orders a walk; it does not map values to rows.

The cache must keep Excel's match semantics exactly:
- text compares case-insensitively and wildcards apply (VLOOKUP, MATCH and XMATCH mode 2);
- numbers, text, booleans and errors never match each other;
- the first match wins, and the last for XLOOKUP and XMATCH search_mode -1;
- blank lookup values follow the existing rules (probes/xmatch-text-bool-equal-runs).

A lookup value with a wildcard, or a mode the cache does not cover, falls back to the linear walk.

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
- [ ] A bench in crates/rustsheet/benches (an `aggregate.rs` case, or a new `lookup.rs`) reproduces the
      synthetic workload above at no less than 5,000 lookups into 20,000 rows. Its recalc is at least
      10x faster than on `main` before this task, and both timings are in the Outcome.
- [ ] corpus-bench 59932's release-build calculation time is in the Outcome; its timeout leaves
      eval/baseline/corpus-bench.txt if it now runs under 60 s.
- [ ] Unit tests cover cache invalidation (a lookup range holding formulas recalculated in the same
      pass), case-insensitive text, mixed types, duplicate keys with first and last match, and the
      wildcard fallback.
- [ ] `make eval` shows no new known failure; `make check` green; reviewers approve.

## Out of scope
- Approximate (sorted) and binary-search modes, which are already logarithmic.
- Criteria functions (T-0449) and growing `SUM($A$1:An)` prefixes (file a follow-up if needed).
