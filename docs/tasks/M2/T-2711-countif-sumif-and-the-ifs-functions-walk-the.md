---
id: T-2711
title: "COUNTIF, SUMIF and the *IFS functions walk the criteria ranges' stored cells, not every used-area position (COUNTIFS(A:J,1) beside one cell in J1048576 takes 3.9 s)"
milestone: M2
lane: functions-criteria
status: in-progress
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
- [ ] The eight functions' range walks do work O(stored cells in their ranges + rows or chunks
      searched), whatever the used area's extent. A test proves it with a work counter, not a
      wall-clock limit (T-2282): `COUNTIF(A:J,1)`, `COUNTIFS(A:J,"",B:K,"<>x")` and
      `SUMIFS(A:A,B:B,"")` beside one cell in the last row visit a few cells, and `COUNTIF(A:J,"")` is
      still 10,485,759 there.
- [ ] Every criteria case in eval passes as before: no baseline gains a line, `make eval-baseline`
      shows no regression.
- [ ] The edit benchmark and any criteria bench are not more than 50% slower (decision 0081), and a
      sparse whole-column COUNTIFS/SUMIFS bench case shows the gain.
- [ ] criteria.md (or references.md "Seen-range cap") says how the criteria walk visits cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Sharing matched-row sets across formulas (T-0449, M11 perf).
- The aggregate walker (T-2710), TEXTJOIN (T-2713), D-functions (`database.rs`, 0.3 s, not dense).
