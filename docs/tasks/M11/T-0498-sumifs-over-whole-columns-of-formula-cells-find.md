---
id: T-0498
title: "SUMIFS over whole columns of formula cells: find why the T-0449/T-0465 pass cache does not pay on calipers pv23_042 (6.75 s, 1,080 calls)"
milestone: M11
lane: perf
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
calipers pv_goldens/pv23_042 recalculates its 1,080 four-criteria SUMIFS in time that grows with the matches, not with
calls times rows, as T-0449 and T-0465 meant for this idiom. It is the slowest real-world case the eval runs, and its
values stay bit-identical.

## Context
Gap triage 2026-10-30 (opus refill). `rustsheet-eval run --suite calipers --suite corpus-bench --suite poi` on origin/main
c59497fc: pv23_042 takes 6,750 ms for 121,666 scored cells. Every cell passes. Its sheet `5.2` holds 1,080 cells (C9:N98)
of the form

    SUMIFS('5.8'!$P:$P,'5.8'!$I:$I,$B$7,'5.8'!$K:$K,"Person1070",'5.8'!$E:$E,'5.2'!$B9,'5.8'!$F:$F,'5.2'!C$8)

over sheet `5.8`'s 22,397 rows. Columns P, K, E and F of `5.8` are shared formulas (`P100` is
`ROUND(IF(ISNUMBER(O100),N100*O100,N100),5)`, `D100` is `DATE(YEAR(C100),MONTH(C100),1)`; 120,586 formula cells on the
sheet). The criteria are very different in selectivity:
- I = "Customer 1" (B7, the same in every call): 398 rows;
- K = "Person1070": 22,271 rows;
- E = a year (B9:B98, one per row of 5.2; 10 distinct): about 4,000-5,800 rows each;
- F = a month name (C8:N8, one per column; 13 distinct): about 3,000-4,300 rows each.

Callgrind on origin/main (`valgrind --tool=callgrind ./target/release/rustsheet-eval show
calipers/pv_goldens/pv23_042/golden.xlsx`), 39.5G instructions in total:
- `Model::evaluate` 21.2G, of which `fn_sumifs` → `sum_ifs` → `lift_values` → `lift_ifs` 20.7G
  (`call_per_reference_element` above it);
- inside it `IndexedCriterion::matcher` closure#6 5.3G and `collation::cmp_text` 4.8G; `Arc<String>::drop_slow` 4.8G
  and `CalcResult` drops 5.5G; `Model::get_cell_value` 4.1G;
- the rest is import (3.7G) and the harness's snapshot and volatile scan (`rustsheet_eval::volatile::volatile_cells`
  7.6G, T-3196). That is not engine time and not in the case's `millis`.

So the calls still test text criteria cell by cell with `cmp_text` and clone and drop cell values. The pass cache
(functions/statistical/ifs_cache.rs, T-0449) should instead have cached each column once, with one match set per
distinct criterion: about 1 + 1 + 10 + 13 sets, then intersect them. The value indexes (T-0465) should take over from the
second distinct criterion on a range. Find out which of these fails here and fix it. Candidates, none confirmed:
- A range is cached only once every formula cell in it has been evaluated in this pass. The first SUMIFS evaluates the
  shared-formula columns lazily during its walk, and the resumable check may never reach `Ready` for a 1,048,576-row
  whole column with 22,397 used rows.
- `MIN_CACHED_CELLS`/`MAX_CACHED_CELLS`/`MAX_CACHED_BITS` or the clipping of the whole column to the used rows.
- `call_per_reference_element` / `lift_ifs` taking a per-call path that skips `ifs_all_matches` for scalar criteria
  read from cells.

Also check T-0465's own counterexample note: corpus-bench 52450 turned out to be a different case (T-0477).

## Acceptance criteria
- [ ] The Outcome names the root cause, with a counter or log that shows the cache state (`RangeState`) of the four
      `5.8` columns over the pass, before and after.
- [ ] calipers pv23_042 runs in at most 2,500 ms in `rustsheet-eval run --suite calipers --filter pv23_042` on a release
      build (from 6,750 ms). The Outcome gives callgrind's `Model::evaluate` instruction count before (21.2G) and after;
      the target is at most 5G.
- [ ] A bench in crates/rustsheet/benches/aggregate.rs reproduces the shape: four criteria over formula columns of
      about 20,000 rows, one constant and selective, one constant and matching nearly all rows, one varying per row
      and one varying per column, 1,000 calls.
- [ ] Unit tests (test_ifs_cache.rs style) compare the new path with the walk on formula-cell criteria columns,
      including a cycle and a spill into a cached range.
- [ ] `make eval` with calipers, corpus-bench and poi fetched shows no new known failure; `make check` green; reviewers
      approve.

## Out of scope
- The harness's own volatile-scan cost on large workbooks (`rustsheet_eval::volatile::volatile_cells`, 19% of this
  run's instructions; T-3196).
- Defined-name parse time on load (T-3195).
