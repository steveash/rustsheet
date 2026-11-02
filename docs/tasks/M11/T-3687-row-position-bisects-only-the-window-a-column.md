---
id: T-3687
title: "Row::position bisects only the window a column can occupy (nearly dense rows, calipers pv26_002)"
milestone: M11
lane: perf
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A row lookup in a nearly dense row (a few gaps) costs a probe or two on neighbouring cache lines, not a
bisection of the whole row. The answer, found or insertion point, is the bisection's.

## Context
After T-3606 and T-3608, `SheetData::cell` and `Worksheet::cell_mut` are still the top two leaves of calipers
`pv26_002`'s evaluation (28 and 11 of 100 samples, 2026-11-01). Counting showed T-3608's dense-offset guess
hits about 10% of lookups. The 'Quarterly ARR Rollup' rows hold up to 755 cells with a few gaps near the
start (columns 3–7, then 9…), so every cell past a gap sits a few slots before its offset.

## Acceptance criteria
- [x] `Row::position` bisects only `len - 1 - (last - column) ..= column - first`, clamped to the row. Columns
      are distinct and ascending, so the cell, or its insertion point, lies there. The arithmetic is in `i64`.
      This replaces the single dense-offset guess, which is the window's one-cell case.
- [x] A randomized test compares `position` with a plain `binary_search` on rows of every density, for every
      column around and in them and the `i32` extremes. It catches off-by-one mutations of either bound and
      of the insertion offset.
- [x] calipers `pv26_002` release `evaluate()` before and after, in the Outcome.
- [x] `make check` green; reviewers approve.

## Out of scope
- The row layout.

## Outcome
- **Code:** `Row::window` in `sheet_data/row.rs`. `position` bisects `cells[window]` and offsets the result.
- **Timing:** calipers `pv26_002` release `evaluate()` went from 7.95–8.2 s (T-3608's guess) to 7.7–7.9 s.
- **Test:** `position_equals_a_bisection` in `sheet_data/test.rs`.
