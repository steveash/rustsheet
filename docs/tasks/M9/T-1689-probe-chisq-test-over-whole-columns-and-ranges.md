---
id: T-1689
title: "Probe CHISQ.TEST over whole columns and ranges with blanks: degrees of freedom and blank pairs"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
CHISQ.TEST over whole columns, and over ranges whose cells are partly blank, text or booleans, has
excel-graph evidence. The engine matches it, or each failing cell has a filed task.

## Context
Found in T-1681 review. `fn_chisq_test` (`functions/statistical/chisq.rs`) turns a pair with a
non-number side into `(1, 1)`, which adds 0 to the statistic. It takes the degrees of freedom from
the shape of the ranges, so `CHISQ.TEST(A:A,B:B)` counts 1,048,575 degrees of freedom whatever the
used area. The unit test `chisq_test_over_whole_columns_counts_every_cell` in
`test/test_clipped_range_walks.rs` pins this behavior, but nothing shows that Excel does the same.
Probe these with the `excel-probe` skill:
- `CHISQ.TEST(A:A,B:B)`, and the same over `A1:A10` with blanks inside.
- 2-D ranges with blanks.
- Text and booleans in either range.

## Acceptance criteria
- [ ] A probe case (≥ 8 cells) covers whole columns, blanks, text and booleans in 1-D and 2-D
      ranges, and is oracle-filled.
- [ ] The unit test's expectation agrees with the evidence, or is corrected with the fix.
- [ ] `make check` green; reviewers approve.

## Out of scope
- CHISQ.INV and CHISQ.DIST accuracy.
