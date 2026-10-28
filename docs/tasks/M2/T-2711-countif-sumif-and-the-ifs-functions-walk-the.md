---
id: T-2711
title: "COUNTIF, SUMIF and the *IFS functions walk the criteria ranges' stored cells, not every used-area position (COUNTIFS(A:J,1) beside one cell in J1048576 takes 3.9 s)"
milestone: M2
lane: functions-criteria
status: done
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
COUNTIF, COUNTIFS, SUMIF, SUMIFS, AVERAGEIF, AVERAGEIFS, MAXIFS and MINIFS cost what the stored cells of
their ranges cost, not every position of the first range's used area. Results, error order and the
read-empty record stay the same: a criterion that matches blanks still counts every blank position,
arithmetically rather than by visiting each one.

## Context
Gap triage 2026-10-25 (robustness, the criteria half of T-2710). One formula on a sheet whose only other
cell is J1048576 = 1, release build, scratch timings: `COUNTIF(A:J,1)` 3.5 s, `SUMIF(A:J,1)` 3.5 s,
`COUNTIFS(A:J,1)` 3.9 s, `MAXIFS(A:J,A:J,1)` 7.0 s. That is about 10.5 million positions at 300-700 ns
each. A cell in XFD1048576 makes one `COUNTIF(A:XFD,1)` hours of work (directive 5).

Code: crates/rustsheet-engine/src/functions/statistical/if_ifs.rs.
- `count_ifs` loops `row`/`column` over the first range up to `used_area_end()`. Positions past it are
  counted at once, once per open whole row or column, when every criterion accepts `EmptyCell`.
  Every position inside it gets an `evaluate_cell` per criteria range.
- `run_ifs` (SUMIF/SUMIFS/AVERAGEIF(S)/MAXIFS/MINIFS) clips the sum range with `clip_to_used_area`,
  then calls `evaluate_cell` per position and criteria range.
- `range_walk_cost` (line 135) prices a call by the clipped area for the lift budget. It should then
  price what the new walk costs.
The sparse primitives exist: the union walk of T-1497/T-1502 (`AreaWalk::Existing`, functions/util.rs
`for_each_existing_position`; sheet_data.rs `next_row_between`, `columns_in_row_between`,
`rows_in_column_between`). T-2710 makes the shared single-range walker sparse. Reuse one definition
(directive 3, scripts/check_duplication.py). Whichever of T-2710 and this task lands second takes the
other's helper.

Semantics the sparse walk must keep (docs/excel-semantics/criteria.md):
- Blank-matching criteria: `""` matches blanks and empty-string cells, `"="` blanks only,
  `"<>abc"`/`"<>TRUE"`/`"<>#N/A"` every non-matching cell, blanks included ("\"=\" matches blank cells,
  not an empty-string cell", and the `<>` sections; probes/criteria-gaps, criteria-grammar,
  criteria-tilde-blank-single-cell-edges). So a position whose cells are all blank matches or fails as
  a whole. COUNTIFS adds the count of such all-blank positions when every criterion accepts
  `EmptyCell`, and visits only the positions where some criteria range holds a cell.
- The ranges of one call may sit on different sheets and at different offsets
  (`range.left.row + row - first_range.left.row`), and each sheet has its own used area. A position is
  "all blank" only if every aligned cell is blank.
- SUMIFS-shaped calls: a matched position contributes only if its sum cell is a number or an error. So
  if no criterion accepts `EmptyCell`, only the positions stored in every criteria range can match.
  Otherwise only the positions whose sum cell is stored can contribute. The first error sum cell in
  row-major order is the result, as today.
- Keep the read-empty records (`record_seen_empty_range`) correct for spills into any walked range.

## Acceptance criteria
- [x] The eight functions' range walks do work O(stored cells in their ranges + rows or chunks
      searched), whatever the used area's extent. A test proves it with a work counter, not a
      wall-clock limit (T-2282): `COUNTIF(A:J,1)`, `COUNTIFS(A:J,"",B:K,"<>x")` and
      `SUMIFS(A:A,B:B,"")` beside one cell in the last row visit a few cells, and `COUNTIF(A:J,"")` is
      still 10,485,759 there.
- [x] Every criteria case in eval passes as before: no baseline gains a line, `make eval-baseline`
      shows no regression.
- [x] The edit benchmark and any criteria bench are not more than 50% slower (decision 0081), and a
      sparse whole-column COUNTIFS/SUMIFS bench case shows the gain.
- [x] criteria.md (or references.md "Seen-range cap") says how the criteria walk visits cells.
- [x] `make check` green; reviewers approve.

## Out of scope
- Sharing matched-row sets across formulas (T-0449, M11 perf).
- The aggregate walker (T-2710), TEXTJOIN (T-2713), D-functions (`database.rs`, 0.3 s, not dense).

## Outcome
- `count_ifs` (COUNTIF/COUNTIFS) and `run_ifs` (SUMIF(S), AVERAGEIF(S), MAXIFS, MINIFS) walk the
  cells stored in their ranges through a new `Model::stored_walk` / `for_each_stored_position`
  (functions/util.rs): a range whose block within the used area is at most 4x the cells stored in
  its columns (plus 1,024) is walked densely in place, any other through the union walk of
  T-1497 (`for_each_existing_position`, now with a `WalkBudget::Free` mode that does not draw on
  the union budget). COUNTIFS counts the all-blank positions arithmetically; SUMIFS visits only its
  sum range's stored cells. The lift budget prices a criteria walk by the cells it visits
  (`criteria_walk_cost`), summed over every range it walks (every criteria range for COUNTIFS).
  Each walked range goes on record as read empty in one piece. Merged with T-0449's per-pass
  match-set cache: a dense plan still takes the cached match sets when every range is cached
  (COUNTIFS over the block holding every range's cells, so the cross-sheet fix holds there too).
- Fixed in passing (Excel-evidenced, probes/criteria-sparse-cross-sheet, excel-graph 2026-10-25):
  rows past the *first* range's used area were taken as blank in every range, so
  `COUNTIFS(Sheet1!A:A,"",Sheet2!A:A,"x")` with Sheet2!A100 = "x" was 0 (Excel 1). Recorded in
  criteria.md "*IF(S) criteria ranges line up position by position".
- Work counter: `Evaluation::cell_reads`; test_criteria_sparse_walk.rs shows COUNTIF(A:J,1),
  COUNTIFS(A:J,"",B:K,"<>x"), SUMIFS/SUMIF/AVERAGEIFS/MAXIFS/MINIFS beside J1048576 read < 50
  cells, and COUNTIF(A:J,"") is still 10,485,759.
- Bench (release, medians, main -> branch, this container): lift "SUMIFS, array criterion"
  38 -> 41 ms, "SUMIFS, 10,000-element criteria range" 36 -> 35 ms, "COUNTIFS, lifted whole column"
  67 -> 54 ms, "COUNTIFS, occurrence count" 53 -> 44 ms; new case "COUNTIFS and SUMIFS, sparse
  whole columns" (beside J1048576) 6.6 us (seconds on main); edit bench recalc 36.2 -> 36.1 ms.
- Eval delta: +9 probe cells (new case, all pass); no baseline change.
