---
id: T-0465
title: "Criteria aggregates with a new criterion on every call: value indexes per cached range (equality hash, sorted numbers) and prefix ranges"
milestone: M11
lane: perf
status: todo
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
- [ ] A bench reproduces the rank idiom and the per-row SUMIFS over 1,500 rows x 20,000 calls; its
      recalc is at least 10x faster than after T-0449, timings in the Outcome.
- [ ] calipers pv26_002 release calculation time in the Outcome; if they
      now calculate within 60 s, their timeout lines and `exclude` entries go
      (`make eval-baseline`, `rustsheet-eval update-baseline --suite calipers`).
- [ ] Unit tests compare every new index path with the walk (equality, each comparison operator,
      mixed types, text and number criteria, prefix ranges), as test_ifs_cache.rs does.
- [ ] `make eval` shows no new known failure; `make check` green; reviewers approve.

## Out of scope
- Wildcard text criteria (they keep the match-set path).
