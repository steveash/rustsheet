---
id: T-3606
title: "SheetData::cell tries the row index its last read found before searching (calipers pv26_002 lookups)"
milestone: M11
lane: perf
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A read of a cell down a column of a wide sheet costs about one probe of its row, not a bisection that
misses the cache at each step. Every read still finds exactly the cell at its position.

## Context
After T-3456, T-3471 and T-3574, calipers `pv26_002` and `pv26_2_002` are by far the slowest cases (about
28 s each in a suite run; every other case of calipers and corpus-bench is under 1.5 s). Stack samples
(2026-11-01, 104 during `evaluate()`) put 44 leaves, 42%, in `SheetData::cell`. That is the row lookup itself,
called by `evaluate_cell` from the SUM prefix fill, operators, lookups and the range-values fill.
- A row is a `Vec<(i32, Cell)>` sorted by column, and an entry is 88 bytes.
- `pv26_002`'s 'Quarterly ARR Rollup' rows hold up to 755 cells, so a bisection makes about 10 probes on
  separate cache lines, and reads down a column meet a cold row each time.
- Those reads find their cell at the same index row after row.

## Acceptance criteria
- [x] `SheetData::cell` first tries the index its last read found, checked against the column stored
      there. The hint is a `#[bitcode(skip)]` atomic (`PositionHint`, `sheet_reads.rs`): nothing is
      serialised, `Model` stays `Sync`, and a copy starts with no hint.
- [x] A test reads every position of uneven rows, across and down, before and after writes that shift
      cells, against a map of the cells.
- [x] calipers `pv26_002` release `evaluate()` before and after, in the Outcome.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing the row layout (a separate column index would change the bitcode format).

## Outcome
- **Change:** `SheetData::cell` checks `row.cells[hint]` first and stores the index it finds after a search.
  `PositionHint` lives beside `ReadLog` in `sheet_reads.rs` (relaxed atomic loads and stores).
  `sheet_data.rs` stays under 1,000 lines.
- **Timing (release `evaluate()`, calipers `pv26_002`):** 10.1 s before, 9.1–9.3 s after. corpus-bench suite
  totals are unchanged within noise (8.2 s against 8.3–8.7 s, same verdicts).
- **Test:** `test/test_sheet_data_hint.rs`. It catches the mutation that drops the hint's column check.
