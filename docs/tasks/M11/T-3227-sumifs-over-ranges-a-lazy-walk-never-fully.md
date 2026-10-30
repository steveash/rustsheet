---
id: T-3227
title: "SUMIFS over ranges a lazy walk never fully evaluates: filter by the cached leading criteria, read the rest at the matching rows (calipers pv23_042, pv26_002)"
milestone: M11
lane: perf
status: done
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
SUMIFS-shaped calls (SUMIFS, AVERAGEIFS, MINIFS, MAXIFS, SUMIF, AVERAGEIF) whose criteria or sum
ranges hold formula cells the lazy walk never reaches get the T-0449/T-0465 caches' speed on the
ranges that are cached. The values, errors and cell reads stay exactly the walk's.

## Context
`ifs_cache.rs` caches a range only once every cell of it is inert (`read_is_inert`). The walk reads
lazily (`apply_ifs_at`): a criteria cell only where the earlier criteria matched, a sum cell only
where all of them did. So in a range of formulas, the cells on non-matching rows stay unevaluated
until the pass reaches them, and every call before that walks the whole block. `ifs_all_matches`
returns `None` at the first uncached range, and `run_ifs` then falls back to the full walk.

Measured on 2026-10-30, release, with temporary instrumentation in `ifs_range_values`:
- calipers `pv23_042` (17 s): its 1,080 `SUMIFS('5.8'!$P:$P,'5.8'!$I:$I,$B$7,'5.8'!$K:$K,"Person1070",...)`
  wait on column K, formulas `IF(OR(LEFT(J9,4)=...),"Person1070","Other")`. I, the first criteria
  range, is cached.
- calipers `pv26_002` (173 s, excluded from the suite): 169,920 waits on the sum ranges
  `'Cust x Product Filtering'!CX$11:CX$1474` and friends, which are formula columns. The criteria
  range `$D$11:$D$1474` is cached.

## Approach
- Criteria ranges `0..k` that are cached give their match sets, intersected as today. The first
  uncached range is `k`.
- The matching positions are visited in row order. At each one:
  - criteria `k..` are tested in order: a cached range by its value, an uncached one by reading its
    cell (`evaluate_cell`) as the walk does, stopping at the first that fails;
  - then the sum cell is read, from the cache or as the walk does.
- So the reads that can evaluate anything are the walk's own, in the walk's order. A row the walk
  would skip, because an earlier criterion fails, is skipped here too. Only inert reads are replaced
  by cached values, which the module docs already allow.

## Acceptance criteria
- [x] Unit tests (test_ifs_cache.rs) compare the mixed path with the walk:
      - formula criteria ranges and formula sum ranges the walk never fully evaluates;
      - errors in sum cells at matching and non-matching rows;
      - a cycle through an uncached sum range;
      - SUMIFS, AVERAGEIFS, MINIFS and MAXIFS.
- [x] calipers `pv23_042` and `pv26_002` release calculation times before and after, in the Outcome.
      If `pv26_002` and `pv26_2_002` now calculate within the 60 s case timeout, their `exclude`
      entries go and the calipers baseline is updated.
- [x] `make eval` shows no new known failure; `make check` green; reviewers approve.

## Out of scope
- COUNTIFS and COUNTIF (`count_ifs_cached`): no measured case waits there.

## Outcome
- **New path.** `ifs_leading_matches` (ifs_cache.rs) returns:
  - the match set of the criteria ranges cached before the first uncached one;
  - the cached values, or `None`, of each range from that one on.

  `ifs_all_matches` is now the case where every range is cached, so COUNTIFS is unchanged.
- **How `run_ifs` (if_ifs.rs) uses it.** It visits the leading matches in row order:
  - it tests the later criteria in order, a cached one by its value and an uncached one by reading
    its cell, stopping at the first that fails;
  - it then reads the sum cell from the cache, or as the walk does.

  So every read that can evaluate anything is one the walk makes, in its order. Only inert reads
  are served from the cache.
- **Timings,** release, `evaluate()` after load:

  | file | before | after |
  |---|---|---|
  | calipers `pv23_042` | 17.0 s | 0.50 s |
  | calipers `pv26_002` | 173 s | 71 s |
  | calipers `pv26_2_002` | — | 69 s |

  `pv26_002` and `pv26_2_002` also take 16–17 s to load, so both are still past the 60 s case
  timeout and their `exclude` entries stay. Their remaining time is SUMPRODUCT over bounded
  2-D blocks: filed T-3230.
- **Tests** (test_ifs_cache.rs) compare each formula with the pass caches off (`no_pass_caches`):
  - uncached formula sum ranges with errors at matching rows;
  - a formula criteria range after a cached one, followed by a cached one;
  - AVERAGEIFS, MINIFS, MAXIFS and SUMIF;
  - a first criteria range of formulas, which still walks;
  - a cycle through an uncached sum range at a row that only one of two calls reaches.

  A test-only counter checks that the mixed path ran.
- **Mutations:** reading the sum cell before the later criteria are tested fails the cycle test,
  and ignoring an error in an uncached sum cell fails the mixed-path test.

