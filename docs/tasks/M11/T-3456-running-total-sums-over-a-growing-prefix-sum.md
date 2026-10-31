---
id: T-3456
title: "Running-total SUMs over a growing prefix (SUM($A$1:A1) filled down or across) read from a per-pass prefix cache (calipers pv26_002)"
milestone: M11
lane: perf
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A running total, `SUM($A$1:A1)` filled down or `SUM($I11:I11)` filled across, costs about one cell per formula,
not the length of its range. The results stay the walk's: the same value, the same snapped final addition, the
same first error, and the same cells read.

## Context
Running totals are among the commonest spreadsheet idioms. Each copy re-walks the whole prefix, so n copies read
about n²/2 cells. Stack sampling of calipers `pv26_002` (2026-10-31, release, 12 s `evaluate()`) puts about 30%
of samples in `fn_sum` → `walk_areas`. The formulas are 110,880 copies of
`IF(AND(I11=0,I11<J11,SUM($I11:I11)>0),J11-I11-IV11,0)` across 77 columns.

SUM over one range (`sum_with_last_addition`, `walk_areas`):
- it walks the range densely, row by row, inside the used area;
- it skips blanks, adds each `cell_number`, and ends at the first error cell, which is the result;
- it charges no walk budget, which only union walks spend;
- `SumParts` keeps the total and the final addition, for the near-zero snap.

The running state after each cell of a prefix is therefore all a longer prefix from the same start needs.

## Approach
- A pass cache, keyed by the start cell and the axis, holds the `SumParts` after each cell read so far, and
  the error cell where reading stopped.
- `SUM(range)` with one 1-D range inside the used area takes its state at the range's length.
- It extends the cache first by reading the new cells in order with `evaluate_cell`, as the walk reads them. It
  stops at the first error.
- A cell read must leave the cache's conditions holding: `read_is_inert`, the T-0449 rule (evaluated, not
  circular, not a cycle member). When one fails, the cache for that start is dropped and the walk runs: it reads
  the same cells again, which is what it would have done.
- Cleared with the other pass caches, used only where `pass_cache_safe`, and bounded (directive 5).

## Acceptance criteria
- [ ] Tests compare the cached path with the walk (`no_pass_caches`): values, snapped near-zero results, the first
      error, text, booleans and blanks, formulas not evaluated yet, circular cells, iterative calculation, and
      rows and columns. Each comparison includes whether the cell is marked circular.
- [ ] calipers `pv26_002` release `evaluate()` time before and after, in the Outcome.
- [ ] `make check` green; reviewers approve.

## Out of scope
- SUM with several arguments, SUM of a 2-D block, and other aggregates (AVERAGE, COUNT and so on).
