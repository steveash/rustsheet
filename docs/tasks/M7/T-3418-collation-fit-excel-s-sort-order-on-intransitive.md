---
id: T-3418
title: "Collation: fit Excel's SORT order on intransitive cycles of more than three texts and UNIQUE's sort-order grouping (t3211 cells)"
milestone: M7
lane: engine
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-3211]
---

## Goal
SORT and SORTBY give Excel's order on inputs of more than three texts that hold an intransitive cycle of Excel's
text comparison, and UNIQUE groups such texts as Excel does; or the cells are recorded as a DEVIATION (decision
0014) once the algorithm is shown out of reach.

## Context
- Filed by T-3211. Evidence: probes/t3211-collation-sort-cycles (excel-graph 2026-10-31); docs/excel-semantics/
  coercion.md "Not a total order", the SORT/UNIQUE part. SORT and SORTBY agree everywhere; descending is not the
  reverse of ascending; every three-text order matches the engine's natural merge sort
  (`functions::util::sort_indices_pairwise`), and so do 32 of 44 ascending and 36 of 44 descending rows, but not
  the rest of the eight-text shuffles.
- Cells owned (decision 0096): probes/t3211-collation-sort-cycles Calc!A16, A19, A20, A26, A27, A28, A30, A32,
  A33, A36, A40, A42, B14, B17, B18, B21, B23, B27, B35, B36, C16, C19, C20, C26, C27, C28, C30, C32, C33, C36,
  C40, C42, D14, D17, D18, D21, D23, D27, D35, D36 (sort orders) and G4, H4, I4 (UNIQUE of a̧, a͂, a͂͂ keeps three
  rows: Excel seems to group by adjacency in its sorted order, not by pairwise equality with any kept row).
- Tried in T-3211 (a Python simulation over the probed comparison matrix, which matches the engine's on all 81
  pairs, Calc!L1:T9): linear, binary and MSVC-style insertion sorts, selection, bubble, quicksort (first/middle
  pivot), top-down and bottom-up merge sorts with and without a sorted-halves check or a small-array cutoff,
  CPython/Java TimSort for short arrays, libc++ `__sort3` plus insertion, and natural merges (bottom-up, left fold,
  right fold, TimSort stack) with either tie rule. None fits every row. Next: probe four- to six-text inputs
  (every order of a small set) to tell the merge pattern apart, and Excel's comparison count if observable.
- Scores per candidate (rows matched, SORT ascending and descending; SORTBY gives the same), from a Python
  simulation over the probed matrix. All 44 rows were seen together: there was no hold-out, and T-3211 kept the
  best total (`nat_bu`, now `sort_indices_pairwise`), so the eight-text rows are not independent evidence for it.

  | candidate | 3-text asc | 3-text desc | 8-text asc | 8-text desc |
  |---|---|---|---|---|
  | ins_lin | 8/12 | 8/12 | 3/32 | 7/32 |
  | msvc_ins | 7/12 | 7/12 | 4/32 | 8/32 |
  | ins_front | 7/12 | 7/12 | 14/32 | 9/32 |
  | ins_bin | 8/12 | 8/12 | 4/32 | 8/32 |
  | selection | 7/12 | 7/12 | 14/32 | 9/32 |
  | bubble | 8/12 | 8/12 | 3/32 | 7/32 |
  | qs | 3/12 | 3/12 | 5/32 | 6/32 |
  | qs_mid | 10/12 | 10/12 | 4/32 | 2/32 |
  | merge_td | 8/12 | 8/12 | 14/32 | 19/32 |
  | merge_td2 | 7/12 | 7/12 | 14/32 | 19/32 |
  | merge_bu | 7/12 | 7/12 | 14/32 | 19/32 |
  | timsort_small | 12/12 | 12/12 | 4/32 | 8/32 |
  | ins3 | 12/12 | 12/12 | 3/32 | 7/32 |
  | nat_bu | 12/12 | 12/12 | 20/32 | 24/32 |
  | nat_left | 12/12 | 12/12 | 20/32 | 23/32 |
  | nat_right | 12/12 | 12/12 | 12/32 | 17/32 |
  | nat_stack | 12/12 | 12/12 | 18/32 | 19/32 |

## Acceptance criteria
- [ ] Every cell named above passes, or is an accepted deviation with a decision-0014 entry; 0 regressions.
- [ ] The sort still cannot panic on a non-total comparison (test_collation.rs
      `the_pairwise_sort_ends_on_intransitive_texts`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- GROUPBY/PIVOTBY ordering (they sort with the total `cmp_text`).
