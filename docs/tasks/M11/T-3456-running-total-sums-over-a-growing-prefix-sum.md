---
id: T-3456
title: "Running-total SUMs over a growing prefix (SUM($A$1:A1) filled down or across) read from a per-pass prefix cache (calipers pv26_002)"
milestone: M11
lane: perf
status: done
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
- [x] Tests compare the cached path with the walk (`no_pass_caches`): values, snapped near-zero results, the first
      error, text, booleans and blanks, formulas not evaluated yet, circular cells, iterative calculation, and
      rows and columns. Each comparison includes whether the cell is marked circular.
- [x] calipers `pv26_002` release `evaluate()` time before and after, in the Outcome.
- [x] `make check` green; reviewers approve.

## Out of scope
- SUM with several arguments, SUM of a 2-D block, and other aggregates (AVERAGE, COUNT and so on).

## Outcome
`functions/math_and_trigonometry/sum_prefix.rs`: a per-pass cache of `SumParts` states keyed by start cell and
axis, consulted by `sum_with_last_addition` for a lone 1-D range inside the used area. A cell read that is not
inert drops the start's entry and lets the walk run; a running total read while extending its own start (`B5 =
SUM($B$1:B4)+1` reached out of order) leaves the entry as the inner read left it. At most 4,000,000 states per
pass, as the criteria caches.

Tests (`test/mathematical/test_sum_prefix.rs`) compare stored values (bit for bit) and circular marks with
`no_pass_caches` down, up and across, over near-zero cancellation, text, booleans, numeric text, blanks,
formulas evaluated on first read, an error cell, self-referencing and circular totals, and a cycle member under
iterative calculation with a recalculating edit. Mutations caught: no inert check, `== Some(false)` for the
inert check, an off-by-one hit, extending an entry an inner read already extended. Equivalent: dropping the
cached-error shortcut (falls back to the walk).

Timing (release, `evaluate()`, best of two):
- `SUM($A$1:An)` filled down n rows: 20,000 rows 4.63 s → 0.011 s; 50,000 rows 34.4 s → 0.029 s.
- calipers `pv26_002`: 12.4 s → 12.2 s. The Context's 30% was misread: the SUM samples were the first
  evaluation of the I-column formulas the walk reads, not the prefix walk itself; the cache now extends by one
  cell per call (58,000 extensions, 634,000 cells read in all). The rest of its time is SUMPRODUCT (about half
  of samples), SUMIFS, AND and COUNTIFS.
