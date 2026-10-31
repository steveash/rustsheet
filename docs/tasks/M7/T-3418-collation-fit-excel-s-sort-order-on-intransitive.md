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

## Acceptance criteria
- [ ] Every cell named above passes, or is an accepted deviation with a decision-0014 entry; 0 regressions.
- [ ] The sort still cannot panic on a non-total comparison (test_collation.rs
      `the_pairwise_sort_ends_on_intransitive_texts`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- GROUPBY/PIVOTBY ordering (they sort with the total `cmp_text`).
