---
id: T-2557
title: "LINEST/LOGEST/TREND/GROWTH: bound the matrix read of a huge range"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The regression functions never allocate a dense matrix for a range far larger than its used cells (CLAUDE.md directive 5).

## Context
`read_matrix` in functions/statistical/linest.rs (moved, not introduced, by T-2455; security review) allocates `vec![vec![0.0; n_cols]; n_rows]` from the range extent, so `LINEST(A:A)` or `LINEST(A1:XFD1048576,..)` sizes the matrix from the full grid. Reuse the existing array-size limits (`limits.rs`, `check_array_cells`; docs/excel-semantics/arrays.md resource limits) or clip to the used extent like the other range walks (test_clipped_range_walks.rs). Any deviation from Excel needs a DEVIATION entry (decision 0014).

## Acceptance criteria
- [ ] A unit test shows `LINEST(A:A,B:B)` over a few used cells finishes without a full-column allocation and gives the same value as over the used rows.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the error order recorded in statistics.md "LINEST and LOGEST report error values first".
