---
id: T-0465
title: "Criteria aggregates with a new criterion on every call: value indexes per cached range (equality hash, sorted numbers) and prefix ranges"
milestone: M11
lane: perf
status: done
priority: P1
size: M
tier: opus
depends_on: [T-0449]
---

## Goal
Criteria aggregates whose criterion changes on every call, over ranges shared by many calls,
recalculate in time roughly proportional to calls x log(range size), not calls x range size. The
idioms are the rank `COUNTIFS(I$11:I$1450,">"&I12)+1+COUNTIFS(I$11:I11,I12)`, the per-row
`SUMIFS(sum,$D$11:$D$1474,$C11)`, and date windows such as
`COUNTIFS(J:J,">="&W$1,J:J,"<="&W$1+TIME(23,59,59),...)`.

## Context
Follow-up of T-0449. T-0449 caches each criteria range's values and each criterion's match set for
the pass, which pays off when criteria repeat (its bench: 15.6 s -> 0.11 s). Real files use a new
criterion per row, so each call still tests every cell once:
- calipers pv_goldens/pv26_002 and pv26_2_002 (1.38M formulas): 272k rank COUNTIFS over I11:I1450
  plus a growing prefix I$11:I11, and 170k SUMIFS with an equality criterion per row over
  1,464 rows. Both engines take about 11 minutes (T-0446 excludes them).
- corpus-bench 52450 turned out not to be this case: its criteria ranges are a linked workbook's
  cached columns, which were built as arrays only to become `#VALUE!` (fixed by T-0477).

Candidate approach, on top of `ifs_cache.rs`'s cached range values (`RangeState::Ready`):
- an equality index per range (value -> ascending positions) for a criterion without wildcards
  or an operator, reusing the keys of T-0448 (`ExactKey`-style folding, matched to the criteria
  matcher's own equality, which must be checked: criteria equality is not MATCH's);
- a sorted index of the numeric cells for `<`, `<=`, `>`, `>=` against a number: counts by binary
  search; sums must keep the walk's row order (extended_sum rounds per addition), so a sum walks
  the matching positions in row order;
- several criteria on one range: intersect the intervals; criteria on different ranges: drive by
  the most selective one and test the rest at its positions;
- a prefix or sub-block of a cached column (`I$11:I11`) served from the column's values and indexes.

## Acceptance criteria
- [x] A bench reproduces the rank idiom and the per-row SUMIFS over 1,500 rows x 20,000 calls; its
      recalc is at least 10x faster than after T-0449, timings in the Outcome.
- [x] calipers pv26_002 release calculation time in the Outcome (540 s, not within 60 s: it
      calculates with iteration on, which turns the pass caches off; T-0485); if they
      now calculate within 60 s, their timeout lines and `exclude` entries go
      (`make eval-baseline`, `rustsheet-eval update-baseline --suite calipers`).
- [x] Unit tests compare every new index path with the walk (equality, each comparison operator,
      mixed types, text and number criteria, prefix ranges), as test_ifs_cache.rs does.
- [x] `make eval` shows no new known failure; `make check` green; reviewers approve.

## Out of scope
- Wildcard text criteria (they keep the match-set path).

## Outcome
- **Value indexes** per cached criteria range (`ValueIndex`, ifs_cache.rs):
  - the finite numbers, sorted by value then position;
  - the texts, hashed by `collation_key`, which is exactly `cmp_text` equality;
  - every other position in a list that the predicate still tests.
- **When an index is built:** from the second criterion the walk tests on a range, since one
  criterion does not repay the sort. Indexes are charged to the pass's 4,000,000-cell budget,
  their text included.
- **Which criteria use it:** criteria.rs gains `indexed_criterion`, which classifies a comparison
  with a number (or a date) and a plain text equality.
  - `build_criteria_with` builds its predicate for those from the same classification, so the
    index and the walk read the same criterion by construction. The rest of the grammar is
    unchanged, now laid out by operator.
  - Each comparison holds on one run of the sorted numbers: binary search, with equality to
    15 digits via `cmp_excel_precision`, which is monotone.
  - `<>` takes the numbers outside that run. Text, booleans, errors and blanks are still
    tested by the predicate, so the match set is the walk's.
- **Counting:** COUNTIFS with one criterion over a whole range counts from the run without
  visiting it (`ifs_count`). SUMIFS still walks the matches in row order, so sums round alike.
- **Blocks inside a cached range:** a block with the same columns, such as the growing prefix
  `I$11:I11`, of at least 64 cells, is read from the cached range. Its match set is cut from the
  range's (`window`). At most 16 cached ranges per column span are searched.
- **Last row:** `SheetData::last_row` is now kept by every write (`known_last_row`, not
  serialised, rebuilt with the column counts) instead of scanning the last chunk backwards. Every
  clipped walk asks for it, and it was 20% of this bench's recalculation.
- **Bench:** `cargo bench -p rustsheet --bench aggregate -- rank` (new case). 10,500 ranks
  (`COUNTIFS(all,">"&x)+COUNTIFS(prefix,x)`) over 7 columns, each a permutation, so no criterion
  repeats on its range, plus 10,500 numeric and 1,500 text per-row SUMIFS, over 1,500 rows.
  Release, fastest of 5: 806 ms on `main` → 72 ms, about 11x. T-0449's `-- criteria` case is
  unchanged (184 ms on `main`, 175 ms here).
- **calipers pv26_002:** release, 540 s here against 528 s after T-0449, so unchanged. The file is
  saved with iterative calculation on (`iterate="1"`), and `pass_cache_safe` refuses every pass
  cache while iterating: neither T-0449 nor this task runs on it. Its `exclude` entries stay.
  Filed T-0485 (pass caches under iterative calculation). Its SUMPRODUCT is T-0471.
- **Tests** (test_ifs_cache.rs):
  - the mixed-type pairs test gains 18 criteria: each comparison operator, `<>` with numbers,
    15-digit equality, dates, padded numbers, text-numbers, case-folded text;
  - prefixes and an inner block of a cached range against the walks of their pieces;
  - number criteria through the index over values differing past 15 digits, -0, 1E-17, dates
    and text dates;
  - the budget test now takes two criteria, to stay on match sets;
  - sheet_data.rs: the kept last row across decode, rebuild and every kind of write.
- **Mutations:** an off-by-one in `>=`'s run and a wrong shift in `window` each fail tests.
