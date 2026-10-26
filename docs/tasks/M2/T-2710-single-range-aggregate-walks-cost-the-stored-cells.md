---
id: T-2710
title: "Single-range aggregate walks cost the stored cells, not the used area: COUNTA(A:CV) beside one cell in CV1048576 takes 64 s"
milestone: M2
lane: engine
status: done
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
An aggregate whose argument is one range (`SUM(A:Z)`, `COUNTA(A:CV)`, `MAX(A1:J1048576)`, `LARGE(A:J,1)`, ...)
costs what the sheet's stored cells in that range cost, not every position of the used area. Results,
error order and the read-empty record stay the same. A workbook with one far-corner cell can then no
longer make a short formula run for minutes or hours (directive 5).

## Context
Gap triage 2026-10-25 (robustness, engine-level). `make eval` shows `probes/union-walk-repeated-in-lambda`
passing but taking 25.6 s, the slowest case of the run by 23x. Timing each formula on its own copy of
the case (release build, `rustsheet-eval run --root <tmp>`) shows that the union forms T-1497 made sparse
take 1 ms each. Nearly all the time is one single-range cell: Big!E3 `=SUM(BYROW(A1:A100,LAMBDA(r,SUM(B:C))))`
takes 20.5 s, beside data in B1048576 and C1048576 only. Calc!E3 `SUM(B:C)` takes 326 ms and Calc!E4
`SUM(B:B)` 85 ms over 10,000 used rows, against 1 ms for `(B:B,C:C)`.

One formula on a sheet holding a single other cell in the last row (scratch timings, release build,
`J1048576` = 1 unless stated):
- `COUNTA(A:CV)` with the cell in CV1048576: 64 s. `SUM(A:Z)` with it in Z1048576: 16.6 s.
- A:J, about 10.5 million positions: SUM 6.4 s, PRODUCT 6.4 s, AVERAGE 3.6 s, LARGE 3.7 s, GCD 3.6 s,
  AVERAGEA 3.3 s, MAX 3.2 s, COUNTBLANK 3.2 s, SUBTOTAL(9,..) 3.1 s, AGGREGATE(9,6,..) 3.1 s, COUNT 2.9 s,
  `MIN(A1:J1048576)` 2.9 s, FREQUENCY 2.8 s, AND/OR 2.7 s, STDEV, TRIMMEAN, MODE, MEDIAN, DEVSQ, SKEW,
  PERCENTILE 2.5 s each, SUMSQ 2.4 s, NPV 2.4 s, COUNTA 2.3 s, `COUNTA(1:1048576)` 9.2 s.
- That is about 250-700 ns per position. A cell in XFD1048576 makes `SUM(A:XFD)` about 1.7E10 positions,
  hours of work for one short formula.
- Lookups (MATCH, XMATCH, VLOOKUP over J:J) take about 0.2 s and `DSUM` 0.3 s. They are not in scope.
  The criteria family (COUNTIF, SUMIF, COUNTIFS, MAXIFS: 3.5-7 s) has its own loops, filed as T-2711.
  TEXTJOIN's own loop is T-2713.

Code: `Model::walk_area` (crates/rustsheet-engine/src/functions/util.rs) walks `AreaWalk::ClipBlocks`
(`for_each_position_in_area` with `of_union` false, `for_each_cell_or_error_in_range_noting_blanks`)
and `AreaWalk::ClipWholeLines` (`for_each_cell_or_error_in_whole_range`) densely over the used area,
with `evaluate_cell` per position. Only `AreaWalk::Existing`, the union walk (T-1497, T-1502), visits
stored cells, through `SheetData::next_row_between`, `columns_in_row_between` and the column index
`rows_in_column_between`. T-1497's outcome says "Single ranges are walked as before". The rule text is
docs/excel-semantics/references.md "Union area cap": "a single range argument is not counted and is walked
as before (every position of its used area)". The "Seen-range cap" section describes the read-empty
record that single-range walks build (one rectangle past the used area, cell by cell inside it).

Things to settle while doing it:
- Which callers need the blank positions themselves and which only skip them. The noting-blanks walk
  (GCD/LCM read any number of blanks as one 0, T-0438), the whole-range walk (TEXTJOIN's 3-D path,
  ignore_empty FALSE) and COUNTBLANK need a count or a flag, not a visit per blank.
- Keep the row-major order, so the first error cell stays the same (references.md, SPEC-0009).
- Keep the read-empty record correct for spills into the walked range. A rectangle for the walked area,
  as the union walk records it, instead of a record per cell.
- Whether the single-range walk draws on `MAX_AREA_WALK` like a union walk. Over dense data a step is a
  cell, so a budget would make a filled `SUM(A:Z)` `#VALUE!` where Excel computes it. Only take a budget
  if it never fires on a sheet Excel can hold (record the reasoning in references.md either way).

Since filed: T-1690 (PR #1813) walks a `ClipBlocks` single range over its stored cells when its
clipped block holds at least 65,536 positions and more than `DENSE_WALK_FACTOR` per stored cell of its
columns (`Model::walks_sparsely`, `WalkBudget::Free`, one read-empty rectangle), so `SUM(A:XFD)` beside
one far cell is bounded. Still open here: `ClipWholeLines`, COUNTBLANK's count without a visit per
blank, the work-counter test, and the probe timing.

## Acceptance criteria
- [x] Every single-range mode of `walk_area` visits stored cells only (or an equivalent bounded walk):
      the work of one walk is O(stored cells in the range + rows or chunks searched), whatever the used
      area's extent.
- [x] A test shows the bound with a work counter (positions or cells visited, or walk steps), not a
      wall-clock limit (T-2282). Example: `SUM(A:Z)` and `COUNTA(A:CV)` beside one cell in the last row
      visit at most a few cells, and `COUNTBLANK(A:J)` still counts 10,485,759 there.
- [x] `probes/union-walk-repeated-in-lambda` drops from about 25 s to under 1 s in `make eval`'s
      Timing section (release), all its cells still passing.
- [x] No baseline gains a line (`make eval-baseline` refuses regressions). The edit and math-lift
      benchmarks are not more than 50% slower (decision 0081), and a sparse whole-column bench case is
      added to show the gain.
- [x] references.md "Union area cap" and "Seen-range cap" describe the single-range walk as implemented.
- [x] `make check` green; reviewers approve.

## Out of scope
- The criteria functions' own walks (T-2711) and TEXTJOIN's own loop (T-2713).
- Array operands of whole columns (`D:D=K2`, `IF(D:D=x,F:F)`, `Model::evaluate_range` in cast.rs): T-0456.
- Per-formula CPU bounds for repeated LAMBDA work (T-2712).

## Outcome
- Both single-range modes of `Model::walk_area` now take the sparse walk when a far cell leaves the
  area mostly blank (`Model::walks_sparsely`, T-1690's rule). `ClipWholeLines`, which only TEXTJOIN's
  3-D path uses, hands on each run of blank positions as a count (`RangeItem::Blanks`), in row-major
  order between the cells it visits, so the delimiters are counted by `JoinSize::push_empties` and the
  blanks are never read. COUNTBLANK already counted the non-blank cells and took them from the area's
  size, so it needed no change of its own.
- The sparse walk no longer falls back to the dense one when the seen-range cap is spent, since that
  walk reads every position and records each blank one. Past the cap such a formula is now `#VALUE!`
  (DEVIATION, references.md "Seen-range cap"). A dense walk is still used below 65,536 positions,
  so one walk visits at most 65,536 blank positions plus four per stored cell of its columns.
- Work-counter tests (`Evaluation::cell_reads`) in test/test_aggregate_sparse_walk.rs cover 22
  aggregates beside one cell in the last row, `COUNTBLANK(A:J)` = 10485759 included, plus 3-D
  TEXTJOIN and the blank-run order. test_range_reads.rs covers the cap.
- `probes/union-walk-repeated-in-lambda` already took 0.3 s for the whole probes suite after T-1690
  (release, `rustsheet-eval run --suite probes --filter union-walk-repeated`). The bench `aggregate`
  gains two far-cell cases: a 3-D `TEXTJOIN(",",FALSE,Sheet1:Two!A:J)`, which was a dense walk of
  2 x 10,485,760 positions, now takes 0.21 ms, and a LAMBDA repeating `SUM(Two!A:J)` 100 times
  takes 0.19 ms.
- Eval delta: none. No cell changes and the baseline is unchanged.
