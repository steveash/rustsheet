---
id: T-3608
title: "Row::position tries a dense row's offset before bisecting (calipers pv26_002 reference reads and result writes)"
milestone: M11
lane: perf
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Finding a cell in a dense row costs one probe, whatever was read before it: single-cell references across
a row, and the writes of formula results, no longer bisect a 755-cell row.

## Context
With T-3606's last-index hint, calipers `pv26_002`'s `SheetData::cell` is still the leaf of 35 of 97
evaluation samples (2026-11-01). Most come from `evaluate_cell` reading single-cell references such as
`I11`, `J11` and `IV11` of one row, which the single hint misses because each reads another column.
`Worksheet::cell_mut`, used to store results, has 7 more, and it never used the hint. Its rows are dense, so
a cell sits at its column's offset from the row's first column.

## Acceptance criteria
- [x] `Row::position`, past `LINEAR_SEARCH_LIMIT`, tries the index `column - first column`, checked against
      the column found there, before `binary_search`. It serves reads and writes alike.
- [x] `test_sheet_data_hint.rs` (rows of 40 columns, dense and gapped, before and after shifting writes,
      and columns at the `i32` extremes) catches the mutations that drop the check or the overflow guard.
- [x] calipers `pv26_002` release `evaluate()` before and after, in the Outcome.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing the row layout or its serialised form.

## Outcome
- **Timing:** calipers `pv26_002` release `evaluate()` went from 9.1–9.3 s (T-3606) to 8.4–8.5 s.
- **Test tidy-ups:** carried in from the T-3606 quality review. The test file drops its unused
  `unwrap_used` allow, and the down-a-column check runs once instead of once per row.
- **Review fix (security, MAJOR):** the offset is `column.checked_sub(first)`, not a plain subtraction.
  `SheetData::cell` takes any `i32`, so `i32::MIN` would have overflowed and panicked in debug builds
  (directive 5). `reads_at_extreme_columns_find_nothing` covers reads, `cell_mut` and removals at the `i32`
  extremes, and fails on the unchecked subtraction.
