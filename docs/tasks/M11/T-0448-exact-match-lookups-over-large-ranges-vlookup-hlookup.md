---
id: T-0448
title: "Exact-match lookups over large ranges: VLOOKUP, HLOOKUP, MATCH, XMATCH and XLOOKUP stop rescanning the range per formula"
milestone: M11
lane: perf
status: done
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

Real files: corpus-bench 59932 was first blamed on its VLOOKUPs, but profiling (T-0455) shows they
barely register. Its time goes to 150 whole-column `MIN`/`MAX(IF($D:$D=K2,$F:$F))` formulas: first
the 15-digit number comparison, which T-0455 made cheap, and now the million-row arrays (T-0456).
Its `#ERROR!` VLOOKUPs read linked-workbook sheets the file holds no cache for (T-0457). So 59932
is not this task's measure.

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
- [x] A bench in crates/rustsheet/benches (an `aggregate.rs` case, or a new `lookup.rs`) reproduces the
      synthetic workload above at no less than 5,000 lookups into 20,000 rows. Its recalc is at least
      10x faster than on `main` before this task, and both timings are in the Outcome.
- [x] Unit tests cover cache invalidation (a lookup range holding formulas recalculated in the same
      pass), case-insensitive text, mixed types, duplicate keys with first and last match, and the
      wildcard fallback.
- [x] `make eval` shows no new known failure; `make check` green; reviewers approve.

## Out of scope
- Approximate (sorted) and binary-search modes, which are already logarithmic.
- Criteria functions (T-0449) and growing `SUM($A$1:An)` prefixes (file a follow-up if needed).

## Outcome
- A per-pass search cache (`functions/lookup_and_reference/search_cache.rs`) hands every lookup into
  the same range the vector the first one built; `SearchVector` shares its values (`Arc`) and,
  from its second search, answers exact matches from a hash index:
  - MATCH 0, VLOOKUP and HLOOKUP FALSE: numbers by exact value (-0 is 0), text by the units
    `WildcardPattern` folds to (`folded_units`, `WildcardPattern::literal_units`; a pattern with
    `*` or `?` still walks), booleans by value;
  - XMATCH and XLOOKUP match mode 0: numbers by their 15-digit value (`excel_precision_key`, the
    equality of `cmp_excel_precision`), text by `collation_key` (the equality of `cmp_text`),
    first or last for the search direction; a blank or error lookup value still walks.
- Safety (module docs of `search_cache.rs`): a vector is cached only when building it evaluated
  every cell of the range for this pass (no restart, unwind or blocked read, no read of an
  `Evaluating` cell: `Evaluation::evaluating_reads`, no circular cell); the cache is dropped with
  every reset of the cell states; nothing is cached under iterative calculation, Excel-order
  passes or outside a pass. Ranges under 64 cells are not cached; all cached vectors of a pass
  hold at most 4,000,000 cells (directive 5).
- Bench `cargo bench -p rustsheet --bench lookup` (new; 5,000 each of VLOOKUP FALSE, INDEX/MATCH 0
  and XLOOKUP into 20,000 shuffled rows), release, fastest of 3 recalculations:
  - number keys: 6.171 s on `main` → 38.4 ms (about 160x);
  - text keys: 19.565 s on `main` → 41.9 ms (about 470x).
- Real files (corpus-bench, release, wall time of one `rustsheet-eval run`, main → this branch):
  45896 20.7 → 19.8 s, 33935 8.2 → 9.7 s, 68-47 11.9 → 12.4 s, 59969 8.2 → 7.0 s; 17049, 41978,
  52450 and 59932 still pass the 60 s timeout on both. As the Context says, these are not
  lookup-bound: 52450 is criteria aggregates (T-0449) and 59932 whole-column arrays (T-0456).
- Tests: `test_search_cache.rs` (every lookup value of a mixed-type range through MATCH, VLOOKUP,
  XMATCH and XLOOKUP in both directions and wildcard mode equals the walk over a range of its
  own length; duplicates, case and types; a cache per pass with a range of formulas edited
  between evaluations; a cycle through the range; a spill into a cached range that restarts the
  pass); `collation_key_equality_is_cmp_text_equality`;
  the `cmp_excel_precision` shortcut test now also checks `excel_precision_key`. Mutation checks:
  returning the first match for a backward search, keying -0 apart from 0, keeping the cache
  across passes, and caching through a cycle each fail a test.
- Eval: `rustsheet-eval check` 0 regressions in every gated suite; corpus-bench 0 regressions.
  `check --suite calipers` lists 5 PPMT cells of pv23_056 (row 27, 1 to 4 ulp) that `main` gives
  the same (checked on a `main` build): the ungated calipers baseline predates them (T-0458).
- Review: security, architecture, quality and Excel fidelity APPROVE. MINORs done: a note on
  `Evaluation::cells` that every reset must clear `search_cache`, the index weight in the
  cache's directive-5 note, a named `VectorKey`, and a test of a spill into a cached range within
  one pass (it fails with a stale `#N/A` when the per-pass clear is removed).
